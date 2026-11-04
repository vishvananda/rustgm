#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
macro_rules! make{($m:meta)=>{#[$m]unsafe extern "C"{pub fn external()->u8;}}}make!(link(name="candidate",cfg(all())));
