//! The language's own `Clone` for tuples, closures and functions is a compiler shim.
use crate::{
    resolve::{body_origin::DeclarationOrigin, model::DefId},
    sema::model::Declarations,
    support::{bindings::LangItem, constness::Constness},
    types::{
        GenericArgs, ItemId, Kind, Store, TypeId,
        arguments::Argument,
        instance::Instance,
        parameters::predicates::{Bound as Requirement, Predicate},
        substitute::Bound,
        traits::Application,
    },
};

/// A source core writes no `Clone` implementation for these types.
fn provided(types: &Store, subject: TypeId) -> bool {
    matches!(
        types.kind(subject),
        Some(
            Kind::Tuple(_)
                | Kind::Closure { .. }
                | Kind::FunctionItem { .. }
                | Kind::Function { .. }
        )
    )
}

/// Classify an ordinary `Clone` goal of this source provider whose subject the
/// language implements. This is no proof: selection still decides the goal.
pub(in crate::sema) fn application(
    facts: &Declarations,
    predicate: Predicate,
) -> Result<bool, &'static str> {
    let Some((id, Constness::Ordinary)) = predicate.bound.trait_parts() else {
        return Ok(false);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(false);
    };
    let Some(owner) = catalog.role(LangItem::Clone)? else {
        return Ok(false);
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("invalid source Clone application")?;
    if application.item.0 != owner.definition.0 || !provided(&facts.types, predicate.ty) {
        return Ok(false);
    }
    catalog.validated_role(LangItem::Clone)?;
    Ok(true)
}

/// Exact canonical `clone` data for a compiler shim, never an ordinary impl body.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Shim {
    pub instance: Instance,
    pub definition: DefId,
    pub origin: DeclarationOrigin,
    pub signature: TypeId,
    pub subject: TypeId,
}

/// The shim's own proof universe: Self's lifetimes follow the receiver borrow.
pub(crate) struct Inputs {
    pub subject: TypeId,
    pub premise: crate::sema::callee_premises::Assumptions,
}

/// The stored members a shim clones in order; a function is copied whole.
pub(crate) fn members(types: &Store, subject: TypeId) -> Result<Option<Vec<TypeId>>, &'static str> {
    let fields = match types.kind(subject) {
        Some(Kind::Tuple(fields)) => fields,
        Some(Kind::Closure { environment, .. }) => match types.kind(environment) {
            Some(Kind::Tuple(fields)) => fields,
            _ => return Err("source Clone shim closure has no stored capture tuple"),
        },
        Some(Kind::FunctionItem { .. } | Kind::Function { .. }) => return Ok(None),
        _ => return Err("source Clone shim subject is not language provided"),
    };
    types
        .list(fields)
        .map(|fields| Some(fields.to_vec()))
        .ok_or("source Clone shim lost its member types")
}

impl Shim {
    /// The caller selected `Self: Clone`; its witnesses are the only premise.
    pub(crate) fn inputs(self, facts: &mut Declarations) -> Result<Inputs, &'static str> {
        let mut abstraction = Bound::universals(1);
        let subject = abstraction.apply(&mut facts.types, self.subject)?;
        let universals = u32::try_from(abstraction.universal_count())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or("too many source Clone shim lifetimes")?;
        let owner = facts
            .associated
            .trait_owner(self.definition)
            .ok_or("source Clone shim has no trait owner")?;
        let application = facts.types.intern_trait_application(Application {
            item: ItemId(owner.0),
            arguments: GenericArgs::EMPTY,
        })?;
        let premise = crate::sema::callee_premises::declared_condition(
            facts,
            Predicate {
                ty: subject,
                bound: Requirement::Trait(application),
            }
            .into(),
            universals,
        )?;
        Ok(Inputs { subject, premise })
    }
}

/// The actual declared `clone` member and its `Clone` owner, both validated.
fn member(facts: &Declarations) -> Result<Option<super::Role>, &'static str> {
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    let Some(member) = catalog.role(LangItem::CloneFn)? else {
        return Ok(None);
    };
    catalog.validated_role(LangItem::CloneFn)?;
    let owner = catalog
        .validated_role(LangItem::Clone)?
        .ok_or("source Clone shim requires its completed actual Clone owner")?;
    if member.parent != Some(owner.definition)
        || facts.associated.trait_owner(member.definition) != Some(owner.definition)
    {
        return Err("source Clone shim belongs to a different actual trait");
    }
    Ok(Some(member))
}

pub(crate) fn shim(
    facts: &mut Declarations,
    instance: Instance,
) -> Result<Option<Shim>, &'static str> {
    let Some(Kind::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(instance.ty())
    else {
        return Ok(None);
    };
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return Ok(None);
    };
    if catalog
        .role(LangItem::CloneFn)?
        .is_none_or(|member| member.definition.0 != item.0)
    {
        return Ok(None);
    }
    let mut arguments = facts
        .types
        .arguments(arguments)
        .ok_or("source Clone shim has invalid arguments")?;
    let (Some(Argument::Type(subject)), None) = (arguments.next(), arguments.next()) else {
        return Err("source Clone shim requires only its actual Self argument");
    };
    if !provided(&facts.types, subject) {
        return Ok(None);
    }
    if facts.types.contains_parameters(subject) != Some(false)
        || facts.types.contains_projections(subject) != Some(false)
    {
        return Err("source Clone shim requires a closed actual subject");
    }
    let member = member(facts)?.ok_or("source Clone shim lost its actual declaration")?;
    let function = facts
        .declaration_header(member.definition)?
        .ok_or("source Clone shim lost its actual header")?;
    if function.declaration != member.origin || function.body.is_some() {
        return Err("source Clone shim changed its required actual declaration");
    }
    let original = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("source Clone shim lacks its canonical callable")?
        .signature;
    if original != function.signature {
        return Err("source Clone shim canonical callable changed its declaration signature");
    }
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(4);
    let mut fold = crate::sema::dispatch::callable_substitution(facts, instance)?;
    if fold.apply(&mut facts.types, original)? != signature {
        return Err("source Clone shim signature differs from its actual scoped substitution");
    }
    Ok(Some(Shim {
        instance,
        definition: member.definition,
        origin: member.origin,
        signature,
        subject,
    }))
}

/// The declared `clone` at one Self: an identity, never permission to call it.
pub(crate) fn callable(
    facts: &mut Declarations,
    item: ItemId,
    subject: TypeId,
) -> Result<TypeId, &'static str> {
    let [parameter] = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("source Clone shim lacks its canonical parameters")?
    else {
        return Err("source Clone shim requires exactly its Self parameter");
    };
    let bindings = [(*parameter, Argument::Type(subject))];
    let template = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("source Clone shim lacks its canonical callable")?
        .signature;
    let signature =
        Bound::parameters(&facts.types, &bindings)?.apply(&mut facts.types, template)?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(subject)])?;
    facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })
}

/// Select the executable of a language-provided `Clone` member for `subject`,
/// the storage a dispatched Self denotes. A provided member keeps the trait's
/// own checked body; the required `clone` becomes this subject's shim.
pub(in crate::sema) fn select(
    facts: &mut Declarations,
    selected: TypeId,
    subject: TypeId,
) -> Result<TypeId, &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(selected) else {
        return Err("source Clone shim selection requires a function item");
    };
    let declared = member(facts)?.ok_or("source Clone shim requires its actual declaration")?;
    if declared.definition.0 != item.0 {
        let provided = facts
            .declaration_header(DefId(item.0))?
            .is_some_and(|function| function.body.is_some());
        return if provided {
            Ok(selected)
        } else {
            Err("language-provided Clone has no executable for this required member")
        };
    }
    let selected = callable(facts, item, subject)?;
    let instance = Instance::new(&facts.types, selected)?;
    shim(facts, instance)?
        .ok_or("source Clone shim requires a closed tuple, closure or function subject")?;
    Ok(selected)
}
