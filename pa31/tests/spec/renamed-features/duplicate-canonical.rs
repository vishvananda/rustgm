#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#![feature(current_api)]
#[unstable(feature="current_api",issue="none",old_name="former_api")]pub fn changed()->u8{7}
#[unstable(feature="current_api",issue="none",old_name="other_old")]pub fn other()->u8{7}
