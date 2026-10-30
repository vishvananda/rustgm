#![feature(const_trait_impl)]
const trait Mixed{#[rustc_non_const_trait_method]fn f();}
