// error-pattern: E0597|E0716
fn main() {
    let escaped = gp::with_names!(item = 7; { item });
    let _ = escaped;
}
