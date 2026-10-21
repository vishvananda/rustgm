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
mod domains;
use domains::comparison_candidate;
use std::collections::HashMap;
#[cfg(test)]
#[path = "../../tests/invariants/primitive_factories.rs"]
mod tests;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Form {
    Assignment,
    ValueBinary,
    Comparison,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Rhs {
    Owned,
    SharedReference,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Operation {
    Arithmetic(Hook),
    Comparison(crate::support::comparisons::Comparison),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Descriptor {
    operation: Operation,
    form: Form,
    left: Primitive,
    right: Primitive,
    rhs: Rhs,
}
impl Descriptor {
    pub(crate) const fn form(self) -> Form {
        self.form
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
        match self.operation {
            Operation::Arithmetic(hook) => operator(hook),
            Operation::Comparison(_) => Ok(Punctuation::EqualEqual),
        }
    }
    fn member(
        self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<(NodeKey, Punctuation), &'static str> {
        let owner = facts
            .associated
            .trait_owner(definition)
            .ok_or("primitive method has no canonical trait owner")?;
        let operator = match self.operation {
            Operation::Arithmetic(hook)
                if facts.associated.operation(owner) == Some((hook, definition)) =>
            {
                operator(hook)?
            }
            Operation::Comparison(kind) if facts.associated.comparison(owner) == Some(kind) => {
                use crate::support::comparisons::Method;
                let method = facts
                    .associated
                    .comparison_method(definition)
                    .ok_or("comparison member is not canonical")?;
                if method.owner() != kind {
                    return Err("comparison member has different trait owner");
                }
                match method {
                    Method::Eq | Method::PartialCmp | Method::Cmp => Punctuation::EqualEqual,
                    Method::Ne => Punctuation::BangEqual,
                    Method::Lt => Punctuation::Less,
                    Method::Le => Punctuation::LessEqual,
                    Method::Gt => Punctuation::Greater,
                    Method::Ge => Punctuation::GreaterEqual,
                    Method::Min | Method::Max | Method::Clamp => {
                        return Err(
                            "owned comparison helper requires its checked provided original",
                        );
                    }
                }
            }
            _ => return Err("primitive descriptor differs from canonical method identity"),
        };
        let source = facts
            .associated
            .language_origin(owner)
            .ok_or("primitive method has no library origin")?;
        Ok((source, operator))
    }
    fn validate_receiver(
        self,
        facts: &Declarations,
        receiver: TypeId,
        result: TypeId,
        bound_regions: u32,
    ) -> Result<TypeId, &'static str> {
        let left = TypeId::primitive(self.left);
        let valid = match self.form {
            Form::Assignment => {
                matches!(self.operation,Operation::Arithmetic(hook) if hook.assignment())
                    && bound_regions == 1
                    && result == TypeId::UNIT
                    && matches!(facts.types.kind(receiver), Some(Kind::Reference { referent, mutable:true, region })
                    if referent == left && region == RegionId::bound(0,0)?)
            }
            Form::Comparison => {
                matches!(
                    self.operation,
                    Operation::Comparison(
                        crate::support::comparisons::Comparison::PartialEq
                            | crate::support::comparisons::Comparison::PartialOrd
                            | crate::support::comparisons::Comparison::Ord
                    )
                ) && self.rhs == Rhs::Owned
                    && bound_regions == 2
                    && matches!(facts.types.kind(receiver),Some(Kind::Reference{referent,mutable:false,region}) if referent==left && region==RegionId::bound(0,0)?)
            }
            Form::ValueBinary => {
                matches!(self.operation,Operation::Arithmetic(hook) if value_binary(hook))
                    && self.rhs == Rhs::Owned
                    && bound_regions == 0
                    && receiver == left
                    && result == left
            }
        };
        if !valid {
            return Err("primitive method receiver or result differs from descriptor");
        }
        Ok(left)
    }
    pub(in crate::sema) fn output(
        self,
        facts: &Declarations,
        item: crate::types::ItemId,
        family: crate::types::GenericArgs,
    ) -> Result<TypeId, &'static str> {
        let owner = facts
            .types
            .projection_schema(item)
            .ok_or("primitive result requires a declared projection")?
            .trait_item;
        if self.form != Form::ValueBinary
            || !matches!(self.operation,Operation::Arithmetic(hook) if value_binary(hook))
            || self.rhs != Rhs::Owned
            || family != crate::types::GenericArgs::EMPTY
            || facts.associated.member_owner(DefId(item.0)) != Some(DefId(owner.0))
            || !facts
                .associated
                .operation(DefId(owner.0))
                .is_some_and(|(hook, _)| self.operation == Operation::Arithmetic(hook))
            || !super::scalar::binary(self.operator()?, self.left, self.right)
        {
            return Err("primitive evidence applies only to its canonical Output member");
        }
        Ok(TypeId::primitive(self.left))
    }
    fn signature(
        self,
        facts: &mut Declarations,
        signature: TypeId,
    ) -> Result<TypeId, &'static str> {
        if self.form != Form::ValueBinary {
            return Ok(signature);
        }
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions,
        }) = facts.types.kind(signature)
        else {
            return Err("primitive template is not callable");
        };
        let Some(Kind::Projection(application)) = facts.types.kind(result) else {
            return Err("primitive template has no canonical Output projection");
        };
        let result = self.output(facts, application.item, application.arguments)?;
        facts.types.intern(Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions,
        })
    }
}
fn operator(hook: Hook) -> Result<Punctuation, &'static str> {
    Ok(match hook {
        Hook::Add | Hook::AddAssign => Punctuation::Plus,
        Hook::Sub | Hook::SubAssign => Punctuation::Minus,
        Hook::Mul | Hook::MulAssign => Punctuation::Star,
        Hook::Div | Hook::DivAssign => Punctuation::Slash,
        Hook::Rem | Hook::RemAssign => Punctuation::Percent,
        Hook::BitAnd | Hook::BitAndAssign => Punctuation::And,
        Hook::BitOr | Hook::BitOrAssign => Punctuation::Or,
        Hook::BitXor | Hook::BitXorAssign => Punctuation::Caret,
        Hook::Shl | Hook::ShlAssign => Punctuation::ShiftLeft,
        Hook::Shr | Hook::ShrAssign => Punctuation::ShiftRight,
        _ => return Err("primitive descriptor is not a supported operation"),
    })
}

fn value_binary(hook: Hook) -> bool {
    matches!(
        hook,
        Hook::Add
            | Hook::Sub
            | Hook::Mul
            | Hook::Div
            | Hook::Rem
            | Hook::BitAnd
            | Hook::BitOr
            | Hook::BitXor
            | Hook::Shl
            | Hook::Shr
    )
}
pub(in crate::sema) fn supported(hook: Hook) -> bool {
    hook.assignment() || value_binary(hook)
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
    if let Some(kind) = facts.associated.comparison(DefId(item.0)) {
        return comparison_candidate(facts, context, goal, kind, arguments);
    }
    let Some((hook, _)) = facts
        .associated
        .operation(DefId(item.0))
        .filter(|(kind, _)| supported(*kind))
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
    let form = if hook.assignment() {
        Form::Assignment
    } else {
        Form::ValueBinary
    };
    if form == Form::ValueBinary && rhs != Rhs::Owned {
        return Ok(Candidate::Unrelated);
    }
    let op = operator(hook).map_err(|_| infer::Error::InvalidIdentity)?;
    Ok(if super::scalar::binary(op, left, right) {
        Candidate::Present(Descriptor {
            operation: Operation::Arithmetic(hook),
            form,
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
    pub result: TypeId,
    pub operator: Punctuation,
    pub comparison: Option<crate::support::comparisons::Method>,
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
        let (source, operator) = self.descriptor.member(facts, definition)?;
        let Some(Kind::Function {
            parameters,
            result,
            unsafe_: false,
            convention: Convention::RUST,
            bound_regions,
        }) = facts.types.kind(signature)
        else {
            return Err("primitive method signature permissions differ from its contract");
        };
        let parameters = facts
            .types
            .list(parameters)
            .ok_or("missing primitive method parameters")?;
        let [receiver, physical_right] = parameters else {
            return Err("primitive method has incorrect arity");
        };
        let (receiver, physical_right) = (*receiver, *physical_right);
        let subject_type =
            self.descriptor
                .validate_receiver(facts, receiver, result, bound_regions)?;
        let arguments = facts
            .types
            .arguments(arguments)
            .ok_or("missing primitive method arguments")?
            .collect::<Vec<_>>();
        let (subject, right) = match arguments.as_slice() {
            [Argument::Type(subject), Argument::Type(right)] => (*subject, *right),
            [Argument::Type(subject)]
                if matches!(
                    self.descriptor.operation,
                    Operation::Comparison(crate::support::comparisons::Comparison::Ord)
                ) =>
            {
                (*subject, *subject)
            }
            _ => return Err("primitive method has incorrect generic arguments"),
        };
        let expected = TypeId::primitive(self.descriptor.right);
        if self.descriptor.form == Form::Comparison
            && !matches!(facts.types.kind(physical_right),Some(Kind::Reference{referent,mutable:false,region}) if referent==right && region==RegionId::bound(0,1)?)
        {
            return Err("comparison RHS differs from independent shared borrow contract");
        }
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
            || subject != subject_type
            || !super::scalar::binary(
                self.descriptor.operator()?,
                self.descriptor.left,
                self.descriptor.right,
            )
        {
            return Err("primitive method arguments differ from selected descriptor");
        }
        validate_substitution(facts, self.descriptor, item, signature, &arguments)?;
        let comparison = facts.associated.comparison_method(definition);
        if comparison.is_some_and(|method| {
            !matches!(
                method,
                crate::support::comparisons::Method::Cmp
                    | crate::support::comparisons::Method::PartialCmp
            )
        }) && result != TypeId::primitive(Primitive::Bool)
        {
            return Err("primitive relation result is not bool");
        }
        Ok(Validated {
            definition,
            source,
            signature,
            receiver,
            right: physical_right,
            result,
            operator,
            comparison,
        })
    }
}
fn validate_substitution(
    facts: &mut Declarations,
    descriptor: Descriptor,
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
    let substituted = substitution.apply(&mut facts.types, template)?;
    let substituted = descriptor.signature(facts, substituted)?;
    if substituted != signature {
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
