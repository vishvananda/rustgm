//! Authenticate the checked headers shared by Iterator's by-value reductions.
use super::{Catalog, Declarations, DefId, ItemId, Kind, LangItem};
use crate::{
    sema::compiler_items::{const_payments::closed_clause, headers::Trait},
    support::constness::Constness,
    types::{
        GenericArgs, Primitive, TypeId,
        arguments::Argument,
        parameters::{
            ParameterKind,
            predicates::{Bound, Predicate},
        },
    },
};

#[derive(Clone, Copy)]
pub(super) enum Reduction {
    Fold,
    Last,
    Count,
}

struct Roles {
    sized: DefId,
    destruct: DefId,
    meta_sized: DefId,
}

impl Roles {
    fn of(catalog: &Catalog) -> Result<Self, &'static str> {
        Ok(Self {
            sized: actual(catalog, LangItem::Sized)?,
            destruct: actual(catalog, LangItem::Destruct)?,
            meta_sized: actual(catalog, LangItem::MetaSized)?,
        })
    }
}

impl Reduction {
    pub(super) const fn inputs(self) -> usize {
        match self {
            Self::Fold => 3,
            Self::Last | Self::Count => 1,
        }
    }

    pub(super) fn require(
        self,
        facts: &Declarations,
        catalog: &Catalog,
        iterator: &Trait<'_>,
        option: ItemId,
        member: DefId,
        (parameters, result): (&[TypeId], TypeId),
    ) -> Result<(), &'static str> {
        let [receiver, rest @ ..] = parameters else {
            return Err("source Iterator reduction lost its actual receiver");
        };
        if *receiver != iterator.self_type || parameters.len() != self.inputs() {
            return Err("source Iterator reduction requires its actual by-value Self input");
        }
        let next = catalog
            .role(LangItem::IteratorNext)?
            .ok_or("source Iterator reduction requires its actual next declaration")?;
        if next.parent != Some(iterator.definition) {
            return Err("source Iterator reduction changed its actual next owner");
        }
        let item = super::loop_headers::next_item(facts, next, iterator, option)?;
        let roles = Roles::of(catalog)?;
        let item = ItemProjection {
            owner: iterator.definition,
            subject: iterator.self_type,
            member: item,
        };
        match (self, rest) {
            (Self::Fold, &[accumulator, callback]) => {
                if result != accumulator {
                    return Err("source fold requires the actual accumulator as its result");
                }
                require_formals(facts, member, rest)?;
                Fold {
                    base: roles,
                    item,
                    accumulator,
                    callback,
                    callback_trait: actual(catalog, LangItem::FnMut)?,
                    once: actual(catalog, LangItem::FnOnce)?,
                    output: actual(catalog, LangItem::FnOnceOutput)?,
                }
                .require(facts, member)
            }
            (Self::Last, []) => {
                require_formals(facts, member, rest)?;
                if !option_item(facts, result, option, item) {
                    return Err("source Iterator reduction changed its actual result");
                }
                require_consuming(facts, &roles, member, item, "last")
            }
            (Self::Count, []) => {
                require_formals(facts, member, rest)?;
                if result != TypeId::primitive(Primitive::Usize) {
                    return Err("source Iterator reduction changed its actual result");
                }
                require_consuming(facts, &roles, member, item, "count")
            }
            _ => Err("source Iterator reduction changed its actual callable arity"),
        }
    }
}

fn require_formals(
    facts: &Declarations,
    member: DefId,
    expected: &[TypeId],
) -> Result<(), &'static str> {
    let formals = facts
        .generics
        .parameters
        .get(&member)
        .map_or(&[][..], Vec::as_slice);
    if formals.len() != expected.len() {
        return Err("source Iterator reduction changed its method-owned type formals");
    }
    for (formal, &ty) in formals.iter().zip(expected) {
        if formal.anonymous
            || formal.default.is_some()
            || formal.argument != Argument::Type(ty)
            || !matches!(facts.types.kind(ty), Some(Kind::Parameter(id))
                if facts.types.parameter(id).is_some_and(|formal|
                    formal.owner == ItemId(member.0) && formal.kind == ParameterKind::Type { sized: true }))
        {
            return Err("source fold requires its actual two sized type formals");
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
struct ItemProjection {
    owner: DefId,
    subject: TypeId,
    member: DefId,
}

impl ItemProjection {
    fn matches(self, facts: &Declarations, ty: TypeId) -> bool {
        matches!(facts.types.kind(ty), Some(Kind::Projection(projection))
            if projection.item == ItemId(self.member.0)
                && projection.subject == self.subject
                && projection.arguments == GenericArgs::EMPTY
                && facts.types.trait_application(projection.trait_application).is_some_and(|app|
                    app.item == ItemId(self.owner.0) && app.arguments == GenericArgs::EMPTY))
    }
}

fn actual(catalog: &Catalog, role: LangItem) -> Result<DefId, &'static str> {
    catalog
        .role(role)?
        .map(|actual| actual.definition)
        .ok_or("source Iterator reduction lost its actual dependency declaration")
}

fn application(
    facts: &Declarations,
    bound: Bound,
    expected: DefId,
    effect: Constness,
) -> Option<GenericArgs> {
    let (application, actual_effect) = bound.trait_parts()?;
    let application = facts.types.trait_application(application)?;
    (actual_effect == effect && application.item == ItemId(expected.0))
        .then_some(application.arguments)
}

fn sized(facts: &Declarations, bound: Bound, role: DefId) -> bool {
    bound == Bound::Sized
        || application(facts, bound, role, Constness::Ordinary) == Some(GenericArgs::EMPTY)
}

fn option_item(facts: &Declarations, ty: TypeId, option: ItemId, item: ItemProjection) -> bool {
    let Some(Kind::Adt(owner, args)) = facts.types.kind(ty) else {
        return false;
    };
    owner == option
        && facts.types.arguments(args).is_some_and(|mut args| {
            args.len() == 1
                && matches!(args.next(), Some(Argument::Type(ty)) if item.matches(facts, ty))
        })
}

fn require_consuming(
    facts: &Declarations,
    roles: &Roles,
    member: DefId,
    item: ItemProjection,
    name: &str,
) -> Result<(), &'static str> {
    let mut seen = [false; 3];
    for predicate in inputs(facts, member)? {
        let predicate = predicate?;
        let destruct = application(
            facts,
            predicate.bound,
            roles.destruct,
            Constness::Conditional,
        ) == Some(GenericArgs::EMPTY);
        let slot = if predicate.ty == item.subject && sized(facts, predicate.bound, roles.sized) {
            0
        } else if destruct && predicate.ty == item.subject {
            1
        } else if destruct && item.matches(facts, predicate.ty) {
            2
        } else {
            return Err(if name == "count" {
                "source count has an extra method premise"
            } else {
                "source last has an extra method premise"
            });
        };
        record(&mut seen[slot])?;
    }
    if seen.contains(&false) {
        return Err(if name == "count" {
            "source count omitted an actual Sized or Destruct premise"
        } else {
            "source last omitted an actual Sized or Destruct premise"
        });
    }
    Ok(())
}

struct Fold {
    base: Roles,
    item: ItemProjection,
    accumulator: TypeId,
    callback: TypeId,
    callback_trait: DefId,
    once: DefId,
    output: DefId,
}

impl Fold {
    fn require(&self, facts: &Declarations, member: DefId) -> Result<(), &'static str> {
        let mut seen = [false; 7];
        for predicate in inputs(facts, member)? {
            let slot = self.premise(facts, predicate?)?;
            record(&mut seen[slot])?;
        }
        if seen.contains(&false) {
            return Err("source fold omitted an actual sizing, destruction, or callable premise");
        }
        Ok(())
    }

    fn premise(&self, facts: &Declarations, predicate: Predicate) -> Result<usize, &'static str> {
        let Predicate { ty, bound } = predicate;
        if ty == self.item.subject && sized(facts, bound, self.base.sized) {
            return Ok(0);
        }
        if application(facts, bound, self.base.destruct, Constness::Conditional)
            == Some(GenericArgs::EMPTY)
        {
            if ty == self.item.subject {
                return Ok(1);
            }
            if ty == self.callback {
                return Ok(2);
            }
        }
        if ty == self.callback
            && application(facts, bound, self.callback_trait, Constness::Conditional)
                .is_some_and(|args| self.callable_arguments(facts, args))
        {
            return Ok(3);
        }
        if bound == Bound::ProjectionEqual(self.accumulator) && self.callable_result(facts, ty) {
            return Ok(4);
        }
        if application(facts, bound, self.base.meta_sized, Constness::Ordinary)
            == Some(GenericArgs::EMPTY)
        {
            if ty == self.accumulator {
                return Ok(5);
            }
            if ty == self.callback {
                return Ok(6);
            }
        }
        Err("source fold has an extra or altered method premise")
    }

    fn callable_arguments(&self, facts: &Declarations, args: GenericArgs) -> bool {
        let Some(mut args) = facts.types.arguments(args) else {
            return false;
        };
        if args.len() != 1 {
            return false;
        }
        let Some(Argument::Type(tuple)) = args.next() else {
            return false;
        };
        matches!(facts.types.kind(tuple), Some(Kind::Tuple(fields))
            if matches!(facts.types.list(fields), Some([accumulator, item])
                if *accumulator == self.accumulator && self.item.matches(facts, *item)))
    }

    fn callable_result(&self, facts: &Declarations, projection: TypeId) -> bool {
        matches!(facts.types.kind(projection), Some(Kind::Projection(projection))
            if projection.item == ItemId(self.output.0)
                && projection.subject == self.callback
                && projection.arguments == GenericArgs::EMPTY
                && facts.types.trait_application(projection.trait_application)
                    .is_some_and(|application| application.item == ItemId(self.once.0)
                        && self.callable_arguments(facts, application.arguments)))
    }
}

fn inputs(
    facts: &Declarations,
    member: DefId,
) -> Result<impl Iterator<Item = Result<Predicate, &'static str>> + '_, &'static str> {
    Ok(crate::sema::declaration_inputs::declared_own(
        &facts.generics,
        facts.crates.as_deref(),
        member,
    )?
    .iter()
    .map(|row| closed_clause(&facts.types, row.value)))
}

fn record(seen: &mut bool) -> Result<(), &'static str> {
    if std::mem::replace(seen, true) {
        return Err("source Iterator reduction repeats a method premise");
    }
    Ok(())
}
