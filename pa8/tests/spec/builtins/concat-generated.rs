macro_rules! make{()=>{#[macro_export] macro_rules! m{()=>{"a"}}}} fn f()-> &'static str{concat!(crate::m!(),"b")} make!();
