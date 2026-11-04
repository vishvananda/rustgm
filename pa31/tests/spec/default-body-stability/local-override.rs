#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="trait_api",since="1.0.0")]pub trait Service{#[stable(feature="method_api",since="1.0.0")]#[rustc_default_body_unstable(feature="default_api",issue="none")]fn value(&self)->u8{7}}
#[stable(feature="local_api",since="1.0.0")]pub struct Local;
impl Service for Local{fn value(&self)->u8{9}}
