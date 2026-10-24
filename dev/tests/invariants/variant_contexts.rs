//! Alias suffixes consume real enum owners with supplied opaque provenance.
//! Scoped macro source admission is still disabled; these are consumer controls.
use super::*;
use crate::{
    expand::{
        config::Config,
        marks::{ItemContext, SyntaxContextId, Transparency},
    },
    resolve::{
        expansion::{self, Report},
        load::{InputFile, Sources},
        package::ModuleId,
    },
    support::source::SourceFile,
};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}

struct Fixture {
    report: Report,
    key: NodeKey,
    suffix: NodeKey,
    prefix: NodeKey,
    item: ItemId,
    name: NameId,
    members: ScopeId,
    use_scope: ScopeId,
    variant: DefId,
    syntax: SyntaxContextId,
    context: ItemContext,
}

impl Fixture {
    fn new(source: &str) -> TestResult<Self> {
        let report = expansion::analyze(
            InputFile {
                identity: "enum-context.rs".into(),
                path: "enum-context.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
        )?;
        let mut report = crate::resolve::analyze_expanded(report)?;
        let expansion = report
            .invocations
            .first()
            .ok_or("missing invocation")?
            .context;
        let syntax = report.package.marks.apply(
            SyntaxContextId::ROOT,
            expansion,
            Transparency::Opaque,
            SyntaxContextId::ROOT,
        )?;
        let context = report.package.marks.item(syntax)?;
        let (ordinal, definition) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, definition)| {
                definition.kind == DefKind::Enum
                    && report.package.names.get(definition.name) == "Choice"
            })
            .ok_or("missing enum")?;
        let item = ItemId(u32::try_from(ordinal)?);
        let members = definition.members.ok_or("missing enum scope")?;
        let (key, path) = report
            .resolution
            .associated_paths
            .iter()
            .find(|(_, path)| {
                path.suffix.len() == 1 && report.package.names.get(path.suffix[0].1) == "First"
            })
            .ok_or("missing alias use")?;
        let key = *key;
        let suffix = NodeKey {
            node: path.suffix[0].0,
            ..key
        };
        let prefix = NodeKey {
            node: path.prefix,
            ..key
        };
        let name = path.suffix[0].1;
        let use_scope = path.scope;
        let variant = report
            .resolution
            .bindings
            .get(members, name, Namespace::Value)
            .ok_or("missing variant")?
            .target;
        Ok(Self {
            report,
            key,
            suffix,
            prefix,
            item,
            name,
            members,
            use_scope,
            variant,
            syntax,
            context,
        })
    }

    fn set_suffix(&mut self, syntax: SyntaxContextId) -> TestResult {
        let unit = &mut self.report.package.units[self.suffix.file.0 as usize];
        let span = unit.configured.ast.node(self.suffix.node).span;
        let origin = unit
            .origins
            .iter_mut()
            .find(|origin| origin.generated == span)
            .ok_or("missing suffix token provenance")?;
        origin.context = Some(syntax);
        assert_eq!(
            self.report
                .package
                .syntax_context(self.suffix.file, span.start)?,
            syntax
        );
        Ok(())
    }

    fn selected(&self, scope: ScopeId) -> Result<Option<DefId>, Error> {
        Checker::new(&self.report)?.enum_variant(
            self.key,
            self.suffix,
            self.item,
            self.name,
            Namespace::Value,
            scope,
        )
    }

    fn caller_scope(&self) -> TestResult<ScopeId> {
        let caller = self
            .report
            .package
            .modules
            .iter()
            .position(|module| self.report.package.names.get(module.name) == "caller")
            .ok_or("missing caller")?;
        Ok(self.report.resolution.module_scopes[caller])
    }
}

#[test]
fn introduced_variant_uses_suffix_identity_under_a_captured_alias() -> TestResult {
    let source = "mod definition { macro_rules! make { () => { pub enum Choice { First } type Alias = Choice; pub fn produced() { let _ = Alias::First; } } } make!(); } mod caller {}";
    let mut fixture = Fixture::new(source)?;
    assert_eq!(fixture.selected(fixture.use_scope)?, Some(fixture.variant));
    let root_key = (
        fixture.members,
        fixture.name,
        Namespace::Value,
        ItemContext::ROOT,
    );
    let binding = *fixture
        .report
        .resolution
        .bindings
        .get_key(root_key)
        .ok_or("missing root variant")?;
    fixture.report.resolution.bindings.remove_key(root_key);
    fixture.report.resolution.bindings.insert_key(
        (
            fixture.members,
            fixture.name,
            Namespace::Value,
            fixture.context,
        ),
        binding,
    )?;
    let prefix_offset = fixture
        .report
        .package
        .unit(fixture.prefix.file)
        .configured
        .ast
        .node(fixture.prefix.node)
        .span
        .start;
    assert_eq!(
        fixture
            .report
            .package
            .item_context(fixture.prefix.file, prefix_offset)?,
        ItemContext::ROOT
    );
    for syntax in [
        fixture.syntax,
        fixture.syntax,
        SyntaxContextId::ROOT,
        fixture.syntax,
    ] {
        fixture.set_suffix(syntax)?;
        let expected = (syntax == fixture.syntax).then_some(fixture.variant);
        assert_eq!(fixture.selected(fixture.use_scope)?, expected);
    }
    Ok(())
}

#[test]
fn alias_variant_privacy_uses_the_stripped_definition_context() -> TestResult {
    let source = "mod definition { enum Choice { First } type Alias = Choice; macro_rules! make { () => { pub fn produced() { let _ = Alias::First; } } } make!(); } mod caller {}";
    let mut fixture = Fixture::new(source)?;
    let caller = fixture.caller_scope()?;
    let binding = fixture
        .report
        .resolution
        .bindings
        .get(fixture.members, fixture.name, Namespace::Value)
        .ok_or("missing variant")?;
    assert!(matches!(binding.access, Access::Within(ModuleId(_))));
    fixture.set_suffix(fixture.syntax)?;
    for _ in 0..3 {
        assert_eq!(fixture.selected(caller)?, Some(fixture.variant));
    }
    fixture.set_suffix(SyntaxContextId::ROOT)?;
    assert!(fixture.selected(caller).is_err());
    assert_eq!(fixture.selected(fixture.use_scope)?, Some(fixture.variant));
    fixture.set_suffix(fixture.syntax)?;
    assert_eq!(fixture.selected(caller)?, Some(fixture.variant));
    Ok(())
}
