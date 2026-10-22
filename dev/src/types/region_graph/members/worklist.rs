//! One monotonic worklist grows hidden regions to minimal permitted captures.
use super::{Error, Graph, Members, Region, domains::Domains};
use crate::support::hash::{HashMap, HashSet};
use crate::types::region_graph::Solver;

struct Group {
    region: Region,
    domain: u32,
    processed: usize,
    selected: Option<Region>,
    source: u32,
}
struct Changes {
    watch: Vec<Option<usize>>,
    lower: Vec<Vec<Region>>,
    seen: HashSet<(usize, Region)>,
    queued: Vec<bool>,
    work: Vec<usize>,
}
impl Changes {
    fn observed(&mut self, region: Region, bound: Region) {
        if let Some(group) = self.watch[region.0 as usize]
            && self.seen.insert((group, bound))
        {
            self.lower[group].push(bound);
            if !self.queued[group] {
                self.queued[group] = true;
                self.work.push(group);
            }
        }
    }
}

fn unmet(longer: Region, shorter: Region, source: u32) -> Result<(), Error> {
    Err(Error::Unsatisfied {
        source,
        longer,
        shorter,
    })
}

pub(super) fn select(graph: &Graph, members: &Members) -> Result<(Vec<Region>, usize), Error> {
    let mut solver = Solver::new(graph);
    let (classes, required_order, steps) = graph.required_classes()?;
    solver.charge_many(steps)?;
    let (universals, order, steps) =
        graph.classes(&graph.assumed_heads, &graph.assumptions, true, true)?;
    solver.charge_many(steps)?;
    let mut domains = Domains::new(&universals, order, &mut solver)?;
    let (mut groups, indexes) = groups(members, &classes, &universals, &mut domains, &mut solver)?;
    let watch: Vec<_> = classes
        .iter()
        .map(|class| indexes.get(class).copied())
        .collect();
    upper_bounds(&watch, &universals, &mut groups, &mut domains, &mut solver)?;
    let mut work: Vec<_> = (0..groups.len()).collect();
    // Required SCCs were traversed in reverse. Pop smaller dependencies first,
    // before selecting a potentially conservative choice for their predecessors.
    work.sort_unstable_by_key(|&index| required_order[groups[index].region.0 as usize]);
    let mut changes = Changes {
        watch,
        lower: vec![Vec::new(); groups.len()],
        seen: HashSet::default(),
        queued: vec![true; groups.len()],
        work,
    };
    solver.seed_rigids()?;
    solver.propagate_work(unmet, |region, bound| changes.observed(region, bound))?;
    while let Some(index) = changes.work.pop() {
        solver.charge()?;
        changes.queued[index] = false;
        let group = &mut groups[index];
        for &bound in &changes.lower[index][group.processed..] {
            group.domain =
                domains.lower(group.domain, universals[bound.0 as usize], &mut solver)?;
        }
        group.processed = changes.lower[index].len();
        let selected = domains.minimum(group.domain, &mut solver)?;
        if let Some(choice) = selected
            && selected != group.selected
        {
            // The pinned opaque inference rule selects the least candidate
            // comparable with all alternatives. Keep that selected lower bound
            // for predecessor members; the final equality still needs proof.
            solver.insert(group.region, choice)?;
            solver.propagate_work(unmet, |region, bound| changes.observed(region, bound))?;
        }
        group.selected = selected;
    }
    let mut selected = Vec::with_capacity(members.requests.len());
    for request in &members.requests {
        solver.charge()?;
        let group = &groups[indexes[&classes[request.region.0 as usize]]];
        let choice = group.selected.ok_or(Error::Membership {
            source: group.source,
        })?;
        selected.push(domains.name(request.choices, choice)?);
    }
    Ok((selected, solver.steps))
}

fn groups(
    members: &Members,
    classes: &[Region],
    universals: &[Region],
    domains: &mut Domains,
    solver: &mut Solver<'_>,
) -> Result<(Vec<Group>, HashMap<Region, usize>), Error> {
    let mut groups: Vec<Group> = Vec::new();
    let mut indexes: HashMap<Region, usize> = HashMap::default();
    for request in &members.requests {
        solver.charge()?;
        let region = classes[request.region.0 as usize];
        let domain = domains.choices(request.choices, members, universals, solver)?;
        if let Some(&index) = indexes.get(&region) {
            let group = &mut groups[index];
            group.domain = domains.intersect(group.domain, domain, solver)?;
        } else {
            indexes.insert(region, groups.len());
            groups.push(Group {
                region,
                domain,
                processed: 0,
                selected: None,
                source: request.source,
            });
        }
    }
    Ok((groups, indexes))
}

fn upper_bounds(
    watch: &[Option<usize>],
    classes: &[Region],
    groups: &mut [Group],
    domains: &mut Domains,
    solver: &mut Solver<'_>,
) -> Result<(), Error> {
    let graph = solver.graph;
    // Transpose once, then share rigid-frontier traversal across all members.
    let mut outgoing = vec![Vec::new(); graph.kinds.len()];
    for (shorter, &head) in graph.incoming.iter().enumerate() {
        let shorter = Region(u32::try_from(shorter).map_err(|_| Error::Budget)?);
        let mut edge = head;
        while let Some(index) = edge {
            solver.charge()?;
            let relation = graph.obligations[index];
            edge = relation.next;
            outgoing[relation.target.0 as usize].push(shorter);
        }
    }
    let mut work = Vec::new();
    let mut seen = HashSet::default();
    let mut filtered = HashSet::default();
    for (index, kind) in graph.kinds.iter().enumerate() {
        if kind.rigid() {
            let region = Region(u32::try_from(index).map_err(|_| Error::Budget)?);
            work.push((region, region));
            seen.insert((region, region));
        }
    }
    while let Some((region, bound)) = work.pop() {
        solver.charge()?;
        if let Some(group) = watch[region.0 as usize]
            && filtered.insert((group, bound))
            && let Some(upper) = domains.upper(bound, classes, solver)?
        {
            let group = &mut groups[group];
            group.domain = domains.intersect(group.domain, upper, solver)?;
        }
        for &shorter in &outgoing[region.0 as usize] {
            solver.charge()?;
            if !graph.kind(shorter)?.rigid() && seen.insert((shorter, bound)) {
                work.push((shorter, bound));
            }
        }
    }
    Ok(())
}
