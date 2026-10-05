// error-pattern: mismatched types
mod support;
fn main() { gp::name!(u=1_u32,p=2_u32); let plan=support::policy::plan(&p); let _:gp::Either<support::policy::Admin<'_, '_>, ()>=gp::Either::Left(plan); }
