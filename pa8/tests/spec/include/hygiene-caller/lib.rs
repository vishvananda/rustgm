macro_rules! m{()=>{include!("part.rs")}} fn f()->i32{let x=1;m!()}
