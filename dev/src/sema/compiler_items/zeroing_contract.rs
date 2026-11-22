//! Authenticate the source intrinsics that assert zero validity or fill raw storage.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::model::Declarations,
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Operation {
    /// `assert_zero_valid::<T>()`, with its closed subject T.
    Assertion(TypeId),
    /// `write_bytes::<T>`, with its closed `*mut T` destination type.
    Fill(TypeId),
}

/// Whether `kind` is one of the two operations, `Some(true)` for the fill.
fn selected(kind: SourceIntrinsic) -> Option<bool> {
    let SourceIntrinsic::Declaration(identity) = kind else {
        return None;
    };
    match identity.name() {
        "assert_zero_valid" => Some(false),
        "write_bytes" => Some(true),
        _ => None,
    }
}
pub(crate) fn is_operation(kind: SourceIntrinsic) -> bool {
    selected(kind).is_some()
}

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
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
    /// The value types of the call, in source order.
    pub(crate) const fn inputs(&self) -> [TypeId; 3] {
        match self.operation {
            Operation::Assertion(_) => [TypeId::UNIT; 3],
            Operation::Fill(destination) => [
                destination,
                TypeId::primitive(Primitive::U8),
                TypeId::primitive(Primitive::Usize),
            ],
        }
    }
    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        let inputs = self.inputs();
        let (inputs, unsafe_): (&[TypeId], _) = match self.operation {
            Operation::Assertion(_) => (&[], false),
            Operation::Fill(_) => (&inputs, true),
        };
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            inputs,
            TypeId::UNIT,
            unsafe_,
            Convention::RUST,
        )
    }
}

impl Catalog {
    pub(crate) fn zeroing(
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
        let Some(fill) = selected(kind) else {
            return Ok(None);
        };
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source zeroing selection work budget exhausted")?;
        if origin.source().is_none()
            || matches!(self.bindings, super::binding_views::Bindings::Imported(_))
        {
            return Err("source zeroing provenance requires its wider transport profile");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source zeroing requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
            || function.body.is_some()
        {
            return Err("source zeroing requires its own bodyless declaration");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source zeroing has invalid instance arguments")?;
        let (Some(Argument::Type(subject)), None) = (args.next(), args.next()) else {
            return Err("source zeroing requires its one type argument");
        };
        if facts.types.contains_parameters(subject) != Some(false) {
            return Err("source zeroing requires a closed type");
        }
        let operation = if fill {
            let destination = match facts.types.kind(signature) {
                Some(Kind::Function { parameters, .. }) => facts
                    .types
                    .list(parameters)
                    .and_then(|inputs| inputs.first().copied()),
                _ => None,
            }
            .ok_or("source write_bytes has no destination")?;
            if !matches!(facts.types.kind(destination),
                Some(Kind::Pointer { mutable: true, pointee }) if pointee == subject)
            {
                return Err("source write_bytes changed its `*mut T` destination");
            }
            Operation::Fill(destination)
        } else {
            Operation::Assertion(subject)
        };
        let selection = Selection {
            definition,
            signature,
            operation,
        };
        if !selection.opened_matches(facts, signature) {
            return Err("source zeroing changed its exact Rust signature");
        }
        Ok(Some(selection))
    }
    pub(crate) fn require_zeroing_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selection: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((_, kind, _)) = self.intrinsic_record(selection.definition)? else {
            return Err("source zeroing lost its attributed declaration");
        };
        let fill = matches!(selection.operation, Operation::Fill(_));
        if selected(kind) != Some(fill) || !facts.is_const_function(selection.definition) {
            return Err("source zeroing requires its actual compile-time declaration");
        }
        Ok(())
    }
}
