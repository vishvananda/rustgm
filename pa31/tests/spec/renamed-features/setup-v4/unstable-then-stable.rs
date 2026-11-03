#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[unstable(feature="duplicate_api",issue="none",old_name="old_api")]pub fn a(){}
#[stable(feature="duplicate_api",since="1.0.0")]pub fn b(){}
