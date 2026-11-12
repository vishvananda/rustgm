fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn pick(k:u32)->Option<u32>{if k>1{Some(k*2)}else{None}}
fn run(k:u32)->u32{let f=||if let Some(v)=pick(k){v+k}else{0};f()}
fn main(){check(run(3)==9);check(run(1)==0);}
