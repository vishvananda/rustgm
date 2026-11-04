//! Structural matching visits only the finite immutable recipe, never an arbitrary type graph.
use crate::{
    sema::model::Declarations,
    support::bindings::prototype_specs::{Region, Term},
    types::{Kind, RegionId, TypeId, arguments::Argument, callable::Convention, length::Length},
};

pub(super) struct Matcher<'a> {
    facts: &'a Declarations,
    formals: [TypeId; 4],
    regions: [Option<RegionId>; 3],
    bound_regions: u32,
    remaining: u8,
}
impl<'a> Matcher<'a> {
    pub(super) const fn new(
        facts: &'a Declarations,
        formals: [TypeId; 4],
        bound_regions: u32,
        work: u8,
    ) -> Self {
        Self {
            facts,
            formals,
            regions: [None; 3],
            bound_regions,
            remaining: work,
        }
    }
    pub(super) fn list(
        &mut self,
        expected: &[Term],
        actual: &[TypeId],
    ) -> Result<(), &'static str> {
        if expected.len() != actual.len() {
            return Err("source intrinsic signature arity differs from its pinned declaration");
        }
        for (&term, &ty) in expected.iter().zip(actual) {
            self.ty(term, ty)?;
        }
        Ok(())
    }
    pub(super) fn ty(&mut self, expected: Term, actual: TypeId) -> Result<(), &'static str> {
        self.remaining = self
            .remaining
            .checked_sub(1)
            .ok_or("intrinsic signature recipe exhausted")?;
        let kind = self
            .facts
            .types
            .kind(actual)
            .ok_or("missing intrinsic signature type")?;
        match (expected, kind) {
            (Term::Unit, _) if actual == TypeId::UNIT => Ok(()),
            (Term::Never, Kind::Never) => Ok(()),
            (Term::Scalar(scalar), Kind::Primitive(p)) if scalar.name() == p.name() => Ok(()),
            (Term::Parameter(index), _)
                if self.formals.get(usize::from(index)) == Some(&actual) =>
            {
                Ok(())
            }
            (
                Term::Pointer(mutable, element),
                Kind::Pointer {
                    mutable: actual_mutable,
                    pointee,
                },
            ) if mutable == actual_mutable => self.ty(*element, pointee),
            (
                Term::Reference(mutable, expected_region, element),
                Kind::Reference {
                    mutable: actual_mutable,
                    region,
                    referent,
                },
            ) if mutable == actual_mutable => {
                self.region(expected_region, region)?;
                self.ty(*element, referent)
            }
            (Term::Tuple(fields), Kind::Tuple(list)) => self.list(
                fields,
                self.facts
                    .types
                    .list(list)
                    .ok_or("missing intrinsic tuple fields")?,
            ),
            (
                Term::Array(element, length),
                Kind::Array {
                    element: actual_element,
                    ..
                }
                | Kind::ArrayTerm {
                    element: actual_element,
                    ..
                },
            ) if Length::of(&self.facts.types, actual)?.known() == Some(length) => {
                self.ty(*element, actual_element)
            }
            (Term::Language(role, arguments), Kind::Adt(owner, actual_arguments)) => {
                self.nominal(role, arguments, owner, actual_arguments)
            }
            (
                Term::Function(safety, inputs, output),
                Kind::Function {
                    parameters,
                    result,
                    unsafe_,
                    convention: Convention::RUST,
                    ..
                },
            ) if safety == unsafe_ => {
                self.list(
                    inputs,
                    self.facts
                        .types
                        .list(parameters)
                        .ok_or("missing intrinsic callback inputs")?,
                )?;
                self.ty(*output, result)
            }
            (Term::OwnedBox, _) => {
                Err("source intrinsic Box signature requires its real owned-box producer")
            }
            _ => Err("source intrinsic signature differs from its pinned declaration"),
        }
    }
    fn nominal(
        &mut self,
        role: crate::support::bindings::LangItem,
        expected: &[Term],
        owner: crate::types::ItemId,
        arguments: crate::types::GenericArgs,
    ) -> Result<(), &'static str> {
        let declaration = self
            .facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(role))
            .ok_or("source intrinsic signature requires its actual language declaration")?;
        if owner.0 != declaration.definition.0 {
            return Err("source intrinsic signature names another nominal declaration");
        }
        let actual = self
            .facts
            .types
            .arguments(arguments)
            .ok_or("missing intrinsic nominal arguments")?;
        if expected.len() != actual.len() {
            return Err("source intrinsic signature changed its nominal argument count");
        }
        for (&term, argument) in expected.iter().zip(actual) {
            match (term, argument) {
                (Term::Region(index), Argument::Lifetime(region)) => {
                    self.region(Region::Bound(index), region)?;
                }
                (_, Argument::Type(ty)) => self.ty(term, ty)?,
                _ => return Err("source intrinsic signature changed its nominal argument kinds"),
            }
        }
        Ok(())
    }
    fn region(&mut self, expected: Region, actual: RegionId) -> Result<(), &'static str> {
        let Region::Bound(index) = expected else {
            return if actual == RegionId::STATIC {
                Ok(())
            } else {
                Err("source intrinsic requires a static region")
            };
        };
        if !matches!(actual.bound_parts(), Some((0, slot)) if u32::from(slot) < self.bound_regions)
        {
            return Err("source intrinsic requires its actual late-bound region");
        }
        // Equivalent signatures may number used bound regions differently. Preserve
        // equality and distinction, rather than assigning a static or early region.
        if self
            .regions
            .iter()
            .enumerate()
            .any(|(i, &r)| i != usize::from(index) && r == Some(actual))
        {
            return Err("source intrinsic collapsed distinct bound regions");
        }
        let slot = self
            .regions
            .get_mut(usize::from(index))
            .ok_or("intrinsic region recipe is invalid")?;
        if slot.is_some_and(|r| r != actual) {
            return Err("source intrinsic changed a shared bound region");
        }
        *slot = Some(actual);
        Ok(())
    }
}
