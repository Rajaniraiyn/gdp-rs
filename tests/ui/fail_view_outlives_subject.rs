// error-pattern: does not live long enough
mod support;
fn main() {
    gp::make_guard!(guard);
    let proof;
    let view;
    { let p=gp::Named::new(2_u32,guard); proof=support::policy::plan(&p); view=proof.view(&p); }
    let _=view.subject_0();
}
