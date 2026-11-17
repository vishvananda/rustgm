//! Add compiler-retained statics to the checked emission roots.
use crate::{
    lex::token::Punctuation,
    resolve::{
        expansion::Report,
        model::{DefId, DefKind, Namespace},
        package::{Error, ModuleId},
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{diagnostics::WarningCode, hash::HashMap, retained::Growth},
    syntax::{
        ast::Kind,
        attribute::{Attribute, Input, Style},
        item::ItemKind,
        token_tree::{Delimiter, Forest, TreeKind},
    },
};

#[derive(Clone, Copy)]
enum Mode {
    Default,
    Compiler,
    Linker,
}

#[derive(Default)]
struct Mark {
    compiler: bool,
    linker: bool,
    bound: bool,
    section: Option<Box<str>>,
}

#[derive(Default)]
pub(super) struct Draft {
    marks: HashMap<NodeKey, Mark>,
    requests: HashMap<ModuleId, bool>,
}

#[derive(Default)]
pub(super) struct Retention {
    compiler: Vec<DefId>,
    needs_linker: bool,
    sections: HashMap<DefId, Box<str>>,
}

impl Draft {
    pub(super) fn consume(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        (owner, declaration): (NodeKey, NodeKey),
        attribute: Attribute,
        forest: &Forest,
    ) -> Result<(), Error> {
        facts.reserve_must_use_source(owner, attribute.span)?;
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        let unit = report.package.unit(declaration.file);
        let ast = &unit.configured.ast;
        if !matches!(ast.node(declaration.node).kind, Kind::Item(index)
            if matches!(ast.items[index as usize].kind, ItemKind::Static { value: Some(_), .. }))
        {
            return Err(error("`used` requires an actual static definition"));
        }
        let mode = mode(unit.source.text(), attribute, forest).map_err(error)?;
        if !matches!(mode, Mode::Default) && !self.requested(report, facts, declaration)? {
            return Err(error(
                "argument to `used` requires this crate's used_with_arg feature",
            ));
        }
        self.reserve_mark(facts, declaration)?;
        let mark = self.marks.entry(declaration).or_default();
        if mark.insert(mode) {
            facts.must_use_diagnostic(
                report,
                owner,
                attribute.span.start,
                0,
                WarningCode::UnusedAttributes,
                "duplicate `used` attribute".into(),
            )?;
        }
        Ok(())
    }

    fn reserve_mark(
        &mut self,
        facts: &mut Declarations,
        declaration: NodeKey,
    ) -> Result<(), Error> {
        if !self.marks.contains_key(&declaration) {
            let growth = Growth::map::<NodeKey, Mark>(self.marks.len(), self.marks.capacity(), 1);
            crate::sema::header_attributes::reserve_source_contracts(
                facts,
                growth.work,
                growth.bytes,
            )?;
            self.marks.try_reserve(1).map_err(|_| {
                Error::at(declaration.file, 0, "used static draft allocation failed")
            })?;
        }
        Ok(())
    }

    pub(super) fn attach_section(
        &mut self,
        facts: &mut Declarations,
        declaration: NodeKey,
        attribute: Attribute,
        section: Box<str>,
    ) -> Result<(), Error> {
        let error = |message| Error::at(declaration.file, attribute.span.start, message);
        self.reserve_mark(facts, declaration)?;
        let mark = self.marks.entry(declaration).or_default();
        if mark.section.is_some() {
            return Err(error("duplicate source init-array section"));
        }
        mark.section = Some(section);
        Ok(())
    }

    fn requested(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        declaration: NodeKey,
    ) -> Result<bool, Error> {
        let root = report.package.file_crate_root(declaration.file);
        if let Some(&requested) = self.requests.get(&root) {
            return Ok(requested);
        }
        let requested = report.package.requested_used_with_arg(declaration.file)?;
        let growth =
            Growth::map::<ModuleId, bool>(self.requests.len(), self.requests.capacity(), 1);
        crate::sema::header_attributes::reserve_source_contracts(facts, growth.work, growth.bytes)?;
        self.requests
            .try_reserve(1)
            .map_err(|_| Error::unlocated("used feature request cache allocation failed"))?;
        self.requests.insert(root, requested);
        Ok(requested)
    }

    pub(super) fn bind(mut self, report: &Report, facts: &mut Declarations) -> Result<(), Error> {
        if self.marks.is_empty() {
            return Ok(());
        }
        let definitions = &report.resolution.definitions;
        crate::sema::header_attributes::reserve_source_contracts(
            facts,
            definitions.len(),
            size_of::<Retention>(),
        )?;
        let mut retention = Retention::default();
        for (index, definition) in definitions.iter().enumerate() {
            let Some(key) = definition.node else {
                continue;
            };
            let Some(mark) = self.marks.get_mut(&key) else {
                continue;
            };
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "used static identity overflow"))?,
            );
            if definition.kind != DefKind::Static
                || definition.namespace != Namespace::Value
                || report.resolution.foreign_owners.contains_key(&id)
            {
                return Err(Error::at(
                    key.file,
                    0,
                    "used marker requires its actual nonforeign static",
                ));
            }
            if std::mem::replace(&mut mark.bound, true) {
                return Err(Error::at(
                    key.file,
                    0,
                    "used marker has multiple static definitions",
                ));
            }
            let has_init_array = mark.section.is_some();
            if let Some(section) = mark.section.take() {
                let sections = &mut retention.sections;
                let growth = Growth::map::<DefId, Box<str>>(sections.len(), sections.capacity(), 1);
                crate::sema::header_attributes::reserve_source_contracts(
                    facts,
                    growth.work,
                    growth.bytes,
                )?;
                sections
                    .try_reserve(1)
                    .map_err(|_| Error::at(key.file, 0, "init-array section allocation failed"))?;
                sections.insert(id, section);
            }
            if mark.linker && !has_init_array {
                retention.needs_linker = true;
            } else if mark.linker || mark.compiler {
                let roots = &mut retention.compiler;
                let growth = Growth::vector::<DefId>(roots.len(), roots.capacity(), 1);
                crate::sema::header_attributes::reserve_source_contracts(
                    facts,
                    growth.work,
                    growth.bytes,
                )?;
                roots
                    .try_reserve(1)
                    .map_err(|_| Error::at(key.file, 0, "used static root allocation failed"))?;
                roots.push(id);
            }
        }
        if self.marks.values().any(|mark| !mark.bound) {
            return Err(Error::unlocated(
                "used marker lost its actual static declaration",
            ));
        }
        facts
            .must_use_contracts
            .get_or_insert_default()
            .used_statics = Some(Box::new(retention));
        Ok(())
    }
}

impl Mark {
    fn insert(&mut self, mode: Mode) -> bool {
        match mode {
            Mode::Compiler => std::mem::replace(&mut self.compiler, true),
            Mode::Default | Mode::Linker => std::mem::replace(&mut self.linker, true),
        }
    }
}

fn mode(source: &str, attribute: Attribute, forest: &Forest) -> Result<Mode, &'static str> {
    const MALFORMED: &str = "malformed `used` attribute input";
    if attribute.unsafe_wrapper || attribute.style != Style::Outer {
        return Err(MALFORMED);
    }
    match attribute.input {
        Input::Empty => Ok(Mode::Default),
        Input::Delimited(group) => {
            if !matches!(
                forest.tree(group).map(|tree| tree.kind),
                Some(TreeKind::Group {
                    delimiter: Delimiter::Parenthesis,
                    ..
                })
            ) {
                return Err(MALFORMED);
            }
            let args = forest.children(group).ok_or(MALFORMED)?;
            if !(args.end == args.start + 1
                || args.end == args.start + 2
                    && forest.punctuation(args.start + 1, Punctuation::Comma))
            {
                return Err(MALFORMED);
            }
            match forest.name(source, args.start) {
                Some("compiler") => Ok(Mode::Compiler),
                Some("linker") => Ok(Mode::Linker),
                _ => Err(MALFORMED),
            }
        }
        _ => Err(MALFORMED),
    }
}

impl Declarations {
    pub(crate) fn init_array_section(&self, definition: DefId) -> Option<&str> {
        self.must_use_contracts
            .as_deref()?
            .used_statics
            .as_deref()?
            .sections
            .get(&definition)
            .map(Box::as_ref)
    }

    pub(crate) fn compiler_retained_statics(&self) -> Result<&[DefId], &'static str> {
        let Some(retention) = self
            .must_use_contracts
            .as_deref()
            .and_then(|contracts| contracts.used_statics.as_deref())
        else {
            return Ok(&[]);
        };
        if retention.needs_linker {
            return Err("used linker retention is not implemented for this backend");
        }
        Ok(&retention.compiler)
    }
}
