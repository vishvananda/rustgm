#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#![feature(old_const_api)]
#[stable(feature="regular_api",since="1.0.0")]#[rustc_const_unstable(feature="const_api",issue="none",old_name="old_const_api")]pub const fn changed()->u8{7}