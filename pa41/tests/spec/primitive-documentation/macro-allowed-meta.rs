#![feature(allow_internal_unstable)]
#[allow_internal_unstable(rustc_attrs)]macro_rules! make{($m:meta)=>{#[$m]mod docs{}}}make!(rustc_doc_primitive="bool");
