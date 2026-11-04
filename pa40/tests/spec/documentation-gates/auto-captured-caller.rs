#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(doc_cfg)]macro_rules! relay{($i:item)=>{$i}}relay!{#[doc(auto_cfg=false)]pub fn answer()->u8{7}}
