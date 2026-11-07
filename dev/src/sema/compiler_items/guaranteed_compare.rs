//! A pointer query descriptor authenticates declarations, never pointee access.
use super::Catalog;
use crate::{
    resolve::{body_origin::BodyOrigin, expansion::Report, model::DefId},
    sema::{header_attributes::ContractKnowledge, model::Declarations},
    support::bindings::SourceIntrinsic,
    types::{
        Kind, Primitive, TypeId, arguments::Argument, callable::Convention, instance::Instance,
    },
};

pub(crate) struct Selection {
    definition: DefId,
    signature: TypeId,
    pointer: TypeId,
}
impl Selection {
    pub(crate) const fn definition(&self) -> DefId {
        self.definition
    }
    pub(crate) const fn signature(&self) -> TypeId {
        self.signature
    }
    pub(crate) const fn pointer(&self) -> TypeId {
        self.pointer
    }

    pub(crate) fn opened_matches(&self, facts: &Declarations, signature: TypeId) -> bool {
        super::intrinsic_contracts::opened_signature_matches(
            &facts.types,
            signature,
            &[self.pointer; 2],
            TypeId::primitive(Primitive::U8),
            false,
            Convention::RUST,
        )
    }
}

pub(crate) fn is_operation(kind: SourceIntrinsic) -> bool {
    matches!(kind, SourceIntrinsic::Declaration(identity) if identity.name() == "ptr_guaranteed_cmp")
}
impl Catalog {
    pub(crate) fn guaranteed_compare(
        &self,
        facts: &Declarations,
        instance: Instance,
        origin: BodyOrigin,
    ) -> Result<Option<Selection>, &'static str> {
        let selected = self.guaranteed_instance(facts, instance)?;
        if selected.is_some() && origin.source().is_none() {
            return Err("source pointer comparison caller provenance is not transported");
        }
        Ok(selected)
    }
    pub(crate) fn guaranteed_fallback(
        &self,
        facts: &Declarations,
        instance: Instance,
    ) -> Result<bool, &'static str> {
        let Some(selected) = self.guaranteed_instance(facts, instance)? else {
            return Ok(false);
        };
        let function = facts
            .function_indices
            .get(&selected.definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source pointer comparison fallback lost its actual function")?;
        if function.body.is_none() {
            return Err("source pointer comparison requires its actual native fallback body");
        }
        Ok(true)
    }
    fn guaranteed_instance(
        &self,
        facts: &Declarations,
        instance: Instance,
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
        if !is_operation(kind) {
            return Ok(None);
        }
        crate::sema::header_attributes::charge_source_contract_query(facts, 16)
            .map_err(|_| "source pointer comparison selection work budget exhausted")?;
        if matches!(self.bindings, super::binding_views::Bindings::Imported(_)) {
            return Err("source pointer comparison declaration provenance is not transported");
        }
        let function = facts
            .function_indices
            .get(&definition)
            .and_then(|&index| facts.functions.get(index))
            .ok_or("source pointer comparison requires its completed actual function")?;
        if declaration.parent.is_some()
            || function.definition != definition
            || function.node != declaration.origin
            || function.signature != original
        {
            return Err("source pointer comparison completed declaration changed");
        }
        let mut args = facts
            .types
            .arguments(arguments)
            .ok_or("source pointer comparison has invalid instance arguments")?;
        let (Some(Argument::Type(element)), None) = (args.next(), args.next()) else {
            return Err("source pointer comparison requires its complete one-type instance");
        };
        if facts.types.contains_parameters(element) != Some(false)
            || facts.types.contains_projections(element) != Some(false)
            || facts.types.is_sized(element) != Some(true)
        {
            return Err("source pointer comparison requires its closed Sized element");
        }
        let Some(Kind::Function {
            bound_regions: 0,
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
        }) = facts.types.kind(signature)
        else {
            return Err("source pointer comparison changed its safe Rust signature");
        };
        let Some(&[first, second]) = facts.types.list(parameters) else {
            return Err("source pointer comparison requires two pointer inputs");
        };
        let [pointer, other, output] = super::intrinsic_contracts::item_roots(
            &facts.types,
            [first, second, result],
            |steps| {
                crate::sema::header_attributes::charge_source_contract_query(facts, steps)
                    .map_err(|_| "source pointer comparison instance work exhausted")
            },
        )?
        .ok_or("source pointer comparison lacks canonical opened roots")?;
        if pointer != other
            || output != TypeId::primitive(Primitive::U8)
            || facts.types.kind(pointer)
                != Some(Kind::Pointer {
                    mutable: false,
                    pointee: element,
                })
        {
            return Err("source pointer comparison requires (*const T,*const T)->u8");
        }
        Ok(Some(Selection {
            definition,
            signature,
            pointer,
        }))
    }
    pub(crate) fn require_guaranteed_report(
        &self,
        report: &Report,
        facts: &Declarations,
        selected: &Selection,
    ) -> Result<(), &'static str> {
        self.require_provider_report(report)?;
        let Some((declaration, kind, _)) = self.intrinsic_record(selected.definition)? else {
            return Err("source pointer comparison lost its attributed declaration");
        };
        if !is_operation(kind) {
            return Err("source pointer comparison changed its intrinsic identity");
        }
        let current = facts
            .header_attribute_contracts(report, selected.definition)
            .map_err(|_| "source pointer comparison lost its current source header")?;
        if current
            .origin
            .source()
            .is_none_or(|site| site.declaration != declaration.origin)
            || matches!(current.must_use, ContractKnowledge::Unknown)
            || matches!(current.inline, ContractKnowledge::Unknown)
            || !facts.is_const_function(selected.definition)
        {
            return Err(
                "source pointer comparison requires its actual completed const declaration",
            );
        }
        let ast = &report.package.unit(declaration.origin.file).configured.ast;
        if !matches!(ast.node(declaration.origin.node).kind,
        crate::syntax::ast::Kind::Item(index) if matches!(ast.items[index as usize].kind,
            crate::syntax::item::ItemKind::Function {
                flags: crate::syntax::item::FunctionFlags {
                    constant: true, asynchronous: false,
                    safety: crate::syntax::item::Safety::Default,
                }, abi: crate::syntax::item::Abi::Rust, ..
            }))
        {
            return Err("source pointer comparison changed its actual safe const Rust header");
        }
        Ok(())
    }
}
