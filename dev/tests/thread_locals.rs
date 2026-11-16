//! `#[thread_local]` statics as std 1.98's `thread_local!` expands them, over a
//! mini core: one instance per thread, lowered to the backend's
//! `[storage=thread_local]` globals, under rustc's rules for their use.
//!
//! Every outcome was declared before either compiler ran, with rustc 1.98.0
//! (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 -Cpanic=abort
//! -Copt-level=0 --cap-lints=allow`, `PROGRAM` as a staticlib linked by `cc`
//! with `CLIENT`) as the oracle. The core first lacked `UnsafeCell`'s
//! `!Freeze` and `!Sync`, and rustc rejected the program (E0080); with them
//! every case is as declared.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// The mini core, which requests `thread_local`; `Slot` is not `Sync`.
const CORE: &str = r#"#![feature(no_core, lang_items, rustc_attrs, auto_traits, const_trait_impl, negative_impls, freeze_impls, thread_local)]
#![no_core]
#![allow(internal_features, dead_code, unused_unsafe, static_mut_refs)]
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
#[lang = "sync"]
pub unsafe auto trait Sync {}
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
#[lang = "destruct"]
#[rustc_deny_explicit_impl]
pub const trait Destruct: PointeeSized {}
#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
}
impl Clone for u64 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u64 {}
#[lang = "unsafe_cell"]
#[repr(transparent)]
pub struct UnsafeCell<T> {
    value: T,
}
impl<T> !Freeze for UnsafeCell<T> {}
impl<T> !Sync for UnsafeCell<T> {}
impl<T> UnsafeCell<T> {
    pub const fn new(value: T) -> Self {
        UnsafeCell { value }
    }
    pub const fn get(&self) -> *mut T {
        self as *const UnsafeCell<T> as *const T as *mut T
    }
}
/// Not `Sync`: an interior-mutable cell, as std's `Cell` is.
pub struct Slot(pub UnsafeCell<u64>);
"#;

/// Two thread-locals of a type that is not `Sync`, a mutable one and a
/// shared `static mut`, with the addresses a C client compares across
/// threads.
const PROGRAM: &str = r#"pub mod program {
    use crate::{Slot, UnsafeCell};
    #[thread_local]
    static mut COUNT: u64 = 7;
    #[thread_local]
    static SLOT: Slot = Slot(UnsafeCell::new(40));
    #[thread_local]
    static ZEROED: [u64; 4] = [0; 4];
    static mut SHARED: u64 = 0;
    #[unsafe(no_mangle)]
    pub extern "C" fn count() -> u64 {
        unsafe { COUNT }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn set_count(value: u64) {
        unsafe {
            COUNT = value;
            SHARED = value;
        }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn shared() -> u64 {
        unsafe { SHARED }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn slot() -> u64 {
        unsafe { *SLOT.0.get() }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn set_slot(value: u64) {
        unsafe { *SLOT.0.get() = value }
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn slot_address() -> *const Slot {
        &SLOT as *const Slot
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn count_address() -> *const u64 {
        &raw const COUNT
    }
    #[unsafe(no_mangle)]
    pub extern "C" fn zeroed() -> u64 {
        let [_, _, _, last] = ZEROED;
        last
    }
}
"#;

/// A second thread reads the initializers and writes its own instances; the
/// shared static is one for both.
const CLIENT: &str = r#"#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
uint64_t count(void);
void set_count(uint64_t value);
uint64_t shared(void);
uint64_t slot(void);
uint64_t zeroed(void);
void set_slot(uint64_t value);
const void *slot_address(void);
const void *count_address(void);
static const void *their_count;
static const void *their_slot;
static void *worker(void *unused) {
    (void)unused;
    printf("worker %llu %llu\n", (unsigned long long)count(), (unsigned long long)slot());
    set_count(100);
    set_slot(41);
    printf("worker %llu %llu %llu\n", (unsigned long long)count(), (unsigned long long)slot(),
           (unsigned long long)shared());
    their_count = count_address();
    their_slot = slot_address();
    return 0;
}
int main(void) {
    printf("main %llu %llu\n", (unsigned long long)count(), (unsigned long long)slot());
    set_count(8);
    set_slot(9);
    pthread_t thread;
    if (pthread_create(&thread, 0, worker, 0) != 0 || pthread_join(thread, 0) != 0) {
        return 2;
    }
    printf("main %llu %llu %llu\n", (unsigned long long)count(), (unsigned long long)slot(),
           (unsigned long long)shared());
    printf("distinct %d %d\n", their_count != count_address(), their_slot != slot_address());
    printf("zeroed %llu\n", (unsigned long long)zeroed());
    fflush(stdout);
    return 0;
}
"#;

/// What rustc's build of `PROGRAM` with `CLIENT` prints.
const RUSTC: &str = "main 7 40
worker 7 40
worker 100 41 100
main 8 9 100
distinct 1 1
zeroed 0
";

/// One crate lowered from its exports.
fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    crate::ffi_program::exported(source, "thread_locals")
}

/// rustc accepts the program and places the three thread-locals in `.tdata`;
/// the shared static stays ordinary data.
#[test]
fn thread_local_statics_lower_to_thread_local_globals() -> Result {
    let mut text = Vec::new();
    rustgm::lowir::writer::write(&program(&format!("{CORE}{PROGRAM}"))?, &mut text)?;
    let text = String::from_utf8(text)?;
    assert_eq!(text.matches("[storage=thread_local]").count(), 3, "{text}");
    Ok(())
}

/// rustc rejects each: E0658 without the gate, "attribute cannot be used on"
/// a function and a constant, E0712 for a `'static` borrow, E0625 for a
/// constant, a static's initializer and a `const fn` that read one, E0277 for
/// the same cell in a shared static, E0565 for arguments and "not an unsafe
/// attribute"; this compiler names the same rules.
#[test]
fn uses_that_rustc_rejects_are_rejected() -> Result {
    let ungated = CORE.replacen(", thread_local)]", ")]", 1);
    let gate = "`#[thread_local]` is an experimental feature: it requires thread_local";
    let mut cases = vec![(
        format!("{ungated}#[thread_local]\nstatic mut COUNT: u64 = 0;\n"),
        gate,
    )];
    for (case, message) in [
        (
            "#[thread_local]
fn f() {}
",
            "`#[thread_local]` applies only to statics",
        ),
        (
            "#[thread_local]
const C: u64 = 0;
",
            "`#[thread_local]` applies only to statics",
        ),
        (
            "#[thread_local]
static VALUE: u64 = 1;
fn leak() -> &'static u64 {
    &VALUE
}
",
            "required lifetime does not outlive its target",
        ),
        (
            "#[thread_local]
static VALUE: u64 = 1;
const C: u64 = VALUE;
",
            "thread-local statics cannot be accessed at compile-time",
        ),
        (
            "#[thread_local]
static VALUE: u64 = 1;
static ADDRESS: &u64 = &VALUE;
",
            "thread-local statics cannot be accessed at compile-time",
        ),
        (
            "static SHARED: Slot = Slot(UnsafeCell::new(0));
",
            "shared static type must implement Sync",
        ),
        (
            "#[thread_local(fast)]
static VALUE: u64 = 1;
",
            "malformed `thread_local` attribute input: it is an outer word attribute",
        ),
        (
            "#[thread_local]
static VALUE: u64 = 1;
const fn read() -> u64 {
    VALUE
}
",
            "thread-local statics cannot be accessed at compile-time",
        ),
        (
            "#[unsafe(thread_local)]
static VALUE: u64 = 1;
",
            "`thread_local` is not an unsafe attribute",
        ),
    ] {
        cases.push((format!("{CORE}{case}"), message));
    }
    for (source, message) in cases {
        match program(&source) {
            Ok(_) => return Err(format!("accepted, not {message}").into()),
            Err(error) => assert!(error.to_string().contains(message), "{message}: {error}"),
        }
    }
    Ok(())
}

/// rustc accepts a borrow that ends in its function and a foreign
/// thread-local that is never read.
#[test]
fn uses_that_rustc_accepts_are_accepted() -> Result {
    for case in [
        "#[thread_local]
static VALUE: u64 = 1;
fn read() -> u64 {
    let r: &u64 = &VALUE;
    *r
}
",
        "unsafe extern \"C\" {
    #[thread_local]
    static errno: i32;
}
",
    ] {
        program(&format!("{CORE}{case}"))?;
    }
    Ok(())
}

/// rustc's run of `PROGRAM` with `CLIENT`.
#[test]
#[ignore = "requires the pinned CPPGM backend and a C compiler"]
fn each_thread_sees_its_own_instances_as_rustcs_program_does() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/thread-locals/run-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let built = program(&format!("{CORE}{PROGRAM}"))?;
    crate::ffi_program::check_status(&root, &folder, "program", &built, CLIENT, RUSTC, 0)?;
    Ok(())
}
