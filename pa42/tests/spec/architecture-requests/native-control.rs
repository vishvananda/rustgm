#[target_feature(enable="sse2")]unsafe fn baseline()->u32{7}
unsafe extern "C"{fn observe(x:u32);}
fn main(){unsafe{observe(baseline());}let v=if cfg!(target_arch="x86_64")&&!cfg!(target_arch="aarch64")&&cfg!(target_feature="sse2")&&!cfg!(target_feature="avx2"){9}else{0};unsafe{observe(v);}}
