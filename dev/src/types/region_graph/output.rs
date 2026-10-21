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
    pub(crate) fn prove_output(mut self, interface: &[Region]) -> Result<Conditional, OutputError> {
        if self.members.is_some() {
            return Err(OutputError::Unsupported);
        }
        self.capacity(interface.len())?;
        let allowed: HashSet<_> = interface.iter().copied().collect();
        for &region in &allowed {
            if !matches!(
                self.kind(region)?,
                Kind::Static | Kind::Universal { universe: 0 }
            ) {
                return Err(Error::Identity.into());
            }
        }
        let mut conditions = HashMap::new();
        let mut solver = Solver::new(&self);
        solver.charge_many(interface.len())?;
        solver.propagate(|longer, shorter, source| {
            if !(allowed.contains(&longer) || longer == Region::STATIC)
                || !(allowed.contains(&shorter) || shorter == Region::STATIC)
            {
                return Err(Error::Unsatisfied {
                    longer,
                    shorter,
                    source,
                });
            }
            conditions
                .entry((longer, shorter))
                .and_modify(|previous: &mut u32| {
                    *previous = (*previous).min(source);
                })
                .or_insert(source);
            Ok(())
        })?;
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
        let conditional = graph.prove_output(&[input, output]);
        let Ok(conditional) = conditional else {
            return Err(Error::Identity);
        };
        assert_eq!(conditional.conditions, [(input, output, 1)]);
        assert!(conditional.solution.steps > 0);
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
                graph.prove_output(&[output]),
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
            let result = graph.prove_output(&[input, output]);
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
            let result = graph.prove_output(&[input, output]);
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
