//! Record distinct live requirements by updating multiplicities at entity transitions.
use super::{
    Analysis, Live,
    components::{Component, Components},
};
use crate::borrow::regions::points::Point;

/// Fixed-point transfer keeps plain entity bits; recording adds component counts.
pub(super) trait State {
    fn mark(&mut self, entity: usize, used: bool) -> Result<(), &'static str>;
}
impl State for Live {
    fn mark(&mut self, entity: usize, used: bool) -> Result<(), &'static str> {
        self.mark(entity, used);
        Ok(())
    }
}

pub(super) struct Population<'a> {
    components: &'a Components,
    live: Live,
    active: Live,
    counts: Vec<u32>,
    steps: usize,
}
impl<'a> Population<'a> {
    pub fn new(components: &'a Components) -> Self {
        Self {
            components,
            live: Live::new(components.entities.len().div_ceil(64)),
            active: Live::new(components.values.len().div_ceil(64)),
            counts: vec![0; components.values.len()],
            steps: 0,
        }
    }
    fn charge(&mut self, cost: usize) -> Result<(), &'static str> {
        self.steps = self
            .steps
            .checked_add(cost)
            .filter(|n| *n <= super::LIMIT)
            .ok_or("liveness population work budget exceeded")?;
        Ok(())
    }
    pub fn take_steps(&mut self) -> usize {
        std::mem::take(&mut self.steps)
    }
    /// Each block/edge starts from the same solved entity state as before.
    pub fn reset(&mut self, source: &Live) -> Result<(), &'static str> {
        self.charge(
            self.counts
                .len()
                .saturating_add(self.live.len().saturating_mul(3)),
        )?;
        self.counts.fill(0);
        self.active.clear();
        self.live.copy_from(source.words());
        for (word, mut bits) in source.active() {
            self.charge(source.depth())?;
            while bits != 0 {
                let entity = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                self.change(entity, true)?;
            }
        }
        Ok(())
    }
    fn contains(&self, entity: usize) -> Result<bool, &'static str> {
        let bits = self
            .live
            .words()
            .get(entity / 64)
            .ok_or("invalid liveness population entity")?;
        Ok(bits & (1 << (entity % 64)) != 0)
    }
    fn change(&mut self, entity: usize, used: bool) -> Result<(), &'static str> {
        let set = *self
            .components
            .entities
            .get(entity)
            .ok_or("invalid liveness component entity")?;
        let ids = self
            .components
            .sets
            .get(set)
            .ok_or("invalid liveness component set")?;
        self.charge(
            ids.len()
                .saturating_mul(self.active.depth() + 1)
                .saturating_add(1),
        )?;
        for &id in ids {
            let count = &mut self.counts[id as usize];
            *count = if used {
                count.checked_add(1)
            } else {
                count.checked_sub(1)
            }
            .ok_or("invalid liveness component multiplicity")?;
            if *count == u32::from(used) {
                self.active.mark(id as usize, used);
            }
        }
        Ok(())
    }
    pub fn seed(
        &mut self,
        result: &mut Analysis,
        point: Point,
        borrow: Option<usize>,
    ) -> Result<(), &'static str> {
        // A newly created borrow needs its result at this point even if unused.
        // This temporary union must not keep it alive across earlier statements.
        let extra = borrow
            .map(|entity| self.contains(entity).map(|live| (entity, live)))
            .transpose()?
            .filter(|&(_, live)| !live)
            .map(|(entity, _)| entity);
        if let Some(entity) = extra {
            self.mark(entity, true)?;
        }
        self.emit(result, point)?;
        if let Some(entity) = extra {
            self.mark(entity, false)?;
        }
        Ok(())
    }
    fn emit(&mut self, result: &mut Analysis, point: Point) -> Result<(), &'static str> {
        let mut cost = 1_usize;
        for (word, mut bits) in self.active.active() {
            cost = cost.saturating_add(self.active.depth());
            while bits != 0 {
                let id = word * 64 + bits.trailing_zeros() as usize;
                bits &= bits - 1;
                cost = cost.saturating_add(1);
                if result
                    .seeds
                    .len()
                    .saturating_add(result.parameter_seeds.len())
                    >= 2_000_000
                {
                    return Err("liveness seed budget exceeded");
                }
                match self.components.values[id] {
                    Component::Region(region) => result.seeds.push((region, point)),
                    Component::Parameter(parameter) => {
                        result.parameter_seeds.push((parameter, point));
                    }
                }
            }
        }
        self.charge(cost)
    }
}
impl State for Population<'_> {
    fn mark(&mut self, entity: usize, used: bool) -> Result<(), &'static str> {
        self.charge(self.live.depth())?;
        if self.contains(entity)? != used {
            self.change(entity, used)?;
            self.live.mark(entity, used);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::support::hash::HashSet;
    use crate::{
        borrow::liveness::components::Builder,
        types::{RegionId, outlives::Parts, parameters::ParameterId},
    };

    fn analysis() -> Analysis {
        Analysis {
            seeds: Vec::new(),
            parameter_seeds: Vec::new(),
            points: 0,
            reachable: Vec::new(),
            steps: 0,
            locations: Vec::new(),
            ordered: None,
        }
    }

    #[test]
    fn transitions_and_borrow_births_match_independent_entity_expansion() -> Result<(), &'static str>
    {
        let sets = [
            (vec![RegionId(1)], vec![ParameterId(1)]),
            (vec![RegionId(1)], vec![]),
            (vec![], vec![ParameterId(1)]),
            (vec![RegionId(2)], vec![ParameterId(2)]),
            (vec![], vec![]),
        ];
        let mut builder = Builder::default();
        for (regions, parameters) in &sets {
            let set = builder.intern(Parts {
                regions,
                parameters,
            })?;
            builder.entity(set)?;
        }
        let components = builder.finish();
        let mut population = Population::new(&components);
        let mut expected = vec![false; sets.len()];
        let mut output = analysis();
        for step in 0..500_usize {
            if step % 17 == 0 {
                // Simulate an independent solved successor/edge state. Requirements
                // from the preceding branch must not survive this reset.
                let mut source = Live::new(1);
                for (entity, used) in expected.iter_mut().enumerate() {
                    *used = (step + entity) % 3 == 0;
                    source.mark(entity, *used);
                }
                population.reset(&source)?;
            }
            let entity = (step * 7) % sets.len();
            let used = step % 3 != 0;
            expected[entity] = used;
            population.mark(entity, used)?;
            population.mark(entity, used)?; // Repeated uses must not add owners.
            let borrow = (step % 4 == 0).then_some((entity + 1) % sets.len());
            output.seeds.clear();
            output.parameter_seeds.clear();
            population.seed(&mut output, Point(0), borrow)?;
            let mut regions = HashSet::default();
            let mut parameters = HashSet::default();
            for (index, (rs, ps)) in sets.iter().enumerate() {
                if expected[index] || borrow == Some(index) {
                    regions.extend(rs.iter().copied());
                    parameters.extend(ps.iter().copied());
                }
            }
            assert_eq!(output.seeds.len(), regions.len());
            assert_eq!(output.parameter_seeds.len(), parameters.len());
            assert!(output.seeds.iter().all(|(r, _)| regions.contains(r)));
            assert!(
                output
                    .parameter_seeds
                    .iter()
                    .all(|(p, _)| parameters.contains(p))
            );
            for (entity, &used) in expected.iter().enumerate() {
                assert_eq!(population.contains(entity)?, used);
            }
        }
        Ok(())
    }

    #[test]
    fn shared_live_requirements_scale_with_transfers_and_distinct_output()
    -> Result<(), &'static str> {
        for width in [8, 32, 128, 512, 4096] {
            let mut builder = Builder::default();
            let set = builder.intern(Parts {
                regions: &[RegionId(1)],
                parameters: &[ParameterId(1)],
            })?;
            for _ in 0..width {
                builder.entity(set)?;
            }
            let components = builder.finish();
            assert_eq!(components.sets.edge_count(), 2);
            let mut population = Population::new(&components);
            let mut output = analysis();
            for entity in 0..width {
                population.mark(entity, true)?;
                population.seed(
                    &mut output,
                    Point(u32::try_from(entity).map_err(|_| "point")?),
                    None,
                )?;
            }
            for entity in (0..width).rev() {
                population.mark(entity, false)?;
                population.seed(
                    &mut output,
                    Point(u32::try_from(width + entity).map_err(|_| "point")?),
                    None,
                )?;
            }
            assert_eq!(output.seeds.len(), 2 * width - 1);
            assert_eq!(output.parameter_seeds.len(), 2 * width - 1);
            assert!(population.take_steps() <= 30 * width);
            assert!(population.active.active().next().is_none());
        }
        Ok(())
    }

    #[test]
    fn distinct_parameters_remain_distinct_when_output_is_quadratic() -> Result<(), &'static str> {
        let width = 128_u32;
        let mut builder = Builder::default();
        for id in 0..width {
            let set = builder.intern(Parts {
                regions: &[],
                parameters: &[ParameterId(id)],
            })?;
            builder.entity(set)?;
        }
        let components = builder.finish();
        let mut population = Population::new(&components);
        let mut output = analysis();
        for id in 0..width {
            population.mark(id as usize, true)?;
            population.seed(&mut output, Point(id), None)?;
        }
        assert!(output.seeds.is_empty());
        assert_eq!(
            output.parameter_seeds.len(),
            (width * (width + 1) / 2) as usize
        );
        for (parameter, point) in output.parameter_seeds {
            assert!(parameter.0 <= point.0);
        }
        Ok(())
    }
}
