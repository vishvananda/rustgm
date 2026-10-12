fn f()->i32{let x=1;macro_rules! m{()=>{x}} let x=2;m!()}
