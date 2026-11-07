//! Capture the real Field family after its ordinary source result producers finish.
use super::{Contracts, Declaration, Declarations, Error, LangItem, Rc, Report, header_attributes};
use crate::{
    resolve::model::{DefId, DefKind},
    sema::compiler_items::headers::Trait,
    types::{
        ItemId, Kind, Primitive, TypeId,
        arguments::{Argument, ConstKind},
        parameters::ParameterKind,
    },
};

const ROLES: [LangItem; 5] = [
    LangItem::FieldRepresentingType,
    LangItem::Field,
    LangItem::FieldBase,
    LangItem::FieldType,
    LangItem::FieldOffset,
];

pub(super) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or_else(|| Error::unlocated("Field requires source declarations"))?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    if let Some(contract) = catalog
        .traits
        .as_deref()
        .and_then(|roles| roles.field.as_ref())
    {
        return contract.current(facts).map_err(Error::unlocated);
    }
    let needs_ledger = catalog.traits.is_none();
    let declarations = ROLES.map(|role| catalog.language(role));
    let [
        Some(nominal),
        Some(trait_),
        Some(base),
        Some(field),
        Some(offset),
    ] = declarations
    else {
        return Err(Error::unlocated(
            "demanded Field requires its actual nominal/trait/member family",
        ));
    };
    let declarations = [nominal, trait_, base, field, offset];
    header_attributes::charge_source_retention(facts, 10, 0)?;
    let header = Trait::source(report, facts, trait_).map_err(Error::unlocated)?;
    header.canonical_inputs(facts).map_err(Error::unlocated)?;
    if !header.parameters.is_empty() || header.captures != 0 || header.flags.auto_ {
        return Err(Error::unlocated(
            "Field requires a real non-generic ordinary trait owner",
        ));
    }
    for member in [base, field, offset] {
        if !header.members.contains(&member.definition) || member.parent != Some(trait_.definition)
        {
            return Err(Error::unlocated(
                "demanded Field members belong to another actual trait",
            ));
        }
    }
    validate_nominal(facts, nominal.definition).map_err(Error::unlocated)?;
    validate_targets(report, &declarations)?;
    let arena_bytes = usize::from(facts.types.existing_arena_identity().is_none())
        * crate::types::arena::Identity::allocation_bytes();
    header_attributes::charge_source_retention(
        facts,
        5,
        size_of::<Contracts>()
            + 2 * size_of::<usize>()
            + arena_bytes
            + usize::from(needs_ledger) * size_of::<super::super::roles::Validated>(),
    )?;
    let contract = Rc::new(Contracts {
        arena: facts.types.arena_identity(),
        images: [
            header_attributes::role_source_image(facts, report, nominal)?,
            header_attributes::role_source_image(facts, report, trait_)?,
            header_attributes::role_source_image(facts, report, base)?,
            header_attributes::role_source_image(facts, report, field)?,
            header_attributes::role_source_image(facts, report, offset)?,
        ],
        declarations,
    });
    contract.current(facts).map_err(Error::unlocated)?;
    let catalog = facts
        .source_bindings
        .as_deref_mut()
        .ok_or_else(|| Error::unlocated("Field catalog disappeared"))?;
    let ledger = catalog.traits.get_or_insert_with(|| {
        Box::new(super::super::roles::Validated {
            roles: super::super::roles::RoleSet::EMPTY,
            methods: [None; 3],
            pointee: None,
            discriminant: None,
            range: None,
            field: None,
        })
    });
    ledger.field = Some(contract);
    Ok(())
}

fn validate_targets(report: &Report, declarations: &[Declaration; 5]) -> Result<(), Error> {
    for (index, declaration) in declarations.iter().copied().enumerate() {
        let expected = match index {
            0 => DefKind::Struct,
            1 => DefKind::Trait,
            2 | 3 => DefKind::TypeAlias,
            _ => DefKind::Const,
        };
        if report
            .resolution
            .definitions
            .get(declaration.definition.0 as usize)
            .is_none_or(|actual| actual.node != Some(declaration.origin) || actual.kind != expected)
        {
            return Err(Error::unlocated(
                "Field family changed its actual source definition",
            ));
        }
    }
    Ok(())
}

fn validate_nominal(facts: &Declarations, definition: DefId) -> Result<(), &'static str> {
    if !matches!(facts.definitions.get(definition.0 as usize).copied().flatten().and_then(|ty|facts.types.kind(ty)), Some(Kind::Adt(item,_))if item.0==definition.0)
        || !facts.generics.nominal_contracts.contains(&definition)
    {
        return Err("Field representing type requires its real completed nominal contract");
    }
    let parameters = facts
        .generics
        .parameters
        .get(&definition)
        .ok_or("Field nominal lost its formals")?;
    let [ty, variant, field] = parameters.as_slice() else {
        return Err("Field nominal requires type/u32/u32 formals");
    };
    let Argument::Type(ty) = ty.argument else {
        return Err("Field base formal is not a type");
    };
    let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
        return Err("Field base formal lost its identity");
    };
    if facts.types.parameter(id).is_none_or(|p| {
        p.owner.0 != definition.0 || p.index != 0 || !matches!(p.kind, ParameterKind::Type { .. })
    }) {
        return Err("Field base formal changed its actual owner/position");
    }
    for (index, parameter) in [variant, field].into_iter().enumerate() {
        let Argument::Const(id) = parameter.argument else {
            return Err("Field coordinate formal is not const");
        };
        let Some(ConstKind::Parameter(id)) = facts.types.const_kind(id) else {
            return Err("Field coordinate formal lost its identity");
        };
        if facts.types.parameter(id).is_none_or(|p| {
            p.owner.0 != definition.0
                || p.index as usize != index + 1
                || p.kind
                    != (ParameterKind::Const {
                        ty: TypeId::primitive(Primitive::U32),
                    })
        }) {
            return Err("Field coordinate formal requires its actual owned u32 kind/position");
        }
    }
    Ok(())
}

impl Contracts {
    pub(super) fn current(&self, facts: &Declarations) -> Result<(), String> {
        header_attributes::charge_source_contract_query(facts, 8).map_err(|e| e.to_string())?;
        if !facts.types.has_identity(&self.arena) {
            return Err("Field contract belongs to another type store".into());
        }
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or("Field contract lost its catalog")?;
        for ((role, declaration), image) in
            ROLES.into_iter().zip(self.declarations).zip(&self.images)
        {
            if catalog.language(role) != Some(declaration) {
                return Err("Field contract changed its actual role identities".into());
            }
            header_attributes::require_recorded_pointee_image(facts, declaration, image)
                .map_err(|e| e.to_string())?;
        }
        validate_nominal(facts, self.declarations[0].definition)?;
        canonical_header(facts, self.declarations[1])?;
        for declaration in &self.declarations[2..4] {
            validate_result(facts, *declaration, self.declarations[1].definition)?;
        }
        if facts
            .associated
            .trait_owner(self.declarations[4].definition)
            != Some(self.declarations[1].definition)
        {
            return Err("Field OFFSET changed its actual trait owner".into());
        }
        Ok(())
    }
}

fn validate_result(facts: &Declarations, member: Declaration, owner: DefId) -> Result<(), String> {
    let contract = facts
        .associated
        .require_projection_header(member.definition)?;
    let Some(Kind::Projection(projection)) = facts.types.kind(contract.projection) else {
        return Err("Field result lost its canonical projection".into());
    };
    if member.parent != Some(owner)
        || facts.associated.trait_owner(member.definition) != Some(owner)
        || facts.generics.parents.get(&member.definition) != Some(&owner)
        || facts
            .types
            .projection_schema(ItemId(member.definition.0))
            .is_none_or(|schema| schema.trait_item.0 != owner.0)
        || projection.item.0 != member.definition.0
        || facts
            .associated
            .schema(owner)
            .is_none_or(|schema| schema.self_type != projection.subject)
        || facts
            .types
            .trait_application(projection.trait_application)
            .is_none_or(|application| {
                application.item.0 != owner.0
                    || facts
                        .types
                        .arguments(application.arguments)
                        .is_none_or(|args| args.len() != 0)
            })
        || facts
            .types
            .arguments(projection.arguments)
            .is_none_or(|args| args.len() != 0)
    {
        return Err("Field result changed its full actual member/parent application".into());
    }
    for clause in &contract.bounds {
        use crate::types::parameters::{clauses::Clause, predicates::Bound};
        let Clause::Predicate(predicate) = *clause else {
            return Err(
                "Field quantified result requirements need their complete scoped proof".into(),
            );
        };
        let sized =
            match predicate.bound {
                Bound::Sized => true,
                Bound::Trait(application) => facts
                    .types
                    .trait_application(application)
                    .is_some_and(|application| {
                        facts
                            .source_bindings
                            .as_deref()
                            .and_then(|catalog| catalog.language(LangItem::Sized))
                            .is_some_and(|sized| sized.definition.0 == application.item.0)
                            && facts
                                .types
                                .arguments(application.arguments)
                                .is_some_and(|args| args.len() == 0)
                    }),
                _ => false,
            };
        if predicate.ty != contract.projection || !sized {
            return Err(
                "Field additional result requirements need their complete scoped proof".into(),
            );
        }
    }
    Ok(())
}

/// Canonical declaration inputs precede synthesized caller assumptions.
pub(super) fn canonical_header(
    facts: &Declarations,
    declaration: Declaration,
) -> Result<&crate::sema::traits::Header, String> {
    header_attributes::charge_source_contract_query(facts, 6).map_err(|error| error.to_string())?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .and_then(|id| facts.associated.get(id))
        .ok_or("Field lost its actual trait owner")?;
    if !matches!(owner.kind,crate::sema::associated::OwnerKind::Trait{definition,auto_:false,..}
        if definition==declaration.definition)
        || owner.source != Some(declaration.origin)
    {
        return Err("Field canonical inputs belong to another source owner".into());
    }
    let schema = facts
        .associated
        .schema(declaration.definition)
        .ok_or("Field lost its actual Self schema")?;
    let header = facts
        .generics
        .traits
        .as_deref()
        .and_then(|contracts| contracts.header(ItemId(declaration.definition.0)))
        .ok_or("Field lost its canonical declaration inputs")?;
    header.inputs.source_ref()?;
    if schema.self_type != header.self_type
        || schema.captures != 0
        || !header.parameters.is_empty()
        || facts
            .generics
            .parameters
            .get(&declaration.definition)
            .is_some_and(|parameters| !parameters.is_empty())
        || facts.generics.parents.get(&declaration.definition) != Some(&owner.self_definition)
    {
        return Err(
            "Field canonical inputs changed their actual Self/formal/capture schema".into(),
        );
    }
    crate::sema::compiler_items::headers::require_self(
        &facts.types,
        header.self_type,
        owner.self_definition,
    )?;
    let parameters = facts
        .generics
        .parameters
        .get(&owner.self_definition)
        .ok_or("Field lost its owned Self binding")?;
    let [parameter] = parameters.as_slice() else {
        return Err("Field requires its one actual Self binding".into());
    };
    if parameter.binding != owner.self_definition
        || parameter.anonymous
        || parameter.default.is_some()
        || parameter.argument != Argument::Type(header.self_type)
    {
        return Err("Field canonical inputs changed their actual Self binding".into());
    }
    Ok(header)
}
