macro_rules! ty{()=>{i32}} macro_rules! pat{()=>{1}} fn f(x:ty!())->i32{match x{pat!()=>1,_=>0}}
