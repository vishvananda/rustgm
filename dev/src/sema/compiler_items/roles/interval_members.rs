//! A complete source range family seals declaration identity, never endpoint execution.
use super::{Declaration, Declarations, DefId, Error, LangItem, Report};
use crate::{
    resolve::model::DefKind,
    sema::{
        compiler_items::headers::{Flags, Trait},
        header_attributes::{self, SourceImage},
    },
    syntax::{ast::Kind as Syntax, item::ItemKind},
    types::{Kind, TypeId, arena, callable::Convention},
};
pub(in crate::sema::compiler_items) const ROLES: [LangItem; 3] =
    [LangItem::RangeMin, LangItem::RangeMax, LangItem::RangeSub];

pub(in crate::sema::compiler_items) struct Contracts {
    arena: arena::Identity,
    pub(in crate::sema::compiler_items) parent: Declaration,
    pub(in crate::sema::compiler_items) members: [Declaration; 3],
    pub(in crate::sema::compiler_items) self_type: TypeId,
    pub(in crate::sema::compiler_items) signature: TypeId,
    images: [SourceImage; 4],
}
struct Plan {
    parent: Declaration,
    members: [Declaration; 3],
    self_type: TypeId,
    signature: TypeId,
}

pub(super) fn prepare(
    report: &Report,
    facts: &mut Declarations,
) -> Result<Option<Box<Contracts>>, Error> {
    let Some(plan) = plan(report, facts)? else {
        return Ok(None);
    };
    let arena_bytes = usize::from(facts.types.existing_arena_identity().is_none())
        * arena::Identity::allocation_bytes();
    header_attributes::reserve_source_contracts(
        facts,
        5,
        size_of::<Contracts>().saturating_add(arena_bytes),
    )?;
    let arena = facts.types.arena_identity();
    let images = [
        header_attributes::completed_source_image(facts, report, plan.parent.definition),
        header_attributes::pointee_source_image(facts, report, plan.members[0]),
        header_attributes::pointee_source_image(facts, report, plan.members[1]),
        header_attributes::completed_source_image(facts, report, plan.members[2].definition),
    ];
    let [a, b, c, d] = images;
    let contracts = Box::new(Contracts {
        arena,
        parent: plan.parent,
        members: plan.members,
        self_type: plan.self_type,
        signature: plan.signature,
        images: [a?, b?, c?, d?],
    });
    contracts.current(report, facts)?;
    Ok(Some(contracts))
}
fn plan(report: &Report, facts: &Declarations) -> Result<Option<Plan>, Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    header_attributes::charge_source_contract_query(facts, 4)?;
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    // A valid unused partial family is pending; it cannot confer a completed use.
    let [Some(min), Some(max), Some(sub)] = ROLES.map(|r| catalog.language(r)) else {
        return Ok(None);
    };
    let fail = |message| Declarations::error(report, min.origin, message);
    let parent = min
        .parent
        .ok_or_else(|| fail("source range member has no actual trait parent"))?;
    let owner = facts
        .associated
        .owner(parent)
        .and_then(|id| facts.associated.get(id))
        .ok_or_else(|| fail("source range family lacks its actual trait owner"))?;
    let (header, _) = Trait::source_owner_with_bounds(report, facts, owner).map_err(fail)?;
    header
        .profile(
            &facts.types,
            Flags {
                unsafe_: false,
                constant: true,
                auto_: false,
            },
            0,
        )
        .map_err(fail)?;
    let members = [min, max, sub];
    if header.definition != parent
        || header.written.parameters != 0
        || header.written.supertraits != 0
        || header.members.len() != 3
        || !members
            .iter()
            .all(|m| m.parent == Some(parent) && header.members.contains(&m.definition))
    {
        return Err(fail(
            "source range family requires its three actual common-owner members",
        ));
    }
    for member in [min, max] {
        constant(report, facts, member, header.self_type).map_err(fail)?;
    }
    let signature = method(report, facts, sub, parent, header.self_type).map_err(fail)?;
    let origin = owner.source_owner().map_err(fail)?.source;
    Ok(Some(Plan {
        parent: Declaration {
            definition: parent,
            parent: None,
            origin,
        },
        members,
        self_type: header.self_type,
        signature,
    }))
}
fn constant(
    report: &Report,
    facts: &Declarations,
    member: Declaration,
    self_type: TypeId,
) -> Result<(), &'static str> {
    let ast = &report.package.unit(member.origin.file).configured.ast;
    let Syntax::Item(index) = ast.node(member.origin.node).kind else {
        return Err("source range constant has no actual item");
    };
    let item = ast.items[index as usize];
    if !matches!(item.kind, ItemKind::Const { value: None, .. })
        || !ast.list(item.generics).is_empty()
        || !ast.list(item.where_clause).is_empty()
    {
        return Err("source range constant requires the actual required Self declaration");
    }
    let ty = facts
        .definitions
        .get(member.definition.0 as usize)
        .copied()
        .flatten()
        .ok_or("source range constant declaration type is not prepared")?;
    if ty != self_type {
        return Err("source range constant requires its actual trait Self type");
    }
    Ok(())
}
fn method(
    report: &Report,
    facts: &Declarations,
    member: Declaration,
    parent: DefId,
    self_type: TypeId,
) -> Result<TypeId, &'static str> {
    let ast = &report.package.unit(member.origin.file).configured.ast;
    let Syntax::Item(index) = ast.node(member.origin.node).kind else {
        return Err("source RangeSub has no actual method");
    };
    let item = ast.items[index as usize];
    let function = facts
        .declaration_header(member.definition)?
        .ok_or("source RangeSub requires its completed method signature")?;
    if member.parent != Some(parent)
        || function.declaration != member.origin.into()
        || !matches!(item.kind, ItemKind::Function { body: None, .. })
        || !ast.list(item.generics).is_empty()
        || !ast.list(item.where_clause).is_empty()
    {
        return Err("source RangeSub requires its actual required nongeneric method");
    }
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST,
    }) = facts.types.kind(function.signature)
    else {
        return Err("source RangeSub requires its safe Rust Self signature");
    };
    if result != self_type || facts.types.list(parameters) != Some(&[self_type]) {
        return Err("source RangeSub requires one by-value Self input and Self result");
    }
    Ok(function.signature)
}
impl Contracts {
    pub(in crate::sema::compiler_items) fn current(
        &self,
        report: &Report,
        facts: &Declarations,
    ) -> Result<(), Error> {
        header_attributes::charge_source_contract_query(facts, 16)?;
        let fail = Error::unlocated;
        let catalog = facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| fail("source range family lost its actual catalog"))?;
        catalog.require_provider_report(report).map_err(fail)?;
        if !facts.types.has_identity(&self.arena) {
            return Err(fail("source range family belongs to another type Store"));
        }
        let owner = facts
            .associated
            .owner(self.parent.definition)
            .and_then(|id| facts.associated.get(id))
            .ok_or_else(|| fail("source range family lost its actual owner"))?;
        let schema = facts
            .associated
            .schema(self.parent.definition)
            .ok_or_else(|| fail("source range family lost its actual Self schema"))?;
        if owner.source != Some(self.parent.origin)
            || schema.self_type != self.self_type
            || schema.captures != 0
            || facts
                .associated
                .owner_members(
                    facts
                        .associated
                        .owner(self.parent.definition)
                        .ok_or_else(|| fail("source range parent missing"))?,
                )
                .is_none_or(|members| members.len() != 3)
        {
            return Err(fail(
                "source range family changed its actual parent or Self",
            ));
        }
        super::super::headers::require_self(&facts.types, self.self_type, owner.self_definition)
            .map_err(fail)?;
        for (index, member) in self.members.iter().enumerate() {
            if catalog.language(ROLES[index]) != Some(*member)
                || facts.associated.trait_owner(member.definition) != Some(self.parent.definition)
                || facts.generics.parents.get(&member.definition) != Some(&self.parent.definition)
            {
                return Err(fail("source range member changed its actual role or owner"));
            }
        }
        self.current_images(report, facts)?;
        self.current_types(facts)
    }
    fn current_images(&self, report: &Report, facts: &Declarations) -> Result<(), Error> {
        let fail = Error::unlocated;
        for (index, declaration) in [
            self.parent,
            self.members[0],
            self.members[1],
            self.members[2],
        ]
        .into_iter()
        .enumerate()
        {
            let definition = report
                .resolution
                .definitions
                .get(declaration.definition.0 as usize)
                .ok_or_else(|| fail("source range declaration missing"))?;
            let expected = if index == 0 {
                DefKind::Trait
            } else if index < 3 {
                DefKind::Const
            } else {
                DefKind::Function
            };
            let image = if index == 1 || index == 2 {
                header_attributes::pointee_source_image(facts, report, declaration)?
            } else {
                header_attributes::completed_source_image(facts, report, declaration.definition)?
            };
            if definition.node != Some(declaration.origin)
                || definition.kind != expected
                || !self.images[index].same(&image)
                || !self.images[index].current(
                    report.package.unit(declaration.origin.file),
                    declaration.origin.node,
                )
            {
                return Err(fail(
                    "source range declaration changed its completed source header",
                ));
            }
        }
        Ok(())
    }
    fn current_types(&self, facts: &Declarations) -> Result<(), Error> {
        let fail = Error::unlocated;
        for member in &self.members[..2] {
            if facts
                .definitions
                .get(member.definition.0 as usize)
                .copied()
                .flatten()
                != Some(self.self_type)
            {
                return Err(fail(
                    "source range constant changed its completed Self type",
                ));
            }
        }
        let sub = facts
            .declaration_header(self.members[2].definition)
            .map_err(fail)?
            .ok_or_else(|| fail("source RangeSub signature missing"))?;
        if sub.signature != self.signature || sub.declaration != self.members[2].origin.into() {
            return Err(fail("source RangeSub changed its completed signature"));
        }
        Ok(())
    }
}
