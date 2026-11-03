#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="duplicate_api",since="1.0.0")]pub fn a(){}
#[stable(feature="duplicate_api",since="1.1.0")]pub fn b(){}
