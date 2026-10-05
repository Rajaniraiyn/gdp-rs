// error-pattern: E0597|E0716
mod support;
fn main() {
    let escaped = gp::with_names!(u = 1_u32, p = 7_u32; { support::policy::admin(&u, &p) });
    let _ = escaped;
}
