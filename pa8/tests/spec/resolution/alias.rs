#[macro_export] macro_rules! m{()=>{1}} use crate::m as alias; fn f()->i32{alias!()}
