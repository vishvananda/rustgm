#![allow(internal_features,unused_features,dead_code,unused_macros,unused_attributes,unexpected_cfgs)]
#![feature(link_cfg)]
#[link(name="candidate",cfg(all()))]pub fn answer()->u8{7}
