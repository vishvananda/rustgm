macro_rules! owner{()=>{#[rustc_has_incoherent_inherent_impls]pub trait T{}}}
owner!();
