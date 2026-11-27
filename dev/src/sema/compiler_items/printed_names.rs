//! Type names are computed for checked source constant calls.
//! `type_name::<T>()` is T's name as rustc 1.98.0 prints it. A nominal type
//! is its declaration's crate and module path followed by its arguments, each
//! lifetime erased to `'_`; a reference leaves its lifetime out. Every other
//! structural type is spelled as Rust writes it. A type whose name would need
//! any other path, or a form this printer does not spell, is refused.
mod printer;
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::model::Declarations,
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, RegionId, TypeId, arguments::Argument, callable::Convention,
        instance::Instance,
    },
};

pub(crate) fn is_operation(kind: SourceIntrinsic) -> bool {
    matches!(kind, SourceIntrinsic::Declaration(declared) if declared.name() == "type_name")
}

/// The name of a closed type, at most a bounded number of bytes long.
/// # Errors
/// Refuses an open type and every type this printer does not spell.
pub(crate) fn name(
    facts: &Declarations,
    report: &Report,
    subject: TypeId,
) -> Result<String, &'static str> {
    printer::Printer::name(facts, report, subject)
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    /// The declared result, `&'static str`.
    output: TypeId,
    subject: TypeId,
}
impl Selection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn output(&self) -> TypeId {
        self.output
    }
    pub(crate) const fn subject(&self) -> TypeId {
        self.subject
    }
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            &[],
            self.output,
            false,
            Convention::RUST,
        )
    }
}

/// The result of a signature that takes nothing and returns `&'static str`.
fn static_text(facts: &Declarations, signature: TypeId) -> Option<TypeId> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return None;
    };
    let text = matches!(facts.types.kind(result), Some(Kind::Reference {
        mutable: false, region: RegionId::STATIC, referent,
    }) if referent == TypeId::primitive(Primitive::Str));
    (facts.types.list(parameters) == Some(&[]) && text).then_some(result)
}

impl Catalog {
    pub(crate) fn type_name(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            arguments,
            signature,
        }) = facts.types.kind(instance.ty())
        else {
            return Ok(None);
        };
        let definition = DefId(item.0);
        let original =
            if let Some((declaration, kind, original)) = self.intrinsic_record(definition)? {
                if !is_operation(kind) {
                    return Ok(None);
                }
                crate::sema::header_attributes::charge_source_contract_query(facts, 16)
                    .map_err(|_| "source type name selection work budget exhausted")?;
                if origin.source().is_none()
                    || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
                {
                    return Err("source type name provenance requires its wider transport profile");
                }
                let function = facts
                    .function_indices
                    .get(&definition)
                    .and_then(|&index| facts.functions.get(index))
                    .ok_or("source type name requires its completed actual function")?;
                if declaration.parent.is_some()
                    || function.definition != definition
                    || function.node != declaration.origin
                    || function.signature != original
                    || function.body.is_some()
                {
                    return Err("source type name completed declaration changed");
                }
                original
            } else if let Some(imported) = self.imported_declared_intrinsic(definition) {
                if !is_operation(imported.kind) {
                    return Ok(None);
                }
                let actual = facts
                    .declaration_header(definition)?
                    .ok_or("imported type name lost its canonical declaration")?;
                if actual.declaration != imported.origin
                    || actual.signature != imported.signature
                    || actual.body.is_some()
                {
                    return Err("imported type name completed declaration changed");
                }
                imported.signature
            } else {
                return Ok(None);
            };
        let output = static_text(facts, original)
            .ok_or("source type name requires its exact safe Rust signature")?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source type name has invalid instance arguments")?;
        let (Some(Argument::Type(subject)), None) = (args.next(), args.next()) else {
            return Err("source type name requires its one type argument");
        };
        if facts.types.contains_parameters(subject) != Some(false) {
            return Err("source type name requires its complete closed instance");
        }
        let selection = Selection {
            definition,
            signature,
            output,
            subject,
        };
        if !selection.opened_matches(facts, signature) {
            return Err("source type name changed its exact safe Rust signature");
        }
        Ok(Some(selection))
    }
    pub(crate) fn require_type_name_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selection: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let kind = if let Some((_, kind, _)) = self.intrinsic_record(selection.definition)? {
            kind
        } else if let Some(imported) = self.imported_declared_intrinsic(selection.definition) {
            imported.kind
        } else {
            return Err("source type name lost its attributed declaration");
        };
        if !is_operation(kind) || !facts.is_const_function(selection.definition) {
            return Err("source type name requires its actual compile-time declaration");
        }
        Ok(())
    }
}
