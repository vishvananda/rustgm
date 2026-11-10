//! A proof-only alias check has no observer for isolated rigid self-pairs.
use super::{AliasCheck, Error, Graph, Region, Solver};
impl Graph {
    /// Keep every declared identity, assumption and required edge. Only plain
    /// proof consumption may omit seeds with no incoming requirement to visit.
    /// Rich graphs retain the full solver, including observed rigid self-pairs.
    pub(crate) fn check_aliases(self) -> Result<usize, Error> {
        let checked = self.check_aliases_counted();
        checked.result.map(|()| checked.steps)
    }
    pub(crate) fn check_aliases_counted(self) -> AliasCheck {
        let limit = self.limit;
        if self.members.is_some() || self.verifications.is_some() {
            return match self.solve() {
                Ok(solution) => AliasCheck {
                    result: Ok(()),
                    steps: solution.steps,
                    conservative: false,
                },
                Err(error) => AliasCheck {
                    result: Err(error),
                    steps: limit,
                    conservative: true,
                },
            };
        }
        let mut solver = Solver::new(&self);
        let result = solver.seed_alias_frontier().and_then(|()| {
            solver.propagate_work(
                |longer, shorter, source| {
                    Err(Error::Unsatisfied {
                        source,
                        longer,
                        shorter,
                    })
                },
                |_, _| {},
            )
        });
        let conservative = matches!(result, Err(Error::Budget));
        let steps = if conservative {
            solver.steps.max(limit)
        } else {
            solver.steps
        };
        AliasCheck {
            result,
            steps,
            conservative,
        }
    }
}
impl Solver<'_> {
    fn seed_alias_frontier(&mut self) -> Result<(), Error> {
        // The scan is real work even when almost all declared slots are unused.
        self.charge_many(self.graph.kinds.len())?;
        for (index, kind) in self.graph.kinds.iter().enumerate() {
            if kind.rigid() && self.graph.incoming[index].is_some() {
                let id = Region(u32::try_from(index).map_err(|_| Error::Budget)?);
                self.insert(id, id)?;
            }
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../../tests/invariants/alias_frontiers.rs"]
mod tests;
