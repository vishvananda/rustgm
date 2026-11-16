//! The allocator link contract as rustc 1.98.0 implements it: internal
//! symbol names, `#[global_allocator]`, `#[alloc_error_handler]` and the
//! allocator shim of a program that needs an allocator.
//!
//! Every outcome was declared before either compiler ran, with rustc 1.98.0
//! (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 -Cpanic=abort
//! -Copt-level=0 --cap-lints=allow`) as the oracle; `rustc:` names what it
//! does with the same bytes. The sources are a mini core followed by a mini
//! alloc in one crate.
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CORE: &str = r#"#![feature(no_core, lang_items, rustc_attrs, decl_macro, allow_internal_unstable, auto_traits)]
#![feature(const_trait_impl)]
#![feature(allocator_internals)]
#![no_core]
#![allow(internal_features, dead_code, unused_unsafe)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
impl Clone for usize {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for usize {}
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "destruct"]
#[rustc_deny_explicit_impl]
pub const trait Destruct: PointeeSized {}
#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
}
pub mod mem {
    pub struct Alignment(usize);
    impl crate::Clone for Alignment {
        fn clone(&self) -> Self {
            *self
        }
    }
    impl crate::Copy for Alignment {}
    impl Alignment {
        pub const unsafe fn new_unchecked(align: usize) -> Self {
            Alignment(align)
        }
        pub const fn as_usize(self) -> usize {
            self.0
        }
    }
}
pub mod alloc {
    use crate::mem::Alignment;
    pub struct Layout {
        size: usize,
        align: Alignment,
    }
    impl Layout {
        pub const unsafe fn from_size_alignment_unchecked(size: usize, align: Alignment) -> Self {
            Layout { size, align }
        }
        pub const unsafe fn from_size_align_unchecked(size: usize, align: usize) -> Self {
            Layout {
                size,
                align: unsafe { Alignment::new_unchecked(align) },
            }
        }
        pub const fn size(&self) -> usize {
            self.size
        }
        pub const fn align(&self) -> usize {
            self.align.as_usize()
        }
    }
    pub unsafe trait GlobalAlloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8;
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout);
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8;
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8;
    }
}
pub mod macros {
    #[allow_internal_unstable(rustc_attrs)]
    #[rustc_builtin_macro]
    pub macro global_allocator($item:item) {}
    #[allow_internal_unstable(rustc_attrs)]
    #[rustc_builtin_macro]
    pub macro alloc_error_handler($item:item) {}
}
"#;
const INTERNAL_NAMES: &str = r#"unsafe extern "Rust" {
    #[rustc_std_internal_symbol]
    fn __rust_alloc(size: usize, align: usize) -> *mut u8;
    #[rustc_std_internal_symbol]
    #[link_name = "linked"]
    fn imported();
}
#[rustc_std_internal_symbol]
pub unsafe fn __rdl_alloc_error_handler(size: usize, _align: usize) -> *mut u8 {
    unsafe {
        imported();
        __rust_alloc(size, 8)
    }
}
#[rustc_std_internal_symbol]
#[unsafe(export_name = "renamed")]
pub fn exported() {}
"#;

/// The `LowIR` of one crate lowered from its exports, without an entry.
fn lowered(source: &str) -> Result<String> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(source, "global_allocators", &config)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program, &mut text)?;
    Ok(String::from_utf8(text)?)
}
/// Whether `text` defines (`declared` false) or only declares `symbol`.
fn has_symbol(text: &str, symbol: &str, declared: bool) -> bool {
    let line = format!("function @{symbol}(");
    text.lines().any(|row| {
        row.strip_prefix("declare ")
            .map_or((row.starts_with(&line), false), |rest| {
                (rest.starts_with(&line), true)
            })
            == (true, declared)
    })
}

/// rustc: defines `_RNvCs6rREvFdRhLb_7___rustc25___rdl_alloc_error_handler`
/// and `_RNvCs6rREvFdRhLb_7___rustc7renamed` (an `export_name` is mangled
/// too) and references `_RNvCs6rREvFdRhLb_7___rustc12___rust_alloc` and
/// `_RNvCs6rREvFdRhLb_7___rustc6linked` (so is a `link_name`), as `nm` shows.
#[test]
fn internal_symbols_take_the_version_mangled_names_rustc_gives_them() -> Result {
    let text = lowered(&format!("{CORE}{INTERNAL_NAMES}"))?;
    for (symbol, declared) in [
        (
            "_RNvCs6rREvFdRhLb_7___rustc25___rdl_alloc_error_handler",
            false,
        ),
        ("_RNvCs6rREvFdRhLb_7___rustc7renamed", false),
        ("_RNvCs6rREvFdRhLb_7___rustc12___rust_alloc", true),
        ("_RNvCs6rREvFdRhLb_7___rustc6linked", true),
    ] {
        assert!(has_symbol(&text, symbol, declared), "{symbol}\n{text}");
    }
    for unmangled in [
        "@__rdl_alloc_error_handler(",
        "@renamed(",
        "@__rust_alloc(",
        "@linked(",
    ] {
        assert!(!text.contains(unmangled), "{unmangled}\n{text}");
    }
    Ok(())
}
