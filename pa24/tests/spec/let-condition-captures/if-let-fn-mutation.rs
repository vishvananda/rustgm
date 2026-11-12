fn pick(k:u32)->Option<u32>{if k>1{Some(k*2)}else{None}}
fn call<F:Fn()>(f:F){f()}
fn main(){let mut total=0;call(||if let Some(v)=pick(2){total+=v;});let _=total;}
