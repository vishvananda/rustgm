macro_rules! m{()=>{1}} fn f()->i32{let a=m!();macro_rules! m{()=>{2}} a+m!()}
