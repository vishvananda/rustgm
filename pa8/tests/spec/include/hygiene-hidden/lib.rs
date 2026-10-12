macro_rules! m{()=>{{let x=1;include!("part.rs")}}} fn f()->i32{m!()}
