fn m(){} macro_rules! m{()=>{42}} fn f()->i32{m();m!()}
