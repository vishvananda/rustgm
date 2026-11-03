#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="module_api",since="1.0.0")]mod hidden{#[unstable(feature="current_api",issue="none",old_name="former_api")]pub fn changed()->u8{7}
}