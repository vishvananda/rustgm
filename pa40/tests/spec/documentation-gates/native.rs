#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(doc_cfg,doc_notable_trait)]
#[doc(notable_trait)]trait T{fn value(&self)->u8;}impl T for u8{fn value(&self)->u8{9}}#[doc(cfg(any()))]#[doc(auto_cfg=false)]#[unsafe(no_mangle)]pub extern "C" fn observed_cfg()->u8{7}#[unsafe(no_mangle)]pub extern "C" fn observed_notable()->u8{9u8.value()}
