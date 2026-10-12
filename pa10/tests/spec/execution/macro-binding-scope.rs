macro_rules! bind{($v:ident,$e:expr)=>{let $v=$e;}}fn entry()->u64{bind!(x,40);{bind!(x,7);let _=x;}bind!(y,2);x+y}
