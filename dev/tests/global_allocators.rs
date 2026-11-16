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
const ALLOC: &str = r#"pub mod rt_alloc {
    use crate::mem::Alignment;
    unsafe extern "Rust" {
        #[rustc_allocator]
        #[rustc_nounwind]
        #[rustc_std_internal_symbol]
        #[rustc_allocator_zeroed_variant = "__rust_alloc_zeroed"]
        fn __rust_alloc(size: usize, align: Alignment) -> *mut u8;
        #[rustc_deallocator]
        #[rustc_nounwind]
        #[rustc_std_internal_symbol]
        fn __rust_dealloc(ptr: *mut u8, size: usize, align: Alignment);
        #[rustc_reallocator]
        #[rustc_nounwind]
        #[rustc_std_internal_symbol]
        fn __rust_realloc(ptr: *mut u8, old_size: usize, align: Alignment, new_size: usize) -> *mut u8;
        #[rustc_allocator_zeroed]
        #[rustc_nounwind]
        #[rustc_std_internal_symbol]
        fn __rust_alloc_zeroed(size: usize, align: Alignment) -> *mut u8;
        #[rustc_nounwind]
        #[rustc_std_internal_symbol]
        fn __rust_no_alloc_shim_is_unstable_v2();
        #[rustc_std_internal_symbol]
        fn __rust_alloc_error_handler(size: usize, align: usize) -> !;
    }
    #[link(name = "c")]
    unsafe extern "C" {
        fn printf(format: *const u8, ...) -> i32;
        fn exit(status: i32) -> !;
    }
    pub unsafe fn alloc(size: usize, align: usize) -> *mut u8 {
        unsafe {
            __rust_no_alloc_shim_is_unstable_v2();
            __rust_alloc(size, Alignment::new_unchecked(align))
        }
    }
    pub unsafe fn dealloc(ptr: *mut u8, size: usize, align: usize) {
        unsafe { __rust_dealloc(ptr, size, Alignment::new_unchecked(align)) }
    }
    pub unsafe fn realloc(ptr: *mut u8, size: usize, align: usize, new_size: usize) -> *mut u8 {
        unsafe { __rust_realloc(ptr, size, Alignment::new_unchecked(align), new_size) }
    }
    pub unsafe fn alloc_zeroed(size: usize, align: usize) -> *mut u8 {
        unsafe {
            __rust_no_alloc_shim_is_unstable_v2();
            __rust_alloc_zeroed(size, Alignment::new_unchecked(align))
        }
    }
    pub fn handle_alloc_error(size: usize, align: usize) -> ! {
        unsafe { __rust_alloc_error_handler(size, align) }
    }
    #[rustc_std_internal_symbol]
    pub unsafe fn __rdl_alloc_error_handler(size: usize, _align: usize) -> ! {
        unsafe {
            printf(b"default oom %zu\n\0" as *const [u8; 17] as *const u8, size);
            exit(3)
        }
    }
}
"#;
const HEAP: &str = r#"pub mod heap {
    use crate::alloc::{GlobalAlloc, Layout};
    use crate::macros::global_allocator;
    #[link(name = "c")]
    unsafe extern "C" {
        fn malloc(size: usize) -> *mut u8;
        fn calloc(count: usize, size: usize) -> *mut u8;
        fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
        fn free(ptr: *mut u8);
    }
    pub struct Malloc;
    unsafe impl crate::Sync for Malloc {}
    pub static mut LAST_ALLOC: usize = 0;
    pub static mut LAST_FREE: usize = 0;
    pub static mut LAST_ZEROED: usize = 0;
    pub static mut LAST_GROWN: usize = 0;
    unsafe impl GlobalAlloc for Malloc {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            unsafe {
                LAST_ALLOC = layout.size();
                malloc(layout.size())
            }
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe {
                LAST_FREE = layout.size();
                free(ptr)
            }
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            unsafe {
                LAST_ZEROED = layout.align();
                calloc(1, layout.size())
            }
        }
        unsafe fn realloc(&self, ptr: *mut u8, _layout: Layout, new_size: usize) -> *mut u8 {
            unsafe {
                LAST_GROWN = new_size;
                realloc(ptr, new_size)
            }
        }
    }
    #[global_allocator]
    static HEAP: Malloc = Malloc;
}
"#;
const HANDLER: &str = r#"pub mod handler {
    use crate::alloc::Layout;
    use crate::macros::alloc_error_handler;
    #[link(name = "c")]
    unsafe extern "C" {
        fn printf(format: *const u8, ...) -> i32;
        fn exit(status: i32) -> !;
    }
    #[alloc_error_handler]
    fn oom(layout: Layout) -> ! {
        unsafe {
            printf(b"handler oom %zu %zu\n\0" as *const [u8; 21] as *const u8, layout.size(), layout.align());
            exit(4)
        }
    }
}
"#;
const SECOND: &str = r"pub mod second {
    use crate::macros::global_allocator;
    #[global_allocator]
    static OTHER: crate::heap::Malloc = crate::heap::Malloc;
}
";
const NOT_STATIC: &str = r"pub mod not_static {
    use crate::macros::global_allocator;
    #[global_allocator]
    fn not_static() {}
}
";
const SECOND_HANDLER: &str = r"pub mod second_handler {
    use crate::macros::alloc_error_handler;
    #[alloc_error_handler]
    fn again(_layout: crate::alloc::Layout) -> ! {
        loop {}
    }
}
";
const HANDLER_STATIC: &str = r"pub mod handler {
    use crate::macros::alloc_error_handler;
    #[alloc_error_handler]
    static NOT_A_FUNCTION: usize = 0;
}
";
const HANDLER_UNIT: &str = r"pub mod handler {
    use crate::macros::alloc_error_handler;
    #[alloc_error_handler]
    fn returns(_layout: crate::alloc::Layout) {}
}
";
const GATE: &str = "#![feature(alloc_error_handler)]\n";

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

/// rustc's four allocator functions, as `nm` shows them in its objects.
const GLOBAL_FUNCTIONS: [&str; 4] = [
    "_RNvCs6rREvFdRhLb_7___rustc12___rust_alloc",
    "_RNvCs6rREvFdRhLb_7___rustc14___rust_dealloc",
    "_RNvCs6rREvFdRhLb_7___rustc14___rust_realloc",
    "_RNvCs6rREvFdRhLb_7___rustc19___rust_alloc_zeroed",
];
const HANDLER_FUNCTION: &str = "_RNvCs6rREvFdRhLb_7___rustc26___rust_alloc_error_handler";
const DEFAULT_HANDLER: &str = "_RNvCs6rREvFdRhLb_7___rustc25___rdl_alloc_error_handler";
/// The heap with its `#[global_allocator]` on another static type.
fn plain_heap() -> String {
    HEAP.replace(
        "    #[global_allocator]\n    static HEAP: Malloc = Malloc;\n",
        "    pub struct Plain;\n    unsafe impl crate::Sync for Plain {}\n    #[global_allocator]\n    static PLAIN: Plain = Plain;\n",
    )
}
/// The failure of a crate that must not compile, or `accepted`.
fn rejected(source: &str) -> String {
    lowered(source).map_or_else(|error| format!("{error:?}"), |_| "accepted".to_owned())
}

/// rustc (`global-lib`, a library without `#![needs_allocator]`): accepted;
/// defines the four functions and alloc's `__rdl_alloc_error_handler`.
/// (`handler-lib`) with `#[alloc_error_handler]`: also defines
/// `__rust_alloc_error_handler`.
#[test]
fn allocator_attributes_generate_rustcs_internal_functions() -> Result {
    let text = lowered(&format!("{CORE}{ALLOC}{HEAP}"))?;
    for symbol in GLOBAL_FUNCTIONS.into_iter().chain([DEFAULT_HANDLER]) {
        assert!(has_symbol(&text, symbol, false), "{symbol}\n{text}");
    }
    assert!(!text.contains(HANDLER_FUNCTION), "{text}");
    let text = lowered(&format!("{GATE}{CORE}{ALLOC}{HEAP}{HANDLER}"))?;
    for symbol in GLOBAL_FUNCTIONS.into_iter().chain([HANDLER_FUNCTION]) {
        assert!(has_symbol(&text, symbol, false), "{symbol}\n{text}");
    }
    Ok(())
}

/// rustc rejects each library form: `two-lib` "cannot define multiple global
/// allocators", `two-handlers-lib` "cannot define multiple allocation error
/// handlers", `non-static-lib` "allocators must be statics",
/// `handler-non-fn-lib` "`alloc_error_handler` must be a function",
/// `global-form-lib` "malformed `global_allocator` attribute input",
/// `not-globalalloc-lib` E0277 (`Plain: GlobalAlloc` is not satisfied) and
/// `handler-signature-lib` E0308 (mismatched types). This compiler reports the last
/// two as its unsatisfied obligation and type mismatch inside the expansion.
#[test]
fn misplaced_or_repeated_allocator_attributes_are_rejected() {
    let cases = [
        (
            format!("{CORE}{ALLOC}{HEAP}{SECOND}"),
            "cannot define multiple global allocators",
        ),
        (
            format!("{GATE}{CORE}{ALLOC}{HEAP}{HANDLER}{SECOND_HANDLER}"),
            "cannot define multiple allocation error handlers",
        ),
        (
            format!("{CORE}{ALLOC}{HEAP}{NOT_STATIC}"),
            "allocators must be statics",
        ),
        (
            format!("{GATE}{CORE}{ALLOC}{HEAP}{HANDLER_STATIC}"),
            "alloc_error_handler must be a function",
        ),
        (
            format!(
                "{CORE}{ALLOC}{}",
                HEAP.replace("    #[global_allocator]\n", "    #[global_allocator(x)]\n")
            ),
            "malformed `global_allocator` attribute input",
        ),
        (
            format!("{CORE}{ALLOC}{}", plain_heap()),
            "trait obligation requires an unambiguous matching assumption or implementation\", expansions: [ExpansionTrace { macro_name: \"global_allocator\"",
        ),
        (
            format!("{GATE}{CORE}{ALLOC}{HEAP}{HANDLER_UNIT}"),
            "incompatible types\", expansions: [ExpansionTrace { macro_name: \"alloc_error_handler\"",
        ),
    ];
    let mut failures = Vec::new();
    for (source, expected) in &cases {
        let error = rejected(source);
        if !error.contains(expected) {
            failures.push(format!("{expected}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
