#![no_std]
#![feature(rustc_attrs)]
#[rustc_lint_query_instability]
fn free() {}
struct Actual;
impl Actual {
    #[rustc_lint_query_instability]
    pub fn keys(&self) {}
}
trait Owner {
    #[rustc_lint_query_instability]
    fn required(&self);
    #[rustc_lint_query_instability]
    fn provided(&self) {}
}
impl Owner for Actual {
    #[rustc_lint_query_instability]
    fn required(&self) {}
}
