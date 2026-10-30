#![feature(rustc_attrs,const_trait_impl,staged_api)]
#![stable(feature="nonconst_members",since="1.0.0")]
#![allow(internal_features,dead_code)]
#[stable(feature="nonconst_members",since="1.0.0")]#[rustc_const_unstable(feature="const_members",issue="none")]pub const trait Mixed{#[stable(feature="nonconst_members",since="1.0.0")]#[rustc_non_const_trait_method]fn runtime(&self)->u8{7}}
