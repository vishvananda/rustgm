#![allow(dead_code,unused_variables)]
#[must_use="SHARED"]trait Base{}trait Left:Base{}trait Right:Base{}trait Both:Left+Right{}struct S;impl Base for S{}impl Left for S{}impl Right for S{}impl Both for S{}fn f()->impl Both{S}fn probe(){f();}
fn main(){}
