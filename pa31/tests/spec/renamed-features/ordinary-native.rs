#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[unstable(feature="current_api",issue="none",old_name="former_api")]pub fn changed()->u8{7}

#[unsafe(no_mangle)]pub extern "C" fn observed()->u8{changed()}