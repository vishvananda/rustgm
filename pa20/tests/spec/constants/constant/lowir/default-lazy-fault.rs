#![allow(unconditional_panic)]
trait A{const C:u8=1/0;}impl A for u8{}fn main(){}
