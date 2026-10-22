//! Layout kernels execute only under actual source declarations and checked callers.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]pub trait PointeeSized{}#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}#[lang=\"sized\"]pub trait Sized:MetaSized{}";
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn resolve(tail: &str) -> Result<Report> {
    let identity = CrateIdentity::new("minicore", "provider_v1")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let source = SOURCE.to_owned() + tail;
    let report = resolve::expansion::analyze_with_context(
        InputFile {
            identity: "provider.rs".into(),
            path: "provider.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    Ok(resolve::analyze_expanded(report)?)
}
fn inspect(tail: &str) -> Result<Declarations> {
    let report = resolve(tail)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    Ok(analysis.facts)
}
#[test]
fn real_primitive_layout_calls_execute_with_their_own_const_contracts() -> Result {
    for (tail, expected) in [
        (
            "#[rustc_intrinsic]const fn size_of<T>()->usize;const N:usize=size_of::<u32>();type A=[u8;N];",
            4,
        ),
        (
            "#[rustc_intrinsic]const fn align_of<T>()->usize;const N:usize=align_of::<u64>();type A=[u8;N];",
            8,
        ),
        (
            "#[rustc_intrinsic]const fn size_of<T>()->usize;const fn wrapper<T>()->usize{size_of::<T>()}const N:usize=wrapper::<u16>();type A=[u8;N];",
            2,
        ),
        (
            "#[rustc_intrinsic]const fn size_of<T>()->usize{1}const N:usize=size_of::<u32>();type A=[u8;N];",
            4,
        ),
        (
            "#[rustc_intrinsic]const fn size_of<T>()->usize;const A:usize=size_of::<u32>();const B:usize=size_of::<u32>();type X=[u8;A];type Y=[u8;B];",
            4,
        ),
    ] {
        let facts = inspect(tail).map_err(|e| format!("{tail}: {e}"))?;
        assert!(
            facts.array_lengths.values().any(|&n| n == expected),
            "{tail}"
        );
    }
    Ok(())
}
#[test]
fn source_layout_contracts_reject_malformed_or_unpaid_calls() {
    for tail in [
        "#[lang=\"mem_size_const\"]fn wrong(){}",
        "#[lang=\"mem_align_const\"]const WRONG:usize=0;",
        "#[rustc_intrinsic]const unsafe fn size_of<T>()->usize;",
        "#[rustc_intrinsic]const fn size_of<T>()->u32;",
        "#[rustc_intrinsic]const fn size_of<T:?Sized>()->usize;",
        "#[rustc_intrinsic]const fn size_of<T>(x:T)->usize;",
        "#[rustc_intrinsic]fn size_of<T>()->usize;const N:usize=size_of::<u32>();",
        "#[rustc_intrinsic]const fn size_of<T>()->usize;const P:fn()->usize=size_of::<u32>;",
        "#[rustc_intrinsic]const fn size_of<T>()->usize;fn f(){let _=size_of::<u32> as fn()->usize;}",
        "#[rustc_intrinsic]const fn size_of<T>()->usize{let x=0u8;let _r:&'static u8=&x;0}const N:usize=size_of::<u32>();type A=[u8;N];",
    ] {
        assert!(inspect(tail).is_err(), "{tail}");
    }
}

#[test]
fn selected_source_size_and_alignment_initializers_use_actual_trait_contracts() -> Result {
    let tail = "pub mod intrinsics{#[rustc_intrinsic]pub const fn size_of<T>()->usize;#[rustc_intrinsic]pub const fn align_of<T>()->usize;}pub trait SizedTypeProperties:Sized{#[lang=\"mem_size_const\"]const SIZE:usize=intrinsics::size_of::<Self>();#[lang=\"mem_align_const\"]const ALIGN:usize=intrinsics::align_of::<Self>();}impl<T>SizedTypeProperties for T{}const N:usize=<u32 as SizedTypeProperties>::SIZE;const M:usize=<u16 as SizedTypeProperties>::ALIGN;type A=[u8;N];type B=[u8;M];";
    let facts = inspect(tail)?;
    assert!(facts.array_lengths.values().any(|&n| n == 4));
    assert!(facts.array_lengths.values().any(|&n| n == 2));
    Ok(())
}

#[test]
fn layout_members_require_their_completed_associated_usize_contract() -> Result {
    let report =
        resolve("pub trait Properties:Sized{#[lang=\"mem_size_const\"]const SIZE:usize;}")?;
    let pending = Declarations::new(&report)?;
    let member = pending
        .source_bindings
        .as_deref()
        .and_then(|c| c.language(LangItem::MemSizeConst))
        .ok_or("member")?
        .definition;
    assert!(pending.definitions[member.0 as usize].is_none());
    assert!(validate_member(&report, &pending, member).is_err());
    let mut facts = crate::sema::declarations::analyze(&report)?;
    validate_member(&report, &facts, member)?;
    facts.definitions[member.0 as usize] = Some(TypeId::primitive(Primitive::U32));
    assert!(validate_member(&report, &facts, member).is_err());
    facts.definitions[member.0 as usize] = Some(TypeId::primitive(Primitive::Usize));
    let actual_owner = facts
        .associated
        .owner(member)
        .and_then(|id| facts.associated.get(id))
        .ok_or("owner")?
        .self_definition;
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .languages[LangItem::MemSizeConst.index()]
    .as_mut()
    .ok_or("claim")?
    .parent = Some(actual_owner);
    assert!(validate_member(&report, &facts, member).is_err());
    for tail in [
        "pub trait Properties:Sized{#[lang=\"mem_size_const\"]const SIZE:u32=0;}",
        "pub trait Properties:Sized{#[lang=\"mem_align_const\"]const ALIGN:bool=false;}",
    ] {
        let error = inspect(tail)
            .err()
            .ok_or("wrong member contract accepted")?
            .to_string();
        assert!(error.contains("requires usize type"), "{error}");
    }
    inspect("pub trait Properties:Sized{const SIZE:u32=0;const ALIGN:bool=false;}")?;
    Ok(())
}

#[test]
fn early_function_preparation_preserves_nested_owner_and_local_contexts() -> Result {
    for body in [
        "const fn inner<U>()->usize{size_of::<U>()}inner::<T>()",
        "const{let _x:&'static u8=&7;size_of::<T>()}",
        "const N:usize=2;let _n=N;size_of::<T>()",
    ] {
        let tail = format!(
            "#[rustc_intrinsic]const fn size_of<T>()->usize;const fn wrapper<T>()->usize{{{body}}}const N:usize=wrapper::<u32>();type A=[u8;N];"
        );
        let facts = inspect(&tail).map_err(|e| format!("{body}: {e}"))?;
        assert!(facts.array_lengths.values().any(|&n| n == 4));
    }
    Ok(())
}

#[test]
fn warmed_source_layout_does_not_pay_an_unrelated_callers_predicates() -> Result {
    let tail = "#[rustc_intrinsic]const fn size_of<T>()->usize;type Warm=[u8;size_of::<u32>()];trait Paid{}const fn require<T:Paid>(){}const fn unpaid<T>()->usize{require::<T>();size_of::<u32>()}";
    let error = inspect(tail)
        .err()
        .ok_or("unpaid caller accepted")?
        .to_string();
    assert!(error.contains("trait obligation"), "{error}");
    Ok(())
}

#[test]
fn runtime_layout_calls_do_not_execute_a_source_fallback() -> Result {
    for body in [";", "{0}"] {
        let tail = format!(
            "#[rustc_intrinsic]pub const fn size_of<T>()->usize{body}#[unsafe(no_mangle)]pub extern \"C\" fn observe()->usize{{size_of::<u32>()}}"
        );
        let report = resolve(&tail)?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        let error = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )
        .err()
        .ok_or("runtime layout call accepted")?;
        assert!(
            error
                .to_string()
                .contains("source layout intrinsic runtime lowering is not implemented"),
            "{error}"
        );
    }
    Ok(())
}

#[test]
fn ordinary_source_function_spelling_does_not_claim_intrinsic_privilege() -> Result {
    let tail = "pub const fn size_of<T>()->usize{0}#[unsafe(no_mangle)]pub extern \"C\" fn observe()->usize{size_of::<u32>()}";
    let report = resolve(tail)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .is_some_and(|c| !c.has_intrinsics())
    );
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok(())
}

#[test]
fn source_associated_constants_cannot_receive_bootstrap_primitive_values() -> Result {
    for tail in [
        "const N:u8=u8::MAX;type A=[u8;N as usize];",
        "type Byte=u8;const N:u8=Byte::MAX;type A=[u8;N as usize];",
        "const N:u8=<u8>::MAX;type A=[u8;N as usize];",
    ] {
        assert!(inspect(tail).is_err(), "{tail}");
    }
    for tail in [
        "struct S;impl S{const MAX:u8=3;}const N:u8=S::MAX;type A=[u8;N as usize];",
        "trait Local{const MAX:u8;}impl Local for u8{const MAX:u8=3;}const N:u8=<u8 as Local>::MAX;type A=[u8;N as usize];",
    ] {
        let facts = inspect(tail)?;
        assert!(facts.primitive_constants.is_none());
        assert!(facts.array_lengths.values().any(|&n| n == 3));
    }
    Ok(())
}

#[test]
fn source_closure_demand_cannot_borrow_bootstrap_copy_or_callable_bindings() -> Result {
    let tail = "#[rustc_intrinsic]const fn size_of<T>()->usize;const fn wrapper<T>()->usize{return size_of::<T>();let _f=|x:&'static u8|x;}const N:usize=wrapper::<u32>();type A=[u8;N];";
    let error = inspect(tail)
        .err()
        .ok_or("missing source closure bindings accepted")?
        .to_string();
    assert!(
        error.contains("source Copy and callable bindings"),
        "{error}"
    );
    Ok(())
}

#[path = "provider_layout/native.rs"]
mod native;
