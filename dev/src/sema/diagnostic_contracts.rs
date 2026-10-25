//! Declaration-bound diagnostic text never supplies a trait or execution proof.
mod attribute_directives;
mod obligation_failure;
mod template_text;
mod typed_format;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
        unit::{FileId, NodeKey},
    },
    sema::{model::Declarations, must_use::WarningCode},
    support::hash::HashMap,
    syntax::{
        attribute::Attribute,
        token_tree::{Forest, Span},
    },
};
use std::cell::Cell;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Internal,
    Stable,
}
impl Mode {
    fn attribute(attribute: Attribute, forest: &Forest, source: &str) -> Option<Self> {
        if attribute.name(forest, source) == Some("rustc_on_unimplemented") {
            return Some(Self::Internal);
        }
        let path = attribute.path;
        (path.end == path.start + 3
            && forest.name(source, path.start) == Some("diagnostic")
            && forest.punctuation(path.start + 1, crate::lex::token::Punctuation::ColonColon)
            && forest.name(source, path.start + 2) == Some("on_unimplemented"))
        .then_some(Self::Stable)
    }
}
#[derive(Clone, Copy)]
struct SourceSite<'a> {
    owner: NodeKey,
    effect: usize,
    attribute: Attribute,
    forest: &'a Forest,
    mode: Mode,
}
#[derive(Clone, Copy)]
enum Slot {
    Subject,
    Parameter { binding: DefId, ordinal: usize },
    Trait,
}
enum Piece {
    Literal(Box<str>),
    Argument(Slot),
}
struct Template(Vec<Piece>);
#[derive(Clone, Copy)]
enum Condition {
    Unit(Slot),
    UnsafeFunction(Slot),
    Unsupported,
}
#[derive(Default)]
struct Directive {
    condition: Option<Condition>,
    message: Option<Template>,
    label: Option<Template>,
    notes: Vec<Template>,
}
struct Contract {
    owner: NodeKey,
    declaration: NodeKey,
    definition: DefId,
    effect: usize,
    span: Span,
    mode: Mode,
    directives: Vec<Directive>,
}
#[derive(Default)]
pub(in crate::sema) struct Contracts {
    declarations: HashMap<DefId, Contract>,
    steps: Cell<usize>,
    bytes: Cell<usize>,
    failed: Cell<bool>,
}
impl Contracts {
    fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        self.failed.set(
            self.failed.get()
                || self.steps.get() > 10_000_000
                || self.bytes.get() > 64 * 1024 * 1024,
        );
        if self.failed.get() {
            return Err("diagnostic contracts exceed their work or storage budget");
        }
        Ok(())
    }
}
fn charge(facts: &mut Declarations, key: NodeKey, steps: usize, bytes: usize) -> Result<(), Error> {
    let existing = facts
        .must_use_contracts
        .as_deref()
        .and_then(|table| table.diagnostics.as_deref());
    if let Some(table) = existing {
        return table
            .charge(steps, bytes)
            .map_err(|message| Error::at(key.file, 0, message));
    }
    let table = Contracts::default();
    let owner = if facts.must_use_contracts.is_none() {
        size_of::<super::must_use::Contracts>()
    } else {
        0
    };
    table
        .charge(
            steps,
            bytes
                .saturating_add(size_of::<Contracts>())
                .saturating_add(owner),
        )
        .map_err(|message| Error::at(key.file, 0, message))?;
    facts.must_use_contracts.get_or_insert_default().diagnostics = Some(Box::new(table));
    Ok(())
}
fn warning(
    facts: &mut Declarations,
    report: &Report,
    key: NodeKey,
    offset: u32,
    issue: u32,
    code: WarningCode,
    message: &'static str,
) -> Result<(), Error> {
    charge(
        facts,
        key,
        1,
        message
            .len()
            .saturating_add(4 * size_of::<super::must_use::Warning>()),
    )?;
    facts.must_use_diagnostic(report, key, offset, issue, code, message.into())
}

pub(super) fn collect(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let mut definitions: Option<HashMap<NodeKey, DefId>> = None;
    let mut allowances = report.macro_allowances();
    for (file, unit) in report.package.units.iter().enumerate() {
        if unit.include_context.is_some() {
            continue;
        }
        let file = FileId(
            u32::try_from(file)
                .map_err(|_| Error::at(FileId(0), 0, "too many diagnostic files"))?,
        );
        for owner in unit.configured.owners.iter().filter(|owner| owner.enabled) {
            let key = NodeKey {
                file,
                node: owner.node,
            };
            for effect in owner.attributes.clone() {
                let Some((attribute, forest)) = unit.retained_attributes(effect..effect + 1).next()
                else {
                    continue;
                };
                let Some(mode) = Mode::attribute(attribute, forest, unit.source.text()) else {
                    continue;
                };
                charge(facts, key, 1, 0)?;
                if mode == Mode::Internal {
                    require_internal(report, &mut allowances, file, attribute, forest)?;
                }
                if definitions.is_none() {
                    definitions = Some(trait_definitions(report, facts, key)?);
                }
                let map = definitions.as_ref().ok_or_else(|| {
                    Error::at(
                        file,
                        attribute.span.start,
                        "diagnostic declaration index is missing",
                    )
                })?;
                attribute_directives::collect(
                    report,
                    facts,
                    SourceSite {
                        owner: key,
                        effect,
                        attribute,
                        forest,
                        mode,
                    },
                    map,
                )?;
            }
        }
    }
    Ok(())
}
fn require_internal(
    report: &Report,
    allowances: &mut crate::resolve::expansion::MacroAllowanceSession<'_>,
    file: FileId,
    attribute: Attribute,
    forest: &Forest,
) -> Result<(), Error> {
    if report.package.requested_internal_attributes() {
        return Ok(());
    }
    let span = forest
        .tree(attribute.path.start)
        .ok_or_else(|| {
            Error::at(
                file,
                attribute.span.start,
                "diagnostic attribute has no name token",
            )
        })?
        .span;
    let feature = crate::resolve::bindings::FeatureName::new("rustc_attrs")
        .map_err(|message| Error::at(file, span.start, message))?;
    if allowances.allows_at(file, span, &feature)? {
        return Ok(());
    }
    Err(Error::at(
        file,
        span.start,
        "rustc_on_unimplemented requires rustc_attrs",
    ))
}

fn trait_definitions(
    report: &Report,
    facts: &mut Declarations,
    key: NodeKey,
) -> Result<HashMap<NodeKey, DefId>, Error> {
    let mut definitions = HashMap::default();
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        charge(facts, key, 1, 0)?;
        if definition.kind != DefKind::Trait {
            continue;
        }
        if let Some(node) = definition.node {
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "too many diagnostic declarations"))?,
            );
            charge(
                facts,
                key,
                1,
                crate::support::retained::new_map::<NodeKey, DefId>(1),
            )?;
            definitions.insert(node, id);
        }
    }
    Ok(definitions)
}
