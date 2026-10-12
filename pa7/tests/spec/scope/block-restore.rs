macro_rules! m {(a)=>{}} fn f(){macro_rules! m {(b)=>{}} m!(b);} m!(a);
