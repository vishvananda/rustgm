//! Warm identity queries never turn a source Drop member into an ordinary method.
use super::*;
use crate::resolve::{
    bindings::Context,
    load::{InputFile, Sources},
};
use crate::{
    expand::config::Config,
    resolve, sema,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Empty;
impl Sources for Empty {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture() -> Result<(Declarations, DefId, DefId, DefId)> {
    let source = [crate::provider_cases::GENERIC_COPY,
        "#[lang=\"drop\"]pub const trait Finalizer{fn drop(&mut self);}struct D;impl Finalizer for D{fn drop(&mut self){}}mod local{pub trait Finalizer{fn drop(&mut self);}impl Finalizer for u8{fn drop(&mut self){}}}"].concat();
    let identity = CrateIdentity::new("minicore", "explicit_drop_identity")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let input = InputFile {
        identity: "calls.rs".into(),
        path: "calls.rs".into(),
        source: SourceFile::new(source.into_bytes())?,
    };
    let expanded =
        resolve::expansion::analyze_with_context(input, &mut Empty, &Config::default(), &context)?;
    let report = resolve::analyze_expanded(expanded)?;
    let facts = sema::declarations::analyze(&report)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::Drop)?
        .ok_or("Drop")?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or("owner")?;
    let [member] = facts.associated.owner_members(owner).ok_or("members")? else {
        return Err("member profile".into());
    };
    let member = *member;
    let mut actual = None;
    let mut other = None;
    for header in facts.generics.implementations() {
        if let Some((declared, Some(method))) = header.methods.first() {
            if *declared == member {
                actual = Some(*method);
            } else if report
                .package
                .names
                .get(report.resolution.definitions[method.0 as usize].name)
                == "drop"
            {
                other = Some(*method);
            }
        }
    }
    Ok((
        facts,
        member,
        actual.ok_or("override")?,
        other.ok_or("lookalike")?,
    ))
}
#[test]
fn explicit_source_drop_identity_queries_keep_warm_owners_and_pending_headers_distinct() -> Result {
    let (mut facts, member, actual, other) = fixture()?;
    for _ in 0..32 {
        assert!(!explicit_drop_member(&facts, other)?);
        assert!(explicit_drop_member(&facts, member)?);
        assert!(explicit_drop_member(&facts, actual)?);
        assert!(!explicit_drop_member(&facts, DefId(u32::MAX))?);
    }
    let registry = facts.generics.implementations.take();
    assert!(explicit_drop_member(&facts, actual).is_err());
    assert!(!explicit_drop_member(&facts, other)?);
    facts.generics.implementations = registry;
    assert!(explicit_drop_member(&facts, actual)?);
    let roles = facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .traits
        .take();
    assert!(explicit_drop_member(&facts, member).is_err());
    assert!(!explicit_drop_member(&facts, other)?);
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .traits = roles;
    assert!(explicit_drop_member(&facts, member)?);
    Ok(())
}
