//! A constructor reads each field by value. A field whose type is an
//! associated type that the body proves Copy, such as the discriminant type
//! behind core's `mem::Discriminant<T>`, is copied: its parameter stays
//! initialized, and a Copy value has nothing to clean up. The discriminant
//! read behind it also accepts an enum that names a lifetime of its caller,
//! such as `Option<&T>`: under the intrinsic's own input lifetime that
//! lifetime sits one binder deeper than in the instance's argument.
use super::{
    implemented_copies::run,
    memberwise_clones::{independent, program, rustc},
};
use std::path::{Path, PathBuf};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Every case follows this source core.
const PREFIX: &str = "pa31/tests/spec/source-copy-repeats/const-length.rs";
/// The discriminant protocol, `Wrapped` as core declares `mem::Discriminant`,
/// and `Loud`, which reports its field when dropped.
const SHARED: &str = "unsafe extern \"C\"{fn note(v:u8);}\n\
#[lang=\"drop\"]#[rustc_const_unstable(feature=\"const_destruct\",issue=\"133214\")]pub const trait Drop{fn drop(&mut self);}\n\
#[lang=\"discriminant_kind\"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]\n\
pub trait DiscriminantKind{#[lang=\"discriminant_type\"]type Discriminant:Copy;}\n\
#[unstable(feature=\"core_intrinsics\",issue=\"none\")]#[rustc_intrinsic_const_stable_indirect]#[rustc_nounwind]#[rustc_intrinsic]\n\
pub const fn discriminant_value<T>(v:&T)-><T as DiscriminantKind>::Discriminant;\n\
#[repr(i8)]pub enum Signed{Low=0xffu8 as i8,Zero=0,High=1}\n\
#[repr(u16)]pub enum Tagged{Empty=27,Value(u8)=300}\n\
pub enum Maybe<'a>{Empty,Value(&'a u8)}\n\
pub struct Data{value:u8}\n\
pub struct Loud(u8);impl Drop for Loud{fn drop(&mut self){unsafe{note(self.0)}}}\n\
pub struct Wrapped<T>(<T as DiscriminantKind>::Discriminant);\n\
pub fn wrap<T>(v:&T)->Wrapped<T>{Wrapped(discriminant_value(v))}\n\
pub struct Pair<T>(T,<T as DiscriminantKind>::Discriminant);\n\
pub trait Kind{type Out:Copy;}impl Kind for u8{type Out=u16;}\n\
pub struct Held<T:Kind>(T::Out);\n\
pub trait Owner{type Out;}impl Owner for u8{type Out=u16;}\n\
pub struct Kept<T:Owner>(T::Out);\n\
fn main(){let _=entry();}\n";
const ENTRY: &str = "#[unsafe(no_mangle)]pub extern \"C\" fn entry()->i32";
const OBSERVER: &str = "#include <stdio.h>\nvoid note(unsigned char v){printf(\"%u \",(unsigned)v);}\nextern int entry(void);\nint main(void){printf(\"= %d\\n\",entry());return 0;}\n";

/// Items, the body of `entry`, and the observer's output: each `note` in
/// call order, then the value `entry` returns.
const ACCEPTED: &[(&str, &str, &str, &str)] = &[
    ("signed-repr", "", "wrap(&Signed::Low).0 as i32", "= -1\n"),
    (
        "payload-tag",
        "",
        "wrap(&Tagged::Value(7)).0 as i32",
        "= 300\n",
    ),
    (
        "pointer-niche-empty",
        "",
        "wrap(&Maybe::<'static>::Empty).0 as i32",
        "= 0\n",
    ),
    (
        "pointer-niche-value",
        "",
        "let value=7u8;wrap(&Maybe::Value(&value)).0 as i32",
        "= 1\n",
    ),
    (
        "scoped-niche-parameter",
        "fn of<'a>(v:&'a u8)->i32{wrap(&Maybe::Value(v)).0 as i32}\n",
        "let value=7u8;of(&value)",
        "= 1\n",
    ),
    ("non-enum", "", "wrap(&Data{value:19}).0 as i32", "= 0\n"),
    (
        "direct-constructor",
        "",
        "Wrapped::<Tagged>(discriminant_value(&Tagged::Empty)).0 as i32",
        "= 27\n",
    ),
    (
        "constructor-pointer",
        "",
        "let make:fn(u16)->Wrapped<Tagged>=Wrapped;make(300).0 as i32",
        "= 300\n",
    ),
    (
        "copied-twice",
        "fn both<T>(v:&T)->(Wrapped<T>,Wrapped<T>){let d=discriminant_value(v);(Wrapped(d),Wrapped(d))}\n",
        "let p=both(&Signed::High);unsafe{note((p.0).0 as u8)};(p.1).0 as i32",
        "1 = 1\n",
    ),
    (
        "constant",
        "const TAG:Wrapped<Tagged>=Wrapped(discriminant_value(&Tagged::Value(7)));\n",
        "TAG.0 as i32",
        "= 300\n",
    ),
    (
        "dropped-sibling",
        "fn pair<T>(v:T)->Pair<T>{let d=discriminant_value(&v);Pair(v,d)}\n",
        "let p=pair(Loud(4));unsafe{note(9)};p.1 as i32",
        "9 4 = 0\n",
    ),
    (
        "declared-copy-projection",
        "fn hold<T:Kind>(v:T::Out)->Held<T>{Held(v)}\n",
        "hold::<u8>(9).0 as i32",
        "= 9\n",
    ),
    (
        "moved-projection",
        "fn keep<T:Owner>(v:T::Out)->Kept<T>{Kept(v)}\n",
        "keep::<u8>(8).0 as i32",
        "= 8\n",
    ),
];
/// Items, rustc's error code and the rejection: a field that is not proved
/// Copy is still moved into its constructor.
const REJECTED: &[(&str, &str, &str, &str)] = &[
    (
        "reject-moved-projection",
        "fn twice<T:Owner>(v:T::Out)->(Kept<T>,Kept<T>){(Kept(v),Kept(v))}\n",
        "E0382",
        MOVED,
    ),
    (
        "reject-moved-sibling",
        "fn pairs<T>(v:T)->(Pair<T>,Pair<T>){let d=discriminant_value(&v);(Pair(v,d),Pair(v,d))}\n",
        "E0382",
        MOVED,
    ),
];
const MOVED: &str = "local may be uninitialized or moved at this read";

fn evidence(kind: &str) -> Result<PathBuf> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/copied-fields/{kind}/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

fn source(items: &str, body: &str) -> Result<String> {
    let mut text = crate::resolve::fixture(PREFIX)?;
    text.push('\n');
    text.push_str(SHARED);
    text.push_str(items);
    text.push_str(ENTRY);
    text.push('{');
    text.push_str(body);
    text.push_str("}\n");
    Ok(text)
}

#[test]
fn copied_constructor_fields_type_and_lower_as_rustc_accepts() -> Result {
    let folder = evidence("source")?;
    let mut failures = Vec::new();
    for &(name, items, body, _) in ACCEPTED {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source(items, body)?)?;
        let oracle = rustc(&path)?;
        if !oracle.status.success() {
            failures.push(format!(
                "{name} rustc: {}",
                String::from_utf8_lossy(&oracle.stderr)
            ));
        }
        for mode in ["--emit-types", "--emit-lowir"] {
            let output = run(mode, &path.to_string_lossy())?;
            if !output.status.success() {
                failures.push(format!(
                    "{name} {mode}: {:?} {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn constructor_fields_without_a_copy_proof_are_still_moved() -> Result {
    let folder = evidence("rejected")?;
    let mut failures = Vec::new();
    for &(name, items, code, reason) in REJECTED {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source(items, "0")?)?;
        let oracle = rustc(&path)?;
        let stderr = String::from_utf8_lossy(&oracle.stderr);
        if oracle.status.code() != Some(1) || !stderr.contains(code) {
            failures.push(format!("{name} rustc: {:?} {stderr}", oracle.status.code()));
        }
        let output = run("--emit-lowir", &path.to_string_lossy())?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() != Some(1) || !stderr.contains(reason) {
            failures.push(format!("{name}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// rustc and the native backend print the same discriminants and drops.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn copied_constructor_fields_run_as_rustc_runs() -> Result {
    let folder = evidence("native")?;
    let observer = folder.join("observer.c");
    std::fs::write(&observer, OBSERVER)?;
    for &(name, items, body, expected) in ACCEPTED {
        let text = source(items, body)?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &text)?;
        let printed = independent(&path, &observer)?;
        assert_eq!(
            String::from_utf8_lossy(&printed),
            expected,
            "{name}: rustc differs from the declared output"
        );
        let program = program(&text).map_err(|error| format!("{name}: {error}"))?;
        crate::observed_program::execute(
            &folder.join(name),
            &program,
            &observer,
            expected.as_bytes(),
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
