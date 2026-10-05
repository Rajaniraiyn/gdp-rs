# GDP benchmark method

Run `python3 bench/run.py` from the repository. It writes observations to `bench/results.json`. Each run includes its toolchain and platform so results remain attributable to the measured environment.

The runtime example compares an integer check and accumulation with the same check expressed through named values and zero-sized proof evidence. It uses `black_box`, alternates sample order, checks both results, and reports the median of nine samples. The workload uses integers.

The script also generates baseline and proof fixtures with 100 and 1000 call sites. Dependencies are compiled first. Each timed check cleans only the fixture package, retaining dependency artifacts. Cargo startup, configuration, and any local build wrappers remain part of the timing. Dependency artifacts stay warm.

Release executable sizes are recorded for generated baseline and proof fixtures. Executable size includes standard library and toolchain effects. Zero-sized representation is separately asserted by the runtime tests.

The results describe the recorded toolchain, machine, and integer workload. Application checkers and payloads determine additional query and allocation costs.
