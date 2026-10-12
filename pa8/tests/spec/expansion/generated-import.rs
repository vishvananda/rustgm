mod a{pub fn f()->i32{42}} macro_rules! m{()=>{use a::f;}} m!(); fn g()->i32{f()}
