#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#![feature(a_feature)]
#[unstable(feature="z_feature",issue="none",old_name="a_feature")]pub fn z(){}
#[unstable(feature="a_feature",issue="none")]pub fn a(){}