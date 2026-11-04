#![feature(staged_api,rustc_attrs)]
#![allow(internal_features,dead_code)]
#![stable(feature="root_api",since="1.0.0")]
#[stable(feature="trait_api",since="1.0.0")]pub trait Service{#[stable(feature="method_api",since="1.0.0")]#[rustc_default_body_unstable(feature="default_api",issue="none")]fn value(&self)->u8{7}}
#[stable(feature="local_api",since="1.0.0")]pub struct Local;
impl Service for Local{}
struct Override;impl Service for Override{fn value(&self)->u8{9}}
fn call<T:Service>(value:&T)->u8{value.value()}
#[stable(feature="observer_api",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn observed_default()->u8{call(&Local)}
#[stable(feature="observer_api",since="1.0.0")]#[unsafe(no_mangle)]pub extern "C" fn observed_override()->u8{call(&Override)}
