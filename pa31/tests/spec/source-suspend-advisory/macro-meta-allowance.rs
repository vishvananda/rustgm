#![feature(allow_internal_unstable)]
#[allow_internal_unstable(must_not_suspend)]macro_rules! make{($a:meta)=>{#[$a]pub struct S;}}make!(must_not_suspend);
