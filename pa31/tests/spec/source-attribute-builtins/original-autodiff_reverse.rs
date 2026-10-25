#![feature(no_core,lang_items,rustc_attrs,decl_macro,allow_internal_unstable,staged_api)]
#![no_core]
#![stable(feature="attribute_fixture",since="1.0.0")]
#[stable(feature="attribute_fixture",since="1.0.0")]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[stable(feature="attribute_fixture",since="1.0.0")]
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[stable(feature="attribute_fixture",since="1.0.0")]
#[lang="sized"]pub trait Sized:MetaSized{}
    #[unstable(feature = "autodiff", issue = "124509")]
    #[allow_internal_unstable(rustc_attrs)]
    #[allow_internal_unstable(core_intrinsics)]
    #[rustc_builtin_macro]
    pub macro autodiff_reverse($item:item) {
        /* compiler built-in */
    }
#[stable(feature="attribute_fixture",since="1.0.0")]
#[unsafe(no_mangle)]pub extern "C" fn answer()->u32{7u32}
