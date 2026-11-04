#![feature(staged_api)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="shared_api",since="1.0.0")]pub trait Service{#[stable(feature="shared_api",since="1.0.0")]#[rustc_default_body_unstable(feature="next_api",issue="none")]fn value(&self)->u8{7}}
#[stable(feature="shared_api",since="1.0.0")]#[rustc_const_stable(feature="shared_api",since="1.0.0")]pub const fn value()->u8{9}
