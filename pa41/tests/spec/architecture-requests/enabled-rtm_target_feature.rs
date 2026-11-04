#![feature(aarch64_unstable_target_feature,aarch64_ver_target_feature,arm_target_feature,avr_target_feature,bpf_target_feature,csky_target_feature,hexagon_target_feature,loongarch_target_feature,m68k_target_feature,mips_target_feature,nvptx_target_feature,powerpc_target_feature,riscv_target_feature,s390x_target_feature,sparc_target_feature,wasm_target_feature,xtensa_target_feature)]
#![feature(rtm_target_feature)]
#[target_feature(enable="rtm")]unsafe fn probe(){}fn main(){}
