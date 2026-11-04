#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="trait_api",since="1.0.0")]pub trait Service{#[stable(feature="value_api",since="1.0.0")]#[rustc_default_body_unstable(feature="default_api",issue="none")]const VALUE:u8=7;}
#[stable(feature="local_api",since="1.0.0")]pub struct Local;
#[stable(feature="impl_api",since="1.0.0")]impl Service for Local{}
