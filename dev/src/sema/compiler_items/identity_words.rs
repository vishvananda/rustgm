//! Type identities in constants: bring-up only, waived by name, fenced otherwise.
//! `type_id::<T>()` is the reflected identity structure holding the 128-bit
//! digest of T's spelling as two address words without provenance, low half
//! first; `type_id_eq` compares those words. The words are plain numbers in
//! the data image, so the run-time fallback body of `type_id_eq`, which
//! compares the two values as integers, agrees with every constant result.
mod spelling;
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::model::Declarations,
    support::bindings::{LangItem, SourceIntrinsic},
    types::{
        ItemId, Kind, Primitive, TypeId, arguments::Argument, callable::Convention,
        instance::Instance,
    },
};

const IDENTITY: &str = "source intrinsic uses unqualified constant evaluation: type_id";
const EQUALITY: &str = "source intrinsic uses unqualified constant evaluation: type_id_eq";

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    /// `type_id::<T>()`, with its closed subject T.
    Identity(TypeId),
    Equality,
}

/// Whether `kind` is one of the two operations. The instance supplies the subject.
fn selected(kind: SourceIntrinsic) -> Option<bool> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    match identity.name() {
        "type_id" if crate::support::bringup::waive(IDENTITY) => Some(false),
        "type_id_eq" if crate::support::bringup::waive(EQUALITY) => Some(true),
        _ => None,
    }
}
pub(crate) fn is_operation(kind: SourceIntrinsic) -> bool {
    selected(kind).is_some()
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    /// The reflected identity structure: one array of two `*const ()` words.
    reflected: TypeId,
    operation: Operation,
}
impl Selection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn operation(&self) -> Operation {
        self.operation
    }
    pub(crate) const fn output(&self) -> TypeId {
        match self.operation {
            Operation::Identity(_) => self.reflected,
            Operation::Equality => TypeId::primitive(Primitive::Bool),
        }
    }
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        let pair = [self.reflected; 2];
        let inputs: &[TypeId] = match self.operation {
            Operation::Identity(_) => &[],
            Operation::Equality => &pair,
        };
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            inputs,
            self.output(),
            false,
            Convention::RUST,
        )
    }
    /// The two words of the subject's identity, low half first as in the
    /// integer of the same bytes, and the bytes spelled to compute them.
    pub(crate) fn identity(
        facts: &Declarations,
        subject: TypeId,
    ) -> Result<([u64; 2], usize), &'static str> {
        let spelled = spelling::Spelling::of(facts, subject)?;
        let identity = spelling::identity(&spelled)?;
        #[expect(clippy::cast_possible_truncation, reason = "split both 64-bit halves")]
        let words = [identity as u64, (identity >> 64) as u64];
        Ok((words, spelled.len()))
    }
}

impl Catalog {
    /// The reflected identity structure, whose value this module constructs:
    /// exactly one field, an array of two thin immutable raw pointers.
    fn reflected_identity(&self, facts: &Declarations) -> Result<TypeId, &'static str> {
        let declaration = self
            .language(LangItem::TypeId)
            .ok_or("source type identity requires its reflected identity structure")?;
        let item = ItemId(declaration.definition.0);
        let reflected = facts
            .definitions
            .get(item.0 as usize)
            .copied()
            .flatten()
            .ok_or("reflected identity structure has no canonical declaration type")?;
        let definition = facts
            .aggregates
            .get(item)
            .ok_or("reflected identity structure fields are not complete")?;
        let words = match facts.aggregates.variants(definition) {
            [variant] => facts.types.list(variant.fields),
            _ => None,
        };
        let Some(&[words]) = words else {
            return Err("reflected identity structure requires its one field");
        };
        if !matches!(facts.types.kind(reflected), Some(Kind::Adt(owner, _)) if owner == item)
            || !matches!(facts.types.kind(words), Some(Kind::Array { element, length: 2 })
            if matches!(facts.types.kind(element), Some(Kind::Pointer {
                mutable: false, pointee: TypeId::UNIT,
            })))
        {
            return Err("reflected identity structure requires two `*const ()` words");
        }
        Ok(reflected)
    }
    pub(crate) fn type_identity(
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
        let Some((declaration, kind, original)) = self.intrinsic_record(definition)? else {
            return Ok(None);
        };
        let Some(equality) = selected(kind) else {
            return Ok(None);
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source type identity selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source type identity provenance requires its wider transport profile");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source type identity requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source type identity completed declaration changed");
        }
        let reflected = self.reflected_identity(facts)?;
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source type identity has invalid instance arguments")?;
        let operation = match (equality, args.next(), args.next()) {
            (false, Some(Argument::Type(subject)), None)
                if facts.types.contains_parameters(subject) == Some(false) =>
            {
                Operation::Identity(subject)
            }
            (true, None, _) => Operation::Equality,
            _ => return Err("source type identity requires its complete closed instance"),
        };
        let selection = Selection {
            definition,
            signature,
            reflected,
            operation,
        };
        if !selection.opened_matches(facts, signature) {
            return Err("source type identity changed its exact safe Rust signature");
        }
        Ok(Some(selection))
    }
    pub(crate) fn require_type_identity_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selection: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((_, kind, _)) = self.intrinsic_record(selection.definition)? else {
            return Err("source type identity lost its attributed declaration");
        };
        let equality = matches!(selection.operation, Operation::Equality);
        if selected(kind) != Some(equality) || !facts.is_const_function(selection.definition) {
            return Err("source type identity requires its actual compile-time declaration");
        }
        Ok(())
    }
}
