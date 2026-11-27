#![feature(aarch64_unstable_target_feature,arm_target_feature,avx10_target_feature,clflushopt_target_feature,hexagon_target_feature,loongarch_target_feature,mips_target_feature,movrs_target_feature,nvptx_target_feature,powerpc_target_feature,riscv_target_feature,rtm_target_feature,s390x_target_feature,wasm_target_feature,x86_amx_intrinsics)]
#[target_feature(enable="sse2")]unsafe fn baseline()->u32{7}
unsafe extern "C"{fn observe(x:u32);}
fn main(){unsafe{observe(baseline());}let v=if cfg!(target_arch="x86_64")&&!cfg!(target_arch="aarch64")&&cfg!(target_feature="sse2")&&!cfg!(target_feature="avx2"){9}else{0};unsafe{observe(v);}}
