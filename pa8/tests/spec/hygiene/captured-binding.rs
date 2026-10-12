macro_rules! m{($x:ident)=>{let $x=2;}} fn f()->i32{m!(x);x}
