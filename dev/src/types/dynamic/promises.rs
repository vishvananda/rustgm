//! Object promises keep Self outside the quantified trait inputs.
use crate::types::{
    GenericArgs, Kind, Store, TypeId,
    arguments::Argument,
    parameters::{
        clauses::{Clause, Quantified},
        predicates::{Bound, Predicate},
    },
    traits::Application,
};

impl Store {
    /// Read the explicitly stored trait and equality promises with their complete
    /// binder. Semantic consumers supply inherited contracts and compatibility.
    /// # Errors
    /// Rejects malformed object inputs, unknown types and exhausted typed arenas.
    pub fn dynamic_clauses(&mut self, ty: TypeId) -> Result<(Vec<Clause>, usize), &'static str> {
        let Some(Kind::Dynamic {
            shape,
            arguments,
            bound_regions,
            ..
        }) = self.kind(ty)
        else {
            return Err("object promises require a dynamic type");
        };
        let value = self.dynamic_shape(shape).ok_or("invalid object shape")?;
        let bindings = self
            .dynamic_bindings(shape)
            .ok_or("invalid object binding shape")?
            .to_vec();
        let autos = self
            .dynamic_auto_traits(shape)
            .ok_or("invalid object auto traits")?
            .to_vec();
        let arguments = self
            .arguments(arguments)
            .ok_or("invalid object bound inputs")?
            .collect::<Vec<_>>();
        let (subject, work) = self.shift_type(ty, 1)?;
        let mut predicates = Vec::with_capacity(bindings.len() + autos.len() + 1);
        let mut start = value.principal_arguments as usize;
        if let Some(item) = value.principal {
            let arguments = self.intern_arguments(
                arguments
                    .get(..start)
                    .ok_or("invalid object principal inputs")?,
            )?;
            let application = self.intern_trait_application(Application { item, arguments })?;
            predicates.push(Predicate {
                ty: subject,
                bound: Bound::Trait(application),
            });
        }
        for binding in bindings {
            let end = start
                .checked_add(binding.arguments as usize)
                .ok_or("object binding input overflow")?;
            let parent = self.intern_arguments(
                arguments
                    .get(start..end)
                    .ok_or("invalid object binding inputs")?,
            )?;
            let Some(&Argument::Type(result)) = arguments.get(end) else {
                return Err("object equality requires a result type");
            };
            let trait_application = self.intern_trait_application(Application {
                item: binding.trait_item,
                arguments: parent,
            })?;
            let projected =
                self.intern(Kind::Projection(crate::types::projection::Application {
                    item: binding.item,
                    subject,
                    trait_application,
                    arguments: GenericArgs::EMPTY,
                }))?;
            predicates.push(Predicate {
                ty: projected,
                bound: Bound::ProjectionEqual(result),
            });
            start = end + 1;
        }
        if start != arguments.len() {
            return Err("invalid object promise input arity");
        }
        for marker in autos {
            predicates.push(Predicate {
                ty: subject,
                bound: Bound::Auto(marker),
            });
        }
        let work = work
            .saturating_add(arguments.len())
            .saturating_add(predicates.len());
        let clauses = predicates
            .into_iter()
            .map(|predicate| {
                self.intern_quantified(Quantified {
                    bound_regions,
                    predicate,
                })
                .map(Clause::ForAll)
            })
            .collect::<Result<_, _>>()?;
        Ok((clauses, work))
    }
}
