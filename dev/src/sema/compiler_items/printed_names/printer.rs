//! rustc's type name printer for the types it spells: primitives, `str`,
//! `!`, tuples, arrays, slices, references, raw pointers and nominal types
//! declared directly in a module of this crate. A reference's lifetime is
//! left out; a lifetime argument of a path is erased to `'_`.
use crate::{
    resolve::{
        expansion::Report,
        guests::CrateSlot,
        model::{DefId, DefKind},
    },
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId, Kind, Primitive, TypeId,
        arguments::{Argument, ConstId, ConstKind},
    },
};
use std::fmt::Write as _;

/// A name longer than this is refused instead of being printed further.
const LIMIT: usize = 1 << 16;
const UNSPELLED: &str =
    "type name of a function, closure, object, opaque or foreign type is not implemented";

pub(super) struct Printer<'a> {
    facts: &'a Declarations,
    report: &'a Report,
    text: String,
}
impl<'a> Printer<'a> {
    pub(super) fn name(
        facts: &'a Declarations,
        report: &'a Report,
        ty: TypeId,
    ) -> Result<String, &'static str> {
        let mut printer = Self {
            facts,
            report,
            text: String::new(),
        };
        printer.ty(ty)?;
        Ok(printer.text)
    }
    fn push(&mut self, text: &str) -> Result<(), &'static str> {
        if self.text.len().saturating_add(text.len()) > LIMIT {
            return Err("type name exceeds its size budget");
        }
        self.text.push_str(text);
        Ok(())
    }
    fn number(&mut self, value: impl std::fmt::Display) -> Result<(), &'static str> {
        if self.text.len() >= LIMIT {
            return Err("type name exceeds its size budget");
        }
        write!(self.text, "{value}").map_err(|_| "type name formatting failed")
    }
    fn ty(&mut self, ty: TypeId) -> Result<(), &'static str> {
        let kind = self
            .facts
            .types
            .kind(ty)
            .ok_or("type name has an invalid type")?;
        match kind {
            Kind::Never => self.push("!"),
            Kind::Primitive(primitive) => self.push(primitive.name()),
            Kind::Adt(item, arguments) => {
                self.path(item)?;
                self.arguments(arguments)
            }
            Kind::Tuple(elements) => {
                let facts = self.facts;
                let elements = facts
                    .types
                    .list(elements)
                    .ok_or("type name has an invalid type list")?;
                self.push("(")?;
                self.separated(elements.iter().map(|&element| Argument::Type(element)))?;
                if elements.len() == 1 {
                    self.push(",")?;
                }
                self.push(")")
            }
            Kind::Array { element, length } => {
                self.push("[")?;
                self.ty(element)?;
                self.push("; ")?;
                self.number(length)?;
                self.push("]")
            }
            Kind::ArrayTerm { element, length } => {
                self.push("[")?;
                self.ty(element)?;
                self.push("; ")?;
                self.constant(length)?;
                self.push("]")
            }
            Kind::Slice(element) => {
                self.push("[")?;
                self.ty(element)?;
                self.push("]")
            }
            Kind::Reference {
                mutable, referent, ..
            } => {
                self.push(if mutable { "&mut " } else { "&" })?;
                self.ty(referent)
            }
            Kind::Pointer { mutable, pointee } => {
                self.push(if mutable { "*mut " } else { "*const " })?;
                self.ty(pointee)
            }
            Kind::Parameter(_) | Kind::Projection(_) => {
                Err("type name requires a closed normalized type")
            }
            Kind::Function { .. }
            | Kind::FunctionItem { .. }
            | Kind::Closure { .. }
            | Kind::Dynamic { .. }
            | Kind::Opaque(..)
            | Kind::Future { .. }
            | Kind::Pattern { .. }
            | Kind::Foreign(_) => Err(UNSPELLED),
        }
    }
    /// A path's arguments in order, each lifetime erased to `'_`; none prints nothing.
    fn arguments(&mut self, arguments: GenericArgs) -> Result<(), &'static str> {
        let facts = self.facts;
        let mut shown = facts
            .types
            .arguments(arguments)
            .ok_or("type name has invalid generic arguments")?
            .peekable();
        if shown.peek().is_none() {
            return Ok(());
        }
        self.push("<")?;
        self.separated(shown)?;
        self.push(">")
    }
    fn separated(&mut self, arguments: impl Iterator<Item = Argument>) -> Result<(), &'static str> {
        for (index, argument) in arguments.enumerate() {
            if index != 0 {
                self.push(", ")?;
            }
            match argument {
                Argument::Type(ty) => self.ty(ty)?,
                Argument::Const(id) => self.constant(id)?,
                Argument::Lifetime(_) => self.push("'_")?,
            }
        }
        Ok(())
    }
    /// An evaluated integer or `bool` const, printed without its type suffix.
    fn constant(&mut self, id: ConstId) -> Result<(), &'static str> {
        let Some(ConstKind::Value(value)) = self.facts.types.const_kind(id) else {
            return Err("type name requires evaluated const arguments");
        };
        let Some(Kind::Primitive(primitive)) = self.facts.types.kind(value.ty) else {
            return Err("type name of a non-scalar const argument is not implemented");
        };
        match (primitive, primitive.integer()) {
            (Primitive::Bool, _) => self.push(if value.bits == 0 { "false" } else { "true" }),
            (_, Some((true, width))) => {
                let shift = 128_u32.saturating_sub(u32::from(width));
                #[expect(
                    clippy::cast_possible_wrap,
                    reason = "reinterpret the two's complement bits"
                )]
                let signed = ((value.bits << shift) as i128) >> shift;
                self.number(signed)
            }
            (_, Some((false, _))) => self.number(value.bits),
            _ => Err("type name of a char or float const argument is not implemented"),
        }
    }
    /// The crate, each module from the crate root down and the declaration's name.
    fn path(&mut self, item: ItemId) -> Result<(), &'static str> {
        let report = self.report;
        let imported = self
            .facts
            .crates
            .as_deref()
            .and_then(crate::sema::crates::Linkage::origin_catalog)
            .map(|catalog| catalog.declaration_if_imported(DefId(item.0)))
            .transpose()?
            .flatten();
        if imported.is_some() {
            return Err("type name of another crate's declaration is not implemented");
        }
        let resolution = &report.resolution;
        let definition = resolution
            .definitions
            .get(item.0 as usize)
            .ok_or("type name has an unknown declaration")?;
        let module = resolution
            .scopes
            .get(definition.scope.0 as usize)
            .ok_or("type name declaration has no actual scope")?
            .module;
        if !matches!(definition.kind, DefKind::Struct | DefKind::Enum)
            || resolution.module_scopes.get(module.0 as usize) != Some(&definition.scope)
        {
            return Err("type name of a declaration outside a module is not implemented");
        }
        let names = &report.package.names;
        let mut segments = vec![names.get(definition.name)];
        let mut current = module;
        loop {
            let record = report
                .package
                .modules
                .get(current.0 as usize)
                .ok_or("type name declaration has no actual module")?;
            if record.krate != CrateSlot::HOST {
                return Err("type name of a guest crate's declaration is not implemented");
            }
            let Some(parent) = record.parent else { break };
            segments.push(names.get(record.name));
            current = parent;
        }
        if segments.iter().any(|segment| segment.starts_with("r#")) {
            return Err("type name of a raw identifier path is not implemented");
        }
        self.push(&report.package.crate_name)?;
        for segment in segments.iter().rev() {
            self.push("::")?;
            self.push(segment)?;
        }
        Ok(())
    }
}
