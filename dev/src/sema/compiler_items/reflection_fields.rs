//! Actual Field protocol authority and application-specific builtin evidence.
#[path = "reflection_fields/authentication.rs"]
mod authentication;
#[path = "reflection_fields/prerequisites.rs"]
mod prerequisites;
#[path = "reflection_fields/representing.rs"]
mod representing;
use super::{Declaration, Error, LangItem, Report};
use crate::{
    resolve::model::DefId,
    sema::{header_attributes, model::Declarations},
    types::{ItemId, TypeId, arena, parameters::predicates::Predicate, projection},
};
pub(in crate::sema) use prerequisites::requirements;
pub(crate) use representing::Coordinates;
pub(in crate::sema) use representing::Description;
use std::rc::Rc;

pub(in crate::sema::compiler_items) struct Contracts {
    arena: arena::Identity,
    declarations: [Declaration; 5],
    images: [header_attributes::SourceImage; 5],
}

/// The proof node owns ordinary prerequisites; this token binds its exact subject.
#[derive(Clone)]
pub(in crate::sema) struct Selection(Rc<Selected>);
struct Selected {
    contract: Rc<Contracts>,
    subject: TypeId,
    description: Description,
}

pub(in crate::sema) enum Readiness {
    Unrelated,
    Contracts(DefId),
    Fields(ItemId),
    Deferred,
    Absent,
    Ready(Description),
}

/// The checked caller supplies proof; these are the same representing coordinates.
pub(super) fn coordinates(facts: &Declarations, subject: TypeId) -> Result<Coordinates, String> {
    let contract = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.traits.as_deref())
        .and_then(|roles| roles.field.as_ref())
        .ok_or("field_offset requires the actual completed Field protocol")?;
    contract.current(facts)?;
    match representing::application(facts, subject)? {
        representing::Application::Ready(coordinates) => Ok(coordinates),
        _ => Err("field_offset requires its actual closed representing application".into()),
    }
}

pub(in crate::sema) fn is_result(facts: &Declarations, item: ItemId) -> bool {
    facts.source_bindings.as_deref().is_some_and(|catalog| {
        [LangItem::FieldBase, LangItem::FieldType]
            .into_iter()
            .any(|role| {
                catalog
                    .language(role)
                    .is_some_and(|d| d.definition.0 == item.0)
            })
    })
}

pub(in crate::sema) fn ready(facts: &Declarations, member: DefId) -> bool {
    facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| {
            catalog
                .traits
                .as_deref()
                .and_then(|roles| roles.field.as_ref())
        })
        .is_some_and(|contract| {
            contract.declarations[2..4]
                .iter()
                .any(|d| d.definition == member)
        })
}

pub(in crate::sema) fn inspect(
    facts: &mut Declarations,
    goal: Predicate,
) -> Result<Readiness, String> {
    if !matches_goal(facts, goal)? {
        return Ok(Readiness::Unrelated);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("Field lost its catalog")?;
    let Some(contract) = catalog
        .traits
        .as_deref()
        .and_then(|roles| roles.field.as_ref())
    else {
        let member = catalog
            .language(LangItem::FieldBase)
            .ok_or("Field requires its actual Base declaration")?;
        return Ok(Readiness::Contracts(member.definition));
    };
    contract.current(facts)?;
    representing::describe(facts, goal.ty)
}

pub(in crate::sema) fn matches_goal(facts: &Declarations, goal: Predicate) -> Result<bool, String> {
    let Some(declaration) = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| catalog.language(LangItem::Field))
    else {
        return Ok(false);
    };
    let Some((application, effect)) = goal.bound.trait_parts() else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(application)
        .ok_or("invalid Field application")?;
    if application.item.0 != declaration.definition.0 {
        return Ok(false);
    }
    if effect != crate::support::constness::Constness::Ordinary
        || facts
            .types
            .arguments(application.arguments)
            .is_none_or(|args| args.len() != 0)
    {
        return Err("Field requires its complete ordinary trait application".into());
    }
    Ok(true)
}

pub(in crate::sema) fn complete_selection(
    facts: &mut Declarations,
    goal: Predicate,
) -> Result<Option<Selection>, String> {
    if !matches_goal(facts, goal)? {
        return Ok(None);
    }
    let Readiness::Ready(description) = inspect(facts, goal)? else {
        return Err("Field proof lost its actual selected declaration field".into());
    };
    header_attributes::charge_source_retention(
        facts,
        1,
        size_of::<Selected>() + 2 * size_of::<usize>(),
    )
    .map_err(|e| e.to_string())?;
    let contract = facts
        .source_bindings
        .as_deref()
        .and_then(|catalog| {
            catalog
                .traits
                .as_deref()
                .and_then(|roles| roles.field.as_ref())
        })
        .ok_or("Field proof lost its completed protocol")?;
    Ok(Some(Selection(Rc::new(Selected {
        contract: Rc::clone(contract),
        subject: goal.ty,
        description,
    }))))
}

impl Selection {
    pub(in crate::sema) fn current(&self, facts: &Declarations) -> Result<(), String> {
        let actual = facts
            .source_bindings
            .as_deref()
            .and_then(|catalog| {
                catalog
                    .traits
                    .as_deref()
                    .and_then(|roles| roles.field.as_ref())
            })
            .ok_or("Field selection lost its actual source catalog")?;
        if !Rc::ptr_eq(actual, &self.0.contract) || facts.types.kind(self.0.subject).is_none() {
            return Err("Field selection belongs to another source owner or subject".into());
        }
        self.0.contract.current(facts)
    }

    /// A builtin implementation uses the real source default, including its type and body.
    pub(in crate::sema) fn provided_constant(
        &self,
        facts: &Declarations,
        member: DefId,
    ) -> Result<(), String> {
        self.current(facts)?;
        let parent = self.0.contract.declarations[1];
        if facts.associated.trait_owner(member) != Some(parent.definition)
            || facts.associated.owner(member) != facts.associated.owner(parent.definition)
        {
            return Err("Field constant has another actual declaration owner".into());
        }
        let initializer = facts
            .initializer_index(member)
            .and_then(|index| facts.initializers.get(index))
            .ok_or("Field constant requires its actual provided initializer")?;
        if initializer.definition != member
            || !matches!(initializer.kind, crate::sema::model::InitializerKind::Const)
            || (member == self.0.contract.declarations[4].definition
                && initializer.node != self.0.contract.declarations[4].origin)
        {
            return Err("Field constant changed its actual provided declaration".into());
        }
        Ok(())
    }

    pub(in crate::sema) fn output(
        &self,
        facts: &mut Declarations,
        application: projection::Application,
    ) -> Result<TypeId, String> {
        self.current(facts)?;
        let parent = facts
            .types
            .trait_application(application.trait_application)
            .ok_or("Field result lost its parent application")?;
        if application.subject != self.0.subject
            || parent.item.0 != self.0.contract.declarations[1].definition.0
            || facts
                .types
                .arguments(parent.arguments)
                .is_none_or(|args| args.len() != 0)
            || facts
                .types
                .arguments(application.arguments)
                .is_none_or(|args| args.len() != 0)
        {
            return Err("Field result differs from its proved full application".into());
        }
        let Readiness::Ready(description) = representing::describe(facts, self.0.subject)? else {
            return Err("Field result lost its actual semantic field".into());
        };
        if description != self.0.description {
            return Err("Field result changed its selected semantic coordinate/type".into());
        }
        if application.item.0 == self.0.contract.declarations[2].definition.0 {
            Ok(description.base)
        } else if application.item.0 == self.0.contract.declarations[3].definition.0 {
            Ok(description.field)
        } else {
            Err("Field evidence names another associated member".into())
        }
    }
}

pub(in crate::sema) fn prepare(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    authentication::prepare(report, facts)
}

pub(in crate::sema) fn prepared_member(
    report: &Report,
    facts: &Declarations,
    member: DefId,
) -> Result<bool, Error> {
    if !ready(facts, member) {
        return Ok(false);
    }
    authenticate(report, facts)?;
    Ok(true)
}

/// Facts-only proof replay retains its seal; actual phase boundaries check the Report.
pub(crate) fn authenticate(report: &Report, facts: &Declarations) -> Result<(), Error> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(());
    };
    let Some(contract) = catalog
        .traits
        .as_deref()
        .and_then(|roles| roles.field.as_ref())
    else {
        return Ok(());
    };
    catalog
        .require_provider_report(report)
        .map_err(Error::unlocated)?;
    contract.current(facts).map_err(Error::unlocated)?;
    for (declaration, expected) in contract.declarations.iter().zip(&contract.images) {
        let actual = header_attributes::role_source_image(facts, report, *declaration)?;
        if !expected.same(&actual) {
            return Err(Error::unlocated(
                "Field source belongs to another recorded incarnation",
            ));
        }
    }
    Ok(())
}

/// Ordinary bodies may seal completed declaration data at their actual first use.
/// Earlier declaration/executable work still yields through the owning worklist.
pub(in crate::sema) fn prepare_result_dependency(
    report: &Report,
    facts: &mut Declarations,
    member: DefId,
    availability: crate::sema::traits::implementations::Availability,
) -> Result<bool, Error> {
    if availability != crate::sema::traits::implementations::Availability::Validated
        || !is_result(facts, ItemId(member.0))
    {
        return Ok(false);
    }
    if ready(facts, member) {
        return Err(Error::unlocated(
            "Field result readiness repeated after completion",
        ));
    }
    prepare(report, facts)?;
    if !prepared_member(report, facts, member)? {
        return Err(Error::unlocated(
            "Field result readiness did not complete its actual member",
        ));
    }
    Ok(true)
}
