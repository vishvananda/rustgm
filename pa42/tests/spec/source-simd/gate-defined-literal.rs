#![feature(rustc_attrs,allow_internal_unstable)]
#![allow(internal_features,dead_code)]
#[allow_internal_unstable(repr_simd)]macro_rules! define{()=>{#[repr(simd)]pub struct V([f32;4]);}}define!();
