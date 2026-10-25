#![allow(dead_code,unused_variables)]
#[non_exhaustive]pub struct Named{pub x:u8}
#[non_exhaustive]pub struct Tuple(pub u8);
#[non_exhaustive]pub struct Unit;
fn probe()->u8{let Named{x}=Named{x:7u8};let Tuple(y)=Tuple(2u8);let Unit=Unit;x+y}
