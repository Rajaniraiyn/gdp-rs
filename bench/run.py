#!/usr/bin/env python3
"""Measure reproducible local observations; no benchmark dependencies required."""
import json
import os
import platform
from pathlib import Path
import statistics
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parent.parent
CARGO = os.environ.get("CARGO", "cargo")


def run(*args, cwd=ROOT, env=None):
    result = subprocess.run(args, cwd=cwd, env=env, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    return result.stdout.strip()


def fixture_source(sites, proofs):
    if proofs:
        header = """mod p {
 #[gp::proof] pub struct Positive<'a>;
 pub fn check<'a>(v: &gp::Named<'a,u64>) -> Option<Positive<'a>> {
  (*v.value()>0).then(||Positive::issue(v))
 }
}
"""
        statement = "{ gp::name!(v=std::hint::black_box(value)); if let Some(proof)=p::check(&v) { let _=std::hint::black_box(proof); total=total.wrapping_add(*v.value()); } }"
    else:
        header = ""
        statement = "{ let v=std::hint::black_box(value); if v>0 { total=total.wrapping_add(v); } }"
    return header + "fn main(){let value=std::env::args().count() as u64;let mut total=0_u64;\n" + "\n".join([statement] * sites) + "\nstd::hint::black_box(total);}\n"


def main():
    observations = {
        "rustc": run("rustc", "--version"),
        "cargo": run(CARGO, "--version"),
        "platform": platform.platform(),
        "runtime": run(CARGO, "run", "--quiet", "--release", "--example", "overhead", "--features", "macros"),
        "fixtures": [],
    }
    with tempfile.TemporaryDirectory(prefix="gdp-bench-") as temporary:
        directory = Path(temporary)
        (directory / "src/bin").mkdir(parents=True)
        path_literal = json.dumps(str(ROOT))
        (directory / "Cargo.toml").write_text(
            '[package]\nname="gdp-bench-fixture"\nversion="0.0.0"\nedition="2024"\nrust-version="1.85"\n'
            '[workspace]\n[dependencies]\n'
            f'gp={{package="ghostproof",path={path_literal},features=["macros"]}}\n'
        )
        environment = dict(os.environ, CARGO_TARGET_DIR=str(directory / "build"))
        for sites in [100, 1000]:
            for proofs in [False, True]:
                name = ("proof" if proofs else "plain") + str(sites)
                (directory / f"src/bin/{name}.rs").write_text(fixture_source(sites, proofs))
                run(CARGO, "check", "--offline", "--bin", name, cwd=directory, env=environment)
                elapsed = []
                for _ in range(3):
                    run(CARGO, "clean", "-p", "gdp-bench-fixture", cwd=directory, env=environment)
                    start = time.perf_counter()
                    run(CARGO, "check", "--offline", "--bin", name, cwd=directory, env=environment)
                    elapsed.append(time.perf_counter() - start)
                run(CARGO, "build", "--offline", "--release", "--bin", name, cwd=directory, env=environment)
                executable = directory / "build/release" / (name + (".exe" if os.name == "nt" else ""))
                observations["fixtures"].append({
                    "sites": sites, "proofs": proofs, "check_seconds": elapsed,
                    "check_median_seconds": statistics.median(elapsed),
                    "executable_bytes": executable.stat().st_size,
                })
    output = ROOT / "bench/results.json"
    output.write_text(json.dumps(observations, indent=2) + "\n")
    print(output)


if __name__ == "__main__":
    main()
