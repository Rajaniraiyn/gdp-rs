// error-pattern: __gdp_view lifetime are reserved
#[gp::proof] pub struct P<'__gdp_view>;
fn main() {}
