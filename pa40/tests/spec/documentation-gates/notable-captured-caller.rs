#![allow(internal_features,unused_features,dead_code,unused_macros,unexpected_cfgs)]
#![feature(allow_internal_unstable)]
#[allow_internal_unstable(doc_notable_trait)]macro_rules! relay{($i:item)=>{$i}}relay!{#[doc(notable_trait)]pub fn answer()->u8{7}}
