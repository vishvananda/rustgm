//! Alias identity and authority are separate from the still-closed typed contract.
use crate::{
    expand::config::Config,
    pipeline,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        model::{DefId, DefKind, Target},
        package::Error,
    },
    support::{diagnostics::WarningCode, source::SourceFile},
    syntax::{ast::Kind, gate_sites, item::ItemKind},
};
use std::{fmt::Write, path::Path};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> Result<Report, Error> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "aliases.rs".into(),
            path: "aliases.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec()).map_err(Error::unlocated)?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    resolve::analyze_expanded(report)
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/trait-aliases/{name}.rs"
    ))?)
}
#[test]
fn aliases_have_distinct_identity_lexical_self_and_no_trait_members_or_semantic_grant() -> TestResult
{
    let source = "#![feature(trait_alias)]trait P<T>{}pub trait A<U>=P<U> where Self:P<U>;use A as B;fn main(){}";
    let found = report(source)?;
    let (index, alias) = found
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, item)| {
            item.kind == DefKind::TraitAlias && found.package.names.get(item.name) == "A"
        })
        .ok_or("actual alias identity")?;
    assert!(
        alias.members.is_none(),
        "lexical generics are not associated members"
    );
    let key = alias.node.ok_or("actual alias node")?;
    let scope = found
        .resolution
        .node_scopes
        .get(&(crate::resolve::package::ModuleId(0), key.node))
        .copied()
        .ok_or("actual lexical scope")?;
    assert_ne!(scope, alias.scope);
    let self_definition = found
        .resolution
        .definitions
        .iter()
        .find(|item| item.kind == DefKind::SelfType && item.scope == scope)
        .ok_or("real lexical Self")?;
    assert!(self_definition.members.is_none());
    assert!(found.resolution.definitions.iter().any(|item| {
        item.kind == DefKind::Generic
            && item.scope == scope
            && found.package.names.get(item.name) == "U"
    }));
    let unit = found.package.unit(key.file);
    let Kind::Item(item) = unit.configured.ast.node(key.node).kind else {
        return Err("alias no longer owns its actual item".into());
    };
    let site = unit
        .configured
        .ast
        .gate_site(item)
        .ok_or("actual whole-item site")?;
    assert_eq!(site.kind, gate_sites::Kind::TraitAlias);
    assert_eq!(site.span, unit.configured.ast.node(key.node).span);
    assert_eq!(
        &source[site.span.start as usize..site.span.end as usize],
        "pub trait A<U>=P<U> where Self:P<U>;"
    );
    assert!(unit.configured.ast.defaultness(item).is_none());
    assert!(!unit.configured.ast.has_defaultness());
    assert!(matches!(
        unit.configured.ast.items[item as usize].kind,
        ItemKind::TraitAlias { .. }
    ));
    assert_ne!(found.resolution.definitions[index].kind, DefKind::Trait);
    let error = pipeline::analyze_source(&found)
        .err()
        .ok_or("unimplemented alias published semantic facts")?;
    assert!(
        error
            .message
            .contains("trait alias contract expansion is not implemented"),
        "{error}"
    );
    let projection =
        report("#![feature(trait_alias)]trait P{}trait A<U>=P;type X=A::U;fn main(){}")?;
    let generic = projection
        .resolution
        .definitions
        .iter()
        .position(|item| {
            item.kind == DefKind::Generic && projection.package.names.get(item.name) == "U"
        })
        .ok_or("alias lexical generic")?;
    let generic = Target::Definition(DefId(u32::try_from(generic)?));
    // Associated paths can remain deferred for semantics; lexical U is never
    // an exported member or the resolved target of the written A::U path.
    assert!(
        projection
            .resolution
            .references
            .iter()
            .all(|reference| reference.target != generic)
    );
    assert!(pipeline::analyze_source(&projection).is_err());
    Ok(())
}
#[test]
fn actual_alias_feature_frames_and_unconditional_disabled_diagnostics_match_observations()
-> TestResult {
    for name in ["active-missing-feature", "forwarded-alias-allowance"] {
        let error = report(&fixture(name)?)
            .err()
            .ok_or("ungranted item authority")?;
        assert!(error.message.contains("trait_alias"), "{name}: {error}");
    }
    let literal = report(&fixture("literal-alias-allowance")?)?;
    // The literal definition grants its actual syntax allowance. This direct
    // contract now has complete local consumers independently of that grant.
    pipeline::analyze_source(&literal)?;
    for prefix in [
        "",
        "#![allow(unstable_syntax_pre_expansion)]",
        "#![deny(unstable_syntax_pre_expansion)]",
        "#![forbid(unstable_syntax_pre_expansion)]",
    ] {
        let source = format!("{prefix}#[cfg(any())]pub trait A=Missing;fn main(){{}}");
        let found = report(&source)?;
        let analysis = pipeline::analyze_source(&found)?;
        let warnings: Vec<_> = analysis.facts.warnings().collect();
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].code, WarningCode::UnstableSyntaxPreExpansion);
        assert_eq!(
            warnings[0].diagnostic.message,
            "trait aliases are experimental"
        );
        assert_eq!(
            warnings[0].diagnostic.span.start as usize,
            source.find("pub trait").ok_or("whole-item start")?
        );
    }
    let source = "#![feature(trait_alias,const_trait_impl)]trait P{}const trait A=P;fn main(){}";
    report(source)?;
    let missing = source.replace("trait_alias,const_trait_impl", "trait_alias");
    let error = report(&missing)
        .err()
        .ok_or("const alias omitted its own gate")?;
    assert!(error.message.contains("const_trait_impl"));
    Ok(())
}
#[test]
fn alias_sites_share_bounded_storage_replay_and_reject_changed_images_and_spans() -> TestResult {
    let ordinary = report("fn main(){}")?;
    assert!(
        ordinary.package.units[0]
            .configured
            .ast
            .gate_sites
            .is_none()
    );
    for width in [8, 32, 128] {
        let mut source = String::from("fn main(){}");
        for index in 0..width {
            write!(source, "#[cfg(any())]trait A{index}=Missing;")?;
        }
        let found = report(&source)?;
        let sites = found.package.units[0]
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("sparse owner")?;
        let before = sites.storage();
        assert_eq!(before.0, width);
        assert!(sites.pending_context().is_none());
        for _ in 0..2 {
            assert_eq!(
                pipeline::analyze_source(&found)?.facts.warnings().count(),
                width
            );
        }
        let after = sites.storage();
        assert_eq!((before.0, before.1), (after.0, after.1));
        assert!(after.2 > before.2 && after.2 - before.2 < width * 1024);
        println!(
            "alias sites width={width} row={} owner={} capacity={} replay-work={}",
            size_of::<gate_sites::Declaration>(),
            size_of::<gate_sites::Declarations>(),
            after.1,
            after.2 - before.2
        );
    }
    for image in [true, false] {
        let mut found = report(&fixture("inactive-missing-feature")?)?;
        let site = found.package.units[0]
            .configured
            .ast
            .gate_sites
            .as_ref()
            .ok_or("actual site")?
            .entries()[0];
        if image {
            found.package.units[0].source = found.package.units[0].source.physical_copy();
        } else {
            found.package.units[0].configured.ast.nodes[site.node.get() as usize]
                .span
                .start += 1;
        }
        let error = found
            .pre_expansion_warnings()
            .err()
            .ok_or("changed source/site retained authority")?;
        assert!(error.message.contains("defaultness"), "{error}");
    }
    assert_eq!(size_of::<gate_sites::Declaration>(), 24);
    assert_eq!(size_of::<gate_sites::Declarations>(), 88);
    assert_eq!(size_of::<crate::syntax::item::ItemKind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    Ok(())
}
