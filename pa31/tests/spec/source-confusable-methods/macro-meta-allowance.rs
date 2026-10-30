#![feature(allow_internal_unstable)]
pub struct S;#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($a:meta)=>{impl S{#[$a]pub fn replace(&self)->u32{7}}}}make!(rustc_confusables("swap"));pub fn observe()->u32{S.replace()}
