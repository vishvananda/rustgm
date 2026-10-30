//! A supplied-body descriptor seals raw data, never a semantic proof domain.
use super::{HeaderIndex, SourceRows, functions};
use crate::support::{
    constness::Constness,
    crates::generics::{Convention, Work, source as s},
    edition::Edition,
};

pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    work: &mut Work,
) -> Result<usize, &'static str> {
    work.charge(1)?;
    if rows.policy.edition != Edition::Rust2024 || rows.policy.provider.0 != 0 {
        return Err("provided source data requires its own Rust2024 provider profile");
    }
    functions::bindings(rows, headers, work, functions::BindingProfile::Provided)?;
    for original in &rows.originals {
        for projection in &original.projections {
            work.charge(1)?;
            if matches!(projection.base, s::Place::Temporary(_)) {
                return Err(
                    "source SSA projection bases require their explicit v9 ordinary profile",
                );
            }
        }
        for value in &original.values {
            work.charge(1)?;
            if matches!(
                value.kind,
                s::ValueKind::Read(s::Place::Temporary(_))
                    | s::ValueKind::Borrow(s::Place::Temporary(_))
            ) {
                return Err("source SSA reads require their explicit v9 ordinary profile");
            }
            if matches!(value.kind, s::ValueKind::Scalar(_)) {
                return Err("source scalar operation requires its explicit v4 ordinary profile");
            }
            if matches!(value.kind, s::ValueKind::ScalarCast(_)) {
                return Err("source scalar cast requires its explicit v9 ordinary profile");
            }
        }
    }
    for header in &rows.traits {
        work.charge(1)?;
        if header.flags.bits() & s::TraitFlags::RESTRICTED_IMPL != 0 {
            return Err("restricted trait domains are unavailable in the Provided profile");
        }
    }
    for declaration in &rows.declarations {
        work.charge(1)?;
        if declaration.definition.kind == s::DefinitionKind::Module
            && declaration.definition.parent.is_some()
        {
            return Err("nested source modules require a wider source namespace profile");
        }
    }
    for implementation in &rows.implementations {
        work.charge(1)?;
        if implementation.negative {
            return Err("negative source implementation requires its separate semantic profile");
        }
    }
    for function in &rows.functions {
        work.charge(1)?;
        if function.constness == Constness::Conditional
            || function.original.unsafe_
            || function.callable.unsafe_
            || function.original.variadic
            || function.callable.variadic
            || function.original.convention != Convention::Rust
            || function.callable.convention != Convention::Rust
        {
            return Err("provided source callable requires its explicit safe Rust signature");
        }
        let s::FunctionBody::Original(body) = function.body else {
            continue;
        };
        let default = function
            .parent
            .map(|parent| headers.header(parent, work))
            .transpose()?
            .flatten()
            .is_some_and(|parent| {
                matches!(parent, s::HeaderRef::Trait(index)
                if rows.traits[index as usize].flags.bits() & s::TraitFlags::CONST != 0)
            });
        let expected = if default || function.constness == Constness::Const {
            s::CallContext::ConstDefinition
        } else {
            s::CallContext::Runtime
        };
        if rows.originals[body.0 as usize].context != expected {
            return Err("provided source original changed its actual defining call context");
        }
    }
    for input in &rows.body_inputs {
        for obligation in &input.obligations {
            work.charge(1)?;
            if matches!(
                obligation.kind,
                s::InputKind::Coercion {
                    reference: Some(s::ReferenceConversion {
                        two_phase: true,
                        ..
                    }),
                    ..
                }
            ) {
                return Err("two-phase source borrowing requires its complete operation profile");
            }
        }
    }
    Ok(headers.numbering(work)?.peak_bytes())
}
