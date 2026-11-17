#![no_std]
#![feature(rustc_attrs)]
unsafe extern "C" {
    #[rustc_lint_query_instability]
    fn marked();
}
