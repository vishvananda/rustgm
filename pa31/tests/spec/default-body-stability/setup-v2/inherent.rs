#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="local_api",since="1.0.0")]pub struct Local;
#[stable(feature="impl_api",since="1.0.0")]impl Local{#[rustc_default_body_unstable(feature="default_api",issue="none")]fn f()->u8{7}}
