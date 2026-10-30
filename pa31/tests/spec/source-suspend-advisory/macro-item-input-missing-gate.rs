#![feature(allow_internal_unstable)]
#[allow_internal_unstable(must_not_suspend)]macro_rules! make{($i:item)=>{$i}}make!(#[must_not_suspend]pub struct S;);
