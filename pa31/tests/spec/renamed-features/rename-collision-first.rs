#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#![feature(z_feature)]
#[unstable(feature="a_feature",issue="none",old_name="z_feature")]pub fn a(){}
#[unstable(feature="z_feature",issue="none")]pub fn z(){}