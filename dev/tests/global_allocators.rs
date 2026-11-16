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
const MAIN: &str = r#"pub mod program {
    use crate::{heap, rt_alloc};
    #[link(name = "c")]
    unsafe extern "C" {
        fn printf(format: *const u8, ...) -> i32;
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        unsafe {
            let p = rt_alloc::alloc(24, 8);
            *p = 7;
            let q = rt_alloc::realloc(p, 24, 8, 48);
            let z = rt_alloc::alloc_zeroed(16, 16);
            printf(
                b"%d %d %zu %zu %zu\n\0" as *const [u8; 19] as *const u8,
                *q as i32,
                *z as i32,
                heap::LAST_ALLOC,
                heap::LAST_GROWN,
                heap::LAST_ZEROED,
            );
            rt_alloc::dealloc(q, 48, 8);
            printf(b"%zu\n\0" as *const [u8; 5] as *const u8, heap::LAST_FREE);
            rt_alloc::dealloc(z, 16, 16);
            printf(b"%zu\n\0" as *const [u8; 5] as *const u8, heap::LAST_FREE);
        }
        0
    }
}
"#;
const OOM: &str = r#"pub mod program {
    use crate::rt_alloc;
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        rt_alloc::handle_alloc_error(99, 8)
    }
}
"#;
const DEFAULT: &str = r#"pub mod default_lib {
    #[link(name = "c")]
    unsafe extern "C" {
        fn malloc(size: usize) -> *mut u8;
        fn calloc(count: usize, size: usize) -> *mut u8;
        fn realloc(ptr: *mut u8, size: usize) -> *mut u8;
        fn free(ptr: *mut u8);
        fn printf(format: *const u8, ...) -> i32;
    }
    #[rustc_std_internal_symbol]
    pub unsafe extern "C" fn __rdl_alloc(size: usize, _align: usize) -> *mut u8 {
        unsafe {
            printf(b"rdl alloc %zu\n\0" as *const [u8; 15] as *const u8, size);
            malloc(size)
        }
    }
    #[rustc_std_internal_symbol]
    pub unsafe extern "C" fn __rdl_dealloc(ptr: *mut u8, size: usize, _align: usize) {
        unsafe {
            printf(b"rdl dealloc %zu\n\0" as *const [u8; 17] as *const u8, size);
            free(ptr)
        }
    }
    #[rustc_std_internal_symbol]
    pub unsafe extern "C" fn __rdl_realloc(ptr: *mut u8, _old_size: usize, _align: usize, new_size: usize) -> *mut u8 {
        unsafe {
            printf(b"rdl realloc %zu\n\0" as *const [u8; 17] as *const u8, new_size);
            realloc(ptr, new_size)
        }
    }
    #[rustc_std_internal_symbol]
    pub unsafe extern "C" fn __rdl_alloc_zeroed(size: usize, _align: usize) -> *mut u8 {
        unsafe {
            printf(b"rdl zeroed %zu\n\0" as *const [u8; 16] as *const u8, size);
            calloc(1, size)
        }
    }
}
"#;
const PLAIN: &str = r#"pub mod program {
    use crate::rt_alloc;
    #[link(name = "c")]
    unsafe extern "C" {
        fn printf(format: *const u8, ...) -> i32;
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn main() -> i32 {
        unsafe {
            let p = rt_alloc::alloc(24, 8);
            *p = 7;
            let q = rt_alloc::realloc(p, 24, 8, 48);
            let z = rt_alloc::alloc_zeroed(16, 16);
            printf(b"%d %d\n\0" as *const [u8; 7] as *const u8, *q as i32, *z as i32);
            rt_alloc::dealloc(q, 48, 8);
            rt_alloc::dealloc(z, 16, 16);
        }
        0
    }
}
"#;
const HEADER: &str = "#![needs_allocator]\n";

/// The `LowIR` text of one crate lowered from its exports, without an entry.
fn lowered(source: &str) -> Result<String> {
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program(source)?, &mut text)?;
    Ok(String::from_utf8(text)?)
}
/// One crate lowered from its exports, without an entry, as an executable.
fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
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
    Ok(program)
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

/// rustc rejects each library form and each program form (the library
/// with `#![needs_allocator]` and a C `main`, built as a staticlib):
/// `two`/`two-lib` "cannot define multiple global allocators",
/// `two-handlers`/`two-handlers-lib` "cannot define multiple allocation error
/// handlers", `non-static`/`non-static-lib` "allocators must be statics",
/// `handler-non-fn`/`handler-non-fn-lib` "`alloc_error_handler` must be a
/// function", `global-form`/`global-form-lib` "malformed `global_allocator`
/// attribute input", `not-globalalloc`/`not-globalalloc-lib` E0277
/// (`Plain: GlobalAlloc` is not satisfied) and
/// `handler-signature`/`handler-signature-lib` E0308 (mismatched types). This
/// compiler reports the last two as its unsatisfied obligation and type
/// mismatch inside the expansion.
#[test]
fn misplaced_or_repeated_allocator_attributes_are_rejected() {
    let form = HEAP.replace("    #[global_allocator]\n", "    #[global_allocator(x)]\n");
    let cases = [
        (
            "",
            format!("{HEAP}{SECOND}"),
            MAIN,
            "cannot define multiple global allocators",
        ),
        (
            GATE,
            format!("{HEAP}{HANDLER}{SECOND_HANDLER}"),
            OOM,
            "cannot define multiple allocation error handlers",
        ),
        (
            "",
            format!("{HEAP}{NOT_STATIC}"),
            MAIN,
            "allocators must be statics",
        ),
        (
            GATE,
            format!("{HEAP}{HANDLER_STATIC}"),
            OOM,
            "alloc_error_handler must be a function",
        ),
        (
            "",
            form,
            MAIN,
            "malformed `global_allocator` attribute input",
        ),
        (
            "",
            plain_heap(),
            PLAIN,
            "trait obligation requires an unambiguous matching assumption or implementation\", expansions: [ExpansionTrace { macro_name: \"global_allocator\"",
        ),
        (
            GATE,
            format!("{HEAP}{HANDLER_UNIT}"),
            OOM,
            "incompatible types\", expansions: [ExpansionTrace { macro_name: \"alloc_error_handler\"",
        ),
    ];
    let mut failures = Vec::new();
    for (gate, body, tail, expected) in &cases {
        let library = format!("{gate}{CORE}{ALLOC}{body}");
        let program = format!("{gate}{HEADER}{CORE}{ALLOC}{body}{tail}");
        for source in [library, program] {
            let error = rejected(&source);
            if !error.contains(expected) {
                failures.push(format!("{expected}: {error}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// The shim's own functions, as `nm` shows them in rustc's objects.
const SHIM_MARK: &str = "_RNvCs6rREvFdRhLb_7___rustc35___rust_no_alloc_shim_is_unstable_v2";
const DEFAULT_FUNCTIONS: [&str; 4] = [
    "_RNvCs6rREvFdRhLb_7___rustc11___rdl_alloc",
    "_RNvCs6rREvFdRhLb_7___rustc13___rdl_dealloc",
    "_RNvCs6rREvFdRhLb_7___rustc13___rdl_realloc",
    "_RNvCs6rREvFdRhLb_7___rustc18___rdl_alloc_zeroed",
];
/// rustc's error for a program that needs an allocator and has none.
const MISSING: &str = "no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait";
/// The single-crate programs, as rustc built them (staticlibs that export
/// a C `main`), with rustc's exit status and output.
fn single_programs() -> [(&'static str, String, i32, &'static str); 4] {
    [
        (
            "single-global",
            format!("{HEADER}{CORE}{ALLOC}{HEAP}{MAIN}"),
            0,
            "7 0 24 48 16\n48\n16\n",
        ),
        (
            "single-oom",
            format!("{HEADER}{CORE}{ALLOC}{HEAP}{OOM}"),
            3,
            "default oom 99\n",
        ),
        (
            "single-handler",
            format!("{GATE}{HEADER}{CORE}{ALLOC}{HEAP}{HANDLER}{OOM}"),
            4,
            "handler oom 99 8\n",
        ),
        (
            "single-default",
            format!("{HEADER}#![default_lib_allocator]\n{CORE}{ALLOC}{DEFAULT}{PLAIN}"),
            0,
            "rdl alloc 24\nrdl realloc 48\nrdl zeroed 16\n7 0\nrdl dealloc 48\nrdl dealloc 16\n",
        ),
    ]
}

/// rustc (`single-global`): the program defines the expansion's four
/// functions, alloc's `__rdl_alloc_error_handler`, and the shim's
/// `__rust_alloc_error_handler` and `__rust_no_alloc_shim_is_unstable_v2`.
/// (`single-handler`) the handler is the expansion's. (`single-default`,
/// `#![default_lib_allocator]` without a global allocator) the shim defines
/// the four functions over the `__rdl_*` ones. (`missing`) rejected with
/// rustc's error; (`guest-crates-ungated`) an ungated `#![needs_allocator]`
/// is E0658.
#[test]
fn a_program_that_needs_an_allocator_gets_rustcs_shim() -> Result {
    let [global, _, handler, default] = single_programs();
    let defines = |source: &str, symbols: &[&str]| -> Result {
        let text = lowered(source)?;
        for symbol in symbols {
            assert!(has_symbol(&text, symbol, false), "{symbol}\n{text}");
        }
        Ok(())
    };
    let shim = [DEFAULT_HANDLER, HANDLER_FUNCTION, SHIM_MARK];
    defines(&global.1, &[GLOBAL_FUNCTIONS.as_slice(), &shim].concat())?;
    defines(&handler.1, &[HANDLER_FUNCTION, SHIM_MARK])?;
    let forwarded = [GLOBAL_FUNCTIONS, DEFAULT_FUNCTIONS].concat();
    defines(&default.1, &[forwarded.as_slice(), &shim].concat())?;
    let missing = rejected(&format!("{HEADER}{CORE}{ALLOC}{PLAIN}"));
    assert!(missing.contains(MISSING), "{missing}");
    let ungated = CORE.replace("#![feature(allocator_internals)]\n", "");
    let ungated = rejected(&format!("{HEADER}{ungated}{ALLOC}{PLAIN}"));
    let gate = "`#![needs_allocator]` requires `#![feature(allocator_internals)]`";
    assert!(ungated.contains(gate), "{ungated}");
    Ok(())
}

/// rustc's runs of the single-crate programs (`run` in its results).
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn single_crate_programs_allocate_through_the_shim_as_rustcs_do() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/global-allocators/single-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (name, source, status, expected) in single_programs() {
        let built = program(&source)?;
        crate::ffi_program::check_status(&root, &folder, name, &built, CLIENT, expected, status)?;
    }
    Ok(())
}
/// The programs export C `main`; the client only completes the link.
const CLIENT: &str = "/* The Rust program exports `main`. */\n";

/// The three-crate programs: the oracle's core, alloc and app sources.
fn guest_sources(heap_in_alloc: bool, app: &str) -> [String; 3] {
    let core = CORE.replace("#![feature(allocator_internals)]\n", "");
    let mut alloc = format!(
        "#![feature(no_core, rustc_attrs, allocator_internals)]\n#![no_core]\n#![needs_allocator]\n#![allow(internal_features, dead_code, unused_unsafe)]\nextern crate core;\n{}",
        ALLOC.replace("crate::mem::", "core::mem::")
    );
    let heap = HEAP
        .replace("crate::alloc::", "core::alloc::")
        .replace("crate::Sync", "core::Sync")
        .replace("crate::macros::", "core::macros::");
    if heap_in_alloc {
        alloc.push_str(&heap);
    }
    let app = format!(
        "#![feature(no_core)]\n#![no_core]\n#![no_main]\n#![allow(dead_code, unused_unsafe)]\nextern crate core;\nextern crate alloc;\n{}{}",
        if app == PLAIN { "" } else { heap.as_str() },
        app.replace(
            "use crate::{heap, rt_alloc};",
            "use crate::heap;\n    use alloc::rt_alloc;"
        )
        .replace("use crate::rt_alloc;", "use alloc::rt_alloc;")
    );
    [core, alloc, app]
}
/// Compile core with alloc and the app as bring-up guests, writing the
/// session's `LowIR` beside its sources.
fn guest_session(
    name: &str,
    sources: &[String; 3],
) -> Result<(std::path::PathBuf, std::process::Output)> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/global-allocators")
        .join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    for (file, text) in ["core.rs", "alloc.rs", "app.rs", "skip.tsv"]
        .iter()
        .zip(sources.iter().map(String::as_str).chain([""]))
    {
        std::fs::write(folder.join(file), text)?;
    }
    let identity = "allocators_bringup";
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args(["--emit-lowir", "--cap-lints=allow", "--edition=2024"])
        .args(["--crate-name", "core", "--crate-disambiguator", identity])
        .args(["--source-core", &format!("core@{identity}")])
        .arg("--bringup-skip-list")
        .arg(folder.join("skip.tsv"))
        .arg("--native-links-output")
        .arg(folder.join("session.native-links.json"))
        .arg("--bringup-guest")
        .arg(format!("alloc={}", folder.join("alloc.rs").display()))
        .arg("--bringup-guest")
        .arg(format!("app={}", folder.join("app.rs").display()))
        .arg("-o")
        .arg(folder.join("session.lowir"))
        .arg(folder.join("core.rs"))
        .output()?;
    Ok((folder, output))
}

/// rustc (`guest-global`): accepted, the app's object holds the expansion's
/// four functions and the shim. (`guest-missing`, the app without
/// `#[global_allocator]`) the no-global-allocator error. (`guest-conflict`,
/// alloc also has one) "the `#[global_allocator]` in this crate conflicts
/// with global allocator in: alloc".
#[test]
fn the_program_crate_of_a_guest_session_gets_the_shim() -> Result {
    let (folder, output) = guest_session("global", &guest_sources(false, MAIN))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "{stderr}");
    let text = std::fs::read_to_string(folder.join("session.lowir"))?;
    for symbol in GLOBAL_FUNCTIONS
        .into_iter()
        .chain([HANDLER_FUNCTION, SHIM_MARK])
    {
        assert!(has_symbol(&text, symbol, false), "{symbol}\n{text}");
    }
    let conflict =
        "the `#[global_allocator]` in this crate conflicts with global allocator in: alloc";
    for (name, sources, message) in [
        ("missing", guest_sources(false, PLAIN), MISSING),
        ("conflict", guest_sources(true, MAIN), conflict),
    ] {
        let (_, output) = guest_session(name, &sources)?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(1), "{name}: {stderr}");
        assert!(stderr.contains(message), "{name}: {stderr}");
    }
    Ok(())
}

/// rustc's runs of `guest-global` and `guest-oom`.
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn guest_programs_allocate_through_the_shim_as_rustcs_do() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (name, app, status, expected) in [
        ("guest-global", MAIN, 0, "7 0 24 48 16\n48\n16\n"),
        ("guest-oom", OOM, 3, "default oom 99\n"),
    ] {
        let (folder, output) = guest_session(name, &guest_sources(false, app))?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = std::fs::read(folder.join("session.lowir"))?;
        crate::ffi_program::check_lowir(&root, &folder, name, &text, CLIENT, expected, status)?;
    }
    Ok(())
}
