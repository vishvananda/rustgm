#![allow(dead_code,unused_variables)]
#[must_use="DEEP"]trait Deep{}trait Left:Deep{}#[must_use="SHALLOW"]trait Right{}trait Both:Left+Right{}struct S;impl Deep for S{}impl Left for S{}impl Right for S{}impl Both for S{}fn f()->impl Both{S}fn probe(){f();}
fn main(){}
