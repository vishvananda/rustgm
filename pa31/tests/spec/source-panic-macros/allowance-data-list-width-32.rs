#![feature(no_core, lang_items, rustc_attrs, allow_internal_unstable)]
#![no_core]
#[lang = "pointee_sized"] pub trait PointeeSized {}
#[lang = "meta_sized"] pub trait MetaSized: PointeeSized {}
#[lang = "sized"] pub trait Sized: MetaSized {}
#[allow_internal_unstable(fixture_feature_0, fixture_feature_1, fixture_feature_2, fixture_feature_3, fixture_feature_4, fixture_feature_5, fixture_feature_6, fixture_feature_7, fixture_feature_8, fixture_feature_9, fixture_feature_10, fixture_feature_11, fixture_feature_12, fixture_feature_13, fixture_feature_14, fixture_feature_15, fixture_feature_16, fixture_feature_17, fixture_feature_18, fixture_feature_19, fixture_feature_20, fixture_feature_21, fixture_feature_22, fixture_feature_23, fixture_feature_24, fixture_feature_25, fixture_feature_26, fixture_feature_27, fixture_feature_28, fixture_feature_29, fixture_feature_30, fixture_feature_31)]
macro_rules! route { () => { 7u32 }; }
#[unsafe(no_mangle)] pub extern "C" fn answer()->u32 { route!() }
