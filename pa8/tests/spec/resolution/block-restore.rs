macro_rules! m{()=>{1}} fn f()->i32{{macro_rules! m{()=>{2}}let _=m!();}m!()}
