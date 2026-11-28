//! A nested output is valid only under conditions discharged by its actual caller.
use super::{Error, Graph, HashMap, HashSet, Kind, Region, Solution, Solver};

pub(crate) enum OutputError {
    Unsupported,
    Constraint(Error),
}
impl From<Error> for OutputError {
    fn from(error: Error) -> Self {
        Self::Constraint(error)
    }
}
pub(crate) struct Conditional {
    pub solution: Solution,
    pub conditions: Vec<(Region, Region, u32)>,
}
impl Graph {
    /// Elimination supplies hypotheses, never proof of the original graph.
    /// Reprove all obligations under exactly the exported hypotheses before
    /// returning evidence. The caller must independently discharge every pair.
    /// A `longer_only` region (a rigid type parameter's common extent, which
    /// the caller names as that parameter) is exported only as the longer side.
    pub(crate) fn prove_output(
        mut self,
        interface: &[Region],
        longer_only: &[Region],
    ) -> Result<Conditional, OutputError> {
        if self.members.is_some() {
            return Err(OutputError::Unsupported);
        }
        self.capacity(interface.len().saturating_add(longer_only.len()))?;
        let allowed: HashSet<_> = interface.iter().copied().collect();
        let longer_allowed: HashSet<_> = longer_only.iter().copied().collect();
        for &region in allowed.iter().chain(&longer_allowed) {
            if !matches!(
                self.kind(region)?,
                Kind::Static | Kind::Universal { universe: 0 }
            ) {
                return Err(Error::Identity.into());
            }
        }
        if !allowed.is_disjoint(&longer_allowed) {
            return Err(Error::Identity.into());
        }
        let mut conditions = HashMap::default();
        let mut indirect = Vec::new();
        let mut solver = Solver::new(&self);
        solver.charge_many(interface.len().saturating_add(longer_only.len()))?;
        solver.propagate(|longer, shorter, source| {
            let longer_ok = allowed.contains(&longer)
                || longer_allowed.contains(&longer)
                || longer == Region::STATIC;
            if longer_ok && (allowed.contains(&shorter) || shorter == Region::STATIC) {
                conditions
                    .entry((longer, shorter))
                    .and_modify(|previous: &mut u32| {
                        *previous = (*previous).min(source);
                    })
                    .or_insert(source);
            } else if longer_ok {
                indirect.push((longer, shorter, source));
            }
            // The complete reproof below still requires private obligations.
            // Existing scalar debts can discharge them through declared bounds.
            Ok(())
        })?;
        if !indirect.is_empty() {
            let mut upper_bounds: HashMap<Region, Option<Region>> = HashMap::default();
            for (longer, shorter, source) in indirect {
                let upper = if let Some(&cached) = upper_bounds.get(&shorter) {
                    cached
                } else {
                    let found = assumed_upper_bound(&mut solver, interface, shorter)?;
                    upper_bounds.insert(shorter, found);
                    found
                };
                if let Some(upper) = upper {
                    conditions
                        .entry((longer, upper))
                        .and_modify(|previous: &mut u32| {
                            *previous = (*previous).min(source);
                        })
                        .or_insert(source);
                }
            }
        }
        let prior = solver.steps;
        drop(solver);
        let mut conditions: Vec<_> = conditions
            .into_iter()
            .map(|((longer, shorter), source)| (longer, shorter, source))
            .collect();
        conditions.sort_unstable_by_key(|&(longer, shorter, _)| (longer.0, shorter.0));
        for &(longer, shorter, _) in &conditions {
            self.assume(longer, shorter)?;
        }
        // Alternatives remain tests of the complete proof. Export only scalar
        // interface debts; do not assume a branch of a projected type bound.
        let solution = self.solve_charged(prior)?;
        Ok(Conditional {
            solution,
            conditions,
        })
    }
}

/// Unique minimal non-`'static` interface region assumed to outlive `shorter`.
/// Exporting `longer: upper` lets the caller discharge `longer: shorter` through
/// the body's own assumption `upper: shorter`, as in `Scope<'scope, 'env: 'scope>`.
fn assumed_upper_bound(
    solver: &mut Solver<'_>,
    interface: &[Region],
    shorter: Region,
) -> Result<Option<Region>, Error> {
    let mut candidates = Vec::new();
    for &region in interface {
        if region != Region::STATIC && solver.entails(region, shorter)? {
            candidates.push(region);
        }
    }
    if candidates.is_empty() {
        return Ok(None);
    }
    let mut minimal = Vec::new();
    for &candidate in &candidates {
        let mut is_minimal = true;
        for &other in &candidates {
            if candidate != other
                && solver.entails(candidate, other)?
                && !solver.entails(other, candidate)?
            {
                is_minimal = false;
                break;
            }
        }
        if is_minimal {
            minimal.push(candidate);
        }
    }
    let Some(&first) = minimal.first() else {
        return Ok(None);
    };
    for &other in &minimal[1..] {
        if !solver.entails(first, other)? || !solver.entails(other, first)? {
            return Ok(None);
        }
    }
    Ok(minimal.into_iter().min_by_key(|region| region.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditional_output_retains_its_input_requirement() -> Result<(), Error> {
        let mut graph = Graph::new(1024);
        let input = graph.region(Kind::Universal { universe: 0 })?;
        let output = graph.region(Kind::Universal { universe: 0 })?;
        let actual = graph.region(Kind::Variable { universe: 0 })?;
        graph.require(input, actual, 1)?;
        graph.require(actual, output, 2)?;
        let conditional = graph.prove_output(&[input, output], &[]);
        let Ok(conditional) = conditional else {
            return Err(Error::Identity);
        };
        assert_eq!(conditional.conditions, [(input, output, 1)]);
        assert!(conditional.solution.steps > 0);
        Ok(())
    }

    #[test]
    fn exported_scalar_debts_can_discharge_an_opaque_declared_bound() -> Result<(), Error> {
        for scalar_debt in [false, true] {
            let mut graph = Graph::new(4096);
            let input = graph.region(Kind::Universal { universe: 0 })?;
            let output = graph.region(Kind::Universal { universe: 0 })?;
            let parameter = graph.region(Kind::Universal { universe: 0 })?;
            graph.assume(parameter, input)?;
            graph.require(parameter, output, 1)?;
            if scalar_debt {
                graph.require(input, output, 2)?;
            }
            let result = graph.prove_output(&[input, output], &[]);
            if scalar_debt {
                let Ok(conditional) = result else {
                    return Err(Error::Identity);
                };
                assert_eq!(conditional.conditions, [(input, output, 2)]);
            } else {
                assert!(matches!(
                    result,
                    Err(OutputError::Constraint(Error::Unsatisfied {
                        source: 1,
                        ..
                    }))
                ));
            }
        }
        Ok(())
    }

    /// A parameter's extent named by the caller is exported only as the
    /// longer side; a pair it would have to be shorter in stays unproven.
    #[test]
    fn caller_named_parameters_export_only_as_the_longer_side() -> Result<(), Error> {
        for parameter_longer in [true, false] {
            let mut graph = Graph::new(4096);
            let output = graph.region(Kind::Universal { universe: 0 })?;
            let parameter = graph.region(Kind::Universal { universe: 0 })?;
            let actual = graph.region(Kind::Variable { universe: 0 })?;
            if parameter_longer {
                graph.require(parameter, actual, 1)?;
                graph.require(actual, output, 2)?;
            } else {
                graph.require(output, parameter, 3)?;
            }
            let result = graph.prove_output(&[output], &[parameter]);
            if parameter_longer {
                let Ok(conditional) = result else {
                    return Err(Error::Identity);
                };
                assert_eq!(conditional.conditions, [(parameter, output, 1)]);
            } else {
                assert!(matches!(
                    result,
                    Err(OutputError::Constraint(Error::Unsatisfied {
                        source: 3,
                        ..
                    }))
                ));
            }
        }
        let mut graph = Graph::new(1024);
        let shared = graph.region(Kind::Universal { universe: 0 })?;
        assert!(matches!(
            graph.prove_output(&[shared], &[shared]),
            Err(OutputError::Constraint(Error::Identity))
        ));
        Ok(())
    }

    #[test]
    fn caller_parameter_output_work_is_additive() -> Result<(), Error> {
        for count in [8, 32, 128] {
            let mut graph = Graph::new(64 * count + 128);
            let mut outputs = Vec::new();
            let mut parameters = Vec::new();
            for _ in 0..count {
                let output = graph.region(Kind::Universal { universe: 0 })?;
                let parameter = graph.region(Kind::Universal { universe: 0 })?;
                let actual = graph.region(Kind::Variable { universe: 0 })?;
                graph.require(parameter, actual, 1)?;
                graph.require(actual, output, 2)?;
                outputs.push(output);
                parameters.push(parameter);
            }
            let Ok(proof) = graph.prove_output(&outputs, &parameters) else {
                return Err(Error::Identity);
            };
            assert_eq!(proof.conditions.len(), count);
            assert!(proof.solution.steps <= 32 * count + 128);
        }
        Ok(())
    }

    #[test]
    fn private_storage_and_input_only_receivers_cannot_be_exported() -> Result<(), Error> {
        for local in [true, false] {
            let mut graph = Graph::new(1024);
            let output = graph.region(Kind::Universal { universe: 0 })?;
            let actual = graph.region(Kind::Variable { universe: 0 })?;
            let owner = if local {
                Region::LOCAL
            } else {
                graph.region(Kind::Universal { universe: 0 })?
            };
            graph.require(owner, actual, 1)?;
            graph.require(actual, output, 2)?;
            assert!(matches!(
                graph.prove_output(&[output], &[]),
                Err(OutputError::Constraint(Error::Unsatisfied { .. }))
            ));
        }
        Ok(())
    }

    #[test]
    fn conditional_scalar_debts_must_also_prove_every_verification() -> Result<(), Error> {
        for alternatives in [false, true] {
            let mut graph = Graph::new(4096);
            let input = graph.region(Kind::Universal { universe: 0 })?;
            let unrelated = graph.region(Kind::Universal { universe: 0 })?;
            let output = graph.region(Kind::Universal { universe: 0 })?;
            graph.require(input, output, 1)?;
            let required = graph.verify_outlives(input, output)?;
            let absent = graph.verify_outlives(unrelated, output)?;
            let test = if alternatives {
                graph.verify_any(&[required, absent])?
            } else {
                graph.verify_all(&[required, absent])?
            };
            graph.require_verification(test, 2)?;
            let result = graph.prove_output(&[input, output], &[]);
            if alternatives {
                let Ok(conditional) = result else {
                    return Err(Error::Identity);
                };
                assert_eq!(conditional.conditions, [(input, output, 1)]);
            } else {
                assert!(matches!(
                    result,
                    Err(OutputError::Constraint(Error::Verification { source: 2 }))
                ));
            }
        }
        Ok(())
    }

    #[test]
    fn output_verification_cannot_create_its_own_interface_assumption() -> Result<(), Error> {
        for assumed in [false, true] {
            let mut graph = Graph::new(4096);
            let input = graph.region(Kind::Universal { universe: 0 })?;
            let output = graph.region(Kind::Universal { universe: 0 })?;
            if assumed {
                graph.assume(input, output)?;
            }
            let test = graph.verify_outlives(input, output)?;
            graph.require_verification(test, 7)?;
            let result = graph.prove_output(&[input, output], &[]);
            if assumed {
                let Ok(conditional) = result else {
                    return Err(Error::Identity);
                };
                assert!(conditional.conditions.is_empty());
            } else {
                assert!(matches!(
                    result,
                    Err(OutputError::Constraint(Error::Verification { source: 7 }))
                ));
            }
        }
        Ok(())
    }
}
