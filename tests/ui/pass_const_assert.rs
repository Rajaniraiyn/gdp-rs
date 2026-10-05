const fn valid(value: usize) -> bool { value > 0 && value <= 128 }
gp::const_assert!(valid(64), "invalid configuration");
fn main() { gp::const_assert!(const { 1 + 2 } == 3); }
