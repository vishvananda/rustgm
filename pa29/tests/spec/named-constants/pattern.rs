#![feature(no_core)]
#![cfg_attr(rustc_oracle, feature(lang_items))]
#![cfg_attr(rustc_oracle, allow(internal_features))]
#![no_core]
extern crate base;
// Oracle-only shim: rustc's const pattern needs `usize: PartialEq` through these
// lang items. RustGM never configures `rustc_oracle`; its primitives compare
// structurally.
#[cfg(rustc_oracle)]
#[lang = "eq"]
pub trait PartialEq<Rhs = Self> { fn eq(&self, other: &Rhs) -> bool; }
#[cfg(rustc_oracle)]
impl PartialEq for usize { fn eq(&self, other: &usize) -> bool { *self == *other } }
#[cfg(rustc_oracle)]
#[lang = "structural_peq"]
pub trait StructuralPartialEq {}
#[unsafe(no_mangle)]
pub extern "C" fn entry() -> i32 { match 4_usize { base::LANES => 42, _ => 1 } }
