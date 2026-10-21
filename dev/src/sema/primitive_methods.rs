//! Primitive method descriptors are language evidence, never invented source impls.
use super::model::Declarations;
use crate::{
    lex::token::Punctuation,
    resolve::{model::DefId, unit::NodeKey},
    support::language_hooks::Hook,
    types::{
        Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        callable::Convention,
        infer::{self, Constructor, Context, predicates::Predicate},
        instance::Instance,
    },
};
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../tests/invariants/primitive_factories.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Rhs {
    Owned,
    SharedReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Descriptor {
    hook: Hook,
    left: Primitive,
    right: Primitive,
    rhs: Rhs,
}
impl Descriptor {
    pub(crate) const fn hook(self) -> Hook {
        self.hook
    }
    pub(crate) const fn left(self) -> Primitive {
        self.left
    }
    pub(crate) const fn right(self) -> Primitive {
        self.right
    }
    pub(crate) const fn rhs(self) -> Rhs {
        self.rhs
    }
    pub(crate) fn operator(self) -> Result<Punctuation, &'static str> {
        operator(self.hook)
    }
}
fn operator(hook: Hook) -> Result<Punctuation, &'static str> {
    Ok(match hook {
        Hook::AddAssign => Punctuation::Plus,
        Hook::SubAssign => Punctuation::Minus,
        Hook::MulAssign => Punctuation::Star,
        Hook::DivAssign => Punctuation::Slash,
        Hook::RemAssign => Punctuation::Percent,
        Hook::BitAndAssign => Punctuation::And,
        Hook::BitOrAssign => Punctuation::Or,
        Hook::BitXorAssign => Punctuation::Caret,
        Hook::ShlAssign => Punctuation::ShiftLeft,
        Hook::ShrAssign => Punctuation::ShiftRight,
        _ => return Err("primitive descriptor is not an assignment operation"),
    })
}

pub(in crate::sema) enum Candidate {
    Unrelated,
    Deferred,
    Present(Descriptor),
}
pub(in crate::sema) fn candidate(
    facts: &Declarations,
    context: &mut Context,
    goal: &Predicate,
) -> Result<Candidate, infer::Error> {
    let Some((item, arguments)) = goal.trait_arguments() else {
        return Ok(Candidate::Unrelated);
    };
    let Some((hook, _)) = facts
        .associated
        .operation(DefId(item.0))
        .filter(|(kind, _)| kind.assignment())
    else {
        return Ok(Candidate::Unrelated);
    };
    let [crate::types::infer::arguments::Argument::Type(right)] = arguments else {
        return Err(infer::Error::InvalidIdentity);
    };
    let left = match context.head(&facts.types, goal.subject())? {
        Some(Constructor::Primitive(primitive)) => Some(primitive),
        None => None,
        _ => return Ok(Candidate::Unrelated),
    };
    let (right, rhs) = match context.head(&facts.types, *right)? {
        Some(Constructor::Primitive(primitive)) => (Some(primitive), Rhs::Owned),
        Some(Constructor::Reference { mutable: false, .. }) => {
            let referent = context.child(&facts.types, *right, 0)?;
            match context.head(&facts.types, referent)? {
                Some(Constructor::Primitive(primitive)) => (Some(primitive), Rhs::SharedReference),
                None => (None, Rhs::SharedReference),
                _ => return Ok(Candidate::Unrelated),
            }
        }
        None => (None, Rhs::Owned),
        _ => return Ok(Candidate::Unrelated),
    };
    // A rigid nonprimitive operand excludes builtin evidence even if the other
    // operand is still an unsuffixed scalar. Preserve a selected user impl.
    let (Some(left), Some(right)) = (left, right) else {
        return Ok(Candidate::Deferred);
    };
    let op = operator(hook).map_err(|_| infer::Error::InvalidIdentity)?;
    Ok(if super::scalar::binary(op, left, right) {
        Candidate::Present(Descriptor {
            hook,
            left,
            right,
            rhs,
        })
    } else {
        Candidate::Unrelated
    })
}

#[derive(Clone, Copy)]
pub(crate) struct Selected {
    instance: Instance,
    descriptor: Descriptor,
}
pub(crate) struct Validated {
    pub definition: DefId,
    pub source: NodeKey,
    pub signature: TypeId,
    pub receiver: TypeId,
    pub right: TypeId,
}
impl Selected {
    pub(crate) const fn instance(self) -> Instance {
        self.instance
    }
    pub(crate) const fn descriptor(self) -> Descriptor {
        self.descriptor
    }
    pub(crate) fn validate(self, facts: &mut Declarations) -> Result<Validated, &'static str> {
        let Some(Kind::FunctionItem {
            item,
            signature,
            arguments,
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("primitive method requires a function item");
        };
        let definition = DefId(item.0);
        let owner = facts
            .associated
            .trait_owner(definition)
            .ok_or("primitive method has no canonical trait owner")?;
        if facts.associated.operation(owner) != Some((self.descriptor.hook(), definition)) {
            return Err("primitive descriptor differs from canonical method identity");
        }
        let source = facts
            .associated
            .language_origin(owner)
            .ok_or("primitive method has no library origin")?;
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
            bound_regions: 1,
        }) = facts.types.kind(signature)
        else {
            return Err("primitive method signature permissions differ from its contract");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("missing primitive method parameters")?;
        let [receiver, _] = parameters else {
            return Err("primitive method has incorrect arity");
        };
        let receiver = *receiver;
        let Some(Kind::Reference {
            referent,
            mutable: true,
            region,
        }) = facts.types.kind(receiver)
        else {
            return Err("primitive method requires a mutable receiver");
        };
        if referent != TypeId::primitive(self.descriptor.left)
            || region != RegionId::bound(0, 0)?
            || result != TypeId::UNIT
        {
            return Err("primitive method receiver or result differs from descriptor");
        }
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("missing primitive method arguments")?
            .collect::<Vec<_>>();
        let [Argument::Type(subject), Argument::Type(right)] = arguments.as_slice() else {
            return Err("primitive method has incorrect generic arguments");
        };
        let (subject, right) = (*subject, *right);
        let expected = TypeId::primitive(self.descriptor.right);
        let actual = match self.descriptor.rhs {
            Rhs::Owned => right,
            Rhs::SharedReference => match facts.types.kind(right) {
                Some(Kind::Reference {
                    referent,
                    mutable: false,
                    ..
                }) => referent,
                _ => return Err("primitive forwarding method requires a shared RHS"),
            },
        };
        if actual != expected
            || subject != referent
            || !super::scalar::binary(
                self.descriptor.operator()?,
                self.descriptor.left,
                self.descriptor.right,
            )
        {
            return Err("primitive method arguments differ from selected descriptor");
        }
        validate_substitution(facts, item, signature, &arguments)?;
        Ok(Validated {
            definition,
            source,
            signature,
            receiver,
            right,
        })
    }
}
fn validate_substitution(
    facts: &mut Declarations,
    item: crate::types::ItemId,
    signature: TypeId,
    arguments: &[Argument],
) -> Result<(), &'static str> {
    let callable = facts
        .lifetimes
        .callables
        .get(item)
        .ok_or("primitive method has no canonical callable template")?;
    let template = callable.signature;
    let parameters = facts
        .lifetimes
        .callables
        .generic_parameters(item)
        .ok_or("primitive method has no canonical generic parameters")?;
    if callable.parameters != 0 || parameters.len() != arguments.len() {
        return Err("primitive method generic arguments differ from callable template");
    }
    let bindings = parameters
        .iter()
        .copied()
        .zip(arguments.iter().copied())
        .collect::<Vec<_>>();
    let mut substitution = crate::types::substitute::Bound::parameters(&facts.types, &bindings)?;
    if substitution.apply(&mut facts.types, template)? != signature {
        return Err("primitive method signature differs from canonical scoped substitution");
    }
    Ok(())
}
#[derive(Default)]
pub(crate) struct Registry {
    selected: HashMap<Instance, Selected>,
}
impl Registry {
    pub(crate) fn get(&self, instance: Instance) -> Option<Selected> {
        self.selected.get(&instance).copied()
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }
}
pub(in crate::sema) fn retain(
    facts: &mut Declarations,
    instance: Instance,
    descriptor: Descriptor,
) -> Result<(), &'static str> {
    let selected = Selected {
        instance,
        descriptor,
    };
    selected.validate(facts)?;
    let registry = facts.primitive_methods.get_or_insert_default();
    if registry.selected.len() >= 100_000 {
        return Err("primitive method selection count exceeds 100000");
    }
    if registry
        .selected
        .get(&instance)
        .is_some_and(|old| old.descriptor != descriptor)
    {
        return Err("primitive method instance has conflicting descriptors");
    }
    registry.selected.insert(instance, selected);
    Ok(())
}
