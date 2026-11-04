#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#![feature(default_api)]
#[stable(feature="item_api",since="1.0.0")]pub struct Data(#[rustc_default_body_unstable(feature="default_api",issue="none")]u8);