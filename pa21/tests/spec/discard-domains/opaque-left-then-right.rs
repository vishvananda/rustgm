#![allow(dead_code,unused_variables)]
#[must_use="LEFT"]trait Left{}#[must_use="RIGHT"]trait Right{}struct S;impl Left for S{}impl Right for S{}fn f()->impl Left+Right{S}fn probe(){f();}
fn main(){}
