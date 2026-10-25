//! Borrow limits follow typed places and share persistent reborrow ancestry.
use crate::{
    body::{
        ir::{Action, Body, Place, ProjectionKind, ProofSite, Rvalue, ValueId},
        regions::{Kind as Bound, Obligation},
    },
    types::{Kind, RegionId, Store},
};
const LIMIT: usize = 1_000_000;

/// Raw typed provenance requests, with allocation/work charged by their owner.
/// Visiting these requests grants no region solution or execution permission.
pub(crate) trait PlaceBoundsSink {
    fn charge(&mut self, count: usize) -> Result<(), &'static str>;
    fn scratch(&mut self, growth: usize) -> Result<(), &'static str>;
    fn obligation(&mut self, value: Obligation) -> Result<(), &'static str>;
}
fn capacity<T>(values: &Vec<T>, sink: &mut impl PlaceBoundsSink) -> Result<(), &'static str> {
    if values.len() == values.capacity() {
        sink.scratch(
            values
                .len()
                .saturating_add(1)
                .saturating_mul(4 * size_of::<T>()),
        )?;
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum Origin {
    Local,
    Static,
    Raw,
    Temporary(ValueId),
    Reference(usize),
}
struct Link {
    region: RegionId,
    parent: Option<usize>,
}
struct Places<'a> {
    promotions: Option<&'a crate::body::promote::Facts>,
    extended: crate::support::hash::HashSet<ValueId>,
    projections: Vec<Origin>,
    links: Vec<Link>,
    work: usize,
}
impl<'a> Places<'a> {
    fn new(body: &'a Body, sink: &mut impl PlaceBoundsSink) -> Result<Self, &'static str> {
        let mut this = Self {
            promotions: body.promotions.as_ref(),
            extended: crate::support::hash::HashSet::default(),
            projections: Vec::new(),
            links: Vec::new(),
            work: 0,
        };
        for temporary in &body.temporaries {
            this.step(sink)?;
            if temporary.lifetime == crate::body::scopes::Lifetime::Static {
                if this.extended.len() == this.extended.capacity()
                    && !this.extended.contains(&temporary.value)
                {
                    sink.scratch(crate::support::retained::new_map::<ValueId, ()>(
                        this.extended.len().saturating_add(1),
                    ))?;
                }
                this.extended.insert(temporary.value);
            }
        }
        for projection in &body.projections {
            this.step(sink)?;
            let base = this.origin(projection.base)?;
            let origin = if let ProjectionKind::Dereference { region, mutable } = projection.kind {
                let parent = match base {
                    Origin::Reference(id) if mutable => Some(id),
                    _ => None,
                };
                let id = this.links.len();
                capacity(&this.links, sink)?;
                this.links.push(Link { region, parent });
                Origin::Reference(id)
            } else if matches!(projection.kind, ProjectionKind::RawDereference { .. }) {
                Origin::Raw
            } else {
                base
            };
            capacity(&this.projections, sink)?;
            this.projections.push(origin);
        }
        Ok(this)
    }
    fn origin(&self, place: Place) -> Result<Origin, &'static str> {
        Ok(match place {
            Place::Local(_) => Origin::Local,
            Place::Static(_) => Origin::Static,
            Place::Temporary(id) if self.extended.contains(&id) => Origin::Static,
            Place::Temporary(id) => Origin::Temporary(id),
            Place::Projection(id) => *self
                .projections
                .get(id.0 as usize)
                .ok_or("borrow projection has no earlier base")?,
        })
    }
    fn step(&mut self, sink: &mut impl PlaceBoundsSink) -> Result<(), &'static str> {
        sink.charge(1)?;
        self.work += 1;
        if self.work > LIMIT {
            return Err("borrow provenance work budget exceeded");
        }
        Ok(())
    }
    fn borrow(
        &mut self,
        place: Place,
        region: RegionId,
        source: ProofSite,
        sink: &mut impl PlaceBoundsSink,
    ) -> Result<(), &'static str> {
        self.step(sink)?;
        let mut link = match self.origin(place)? {
            Origin::Reference(id) => Some(id),
            Origin::Static | Origin::Raw => None,
            Origin::Local => {
                sink.obligation(Obligation {
                    source,
                    kind: Bound::LocalBorrow(region),
                })?;
                None
            }
            Origin::Temporary(id) => {
                let facts = self
                    .promotions
                    .ok_or("temporary borrow requires promotion facts")?;
                if !facts.contains(id) {
                    // This is only the body-extent upper bound. Precise storage
                    // liveness remains a separate mandatory execution proof.
                    sink.obligation(Obligation {
                        source,
                        kind: Bound::LocalBorrow(region),
                    })?;
                }
                None
            }
        };
        while let Some(id) = link {
            self.step(sink)?;
            let ancestor = &self.links[id];
            sink.obligation(Obligation {
                source,
                kind: Bound::Outlives((ancestor.region, region)),
            })?;
            link = ancestor.parent;
        }
        Ok(())
    }
}
pub(super) fn bounds(types: &Store, body: &Body) -> Result<Vec<Obligation>, &'static str> {
    let mut output = Collected(Vec::new());
    visit_bounds(types, body, &mut output)?;
    Ok(output.0)
}
struct Collected(Vec<Obligation>);
impl PlaceBoundsSink for Collected {
    fn charge(&mut self, _: usize) -> Result<(), &'static str> {
        Ok(())
    }
    fn scratch(&mut self, _: usize) -> Result<(), &'static str> {
        Ok(())
    }
    fn obligation(&mut self, value: Obligation) -> Result<(), &'static str> {
        self.0.push(value);
        Ok(())
    }
}
pub(crate) fn visit_bounds(
    types: &Store,
    body: &Body,
    sink: &mut impl PlaceBoundsSink,
) -> Result<(), &'static str> {
    let mut borrowed = false;
    for value in &body.values {
        sink.charge(1)?;
        if matches!(value.kind, Rvalue::Borrow { .. }) {
            borrowed = true;
            break;
        }
    }
    if !borrowed {
        return Ok(());
    }
    let mut places = Places::new(body, sink)?;
    for statement in &body.statements {
        places.step(sink)?;
        let Action::Evaluate(id) = statement.action else {
            continue;
        };
        let value = body
            .values
            .get(id.0 as usize)
            .ok_or("missing evaluated borrow value")?;
        let Rvalue::Borrow { place, .. } = value.kind else {
            continue;
        };
        let Some(Kind::Reference { region, .. }) = types.kind(value.ty) else {
            return Err("borrow value has no reference type");
        };
        places.borrow(
            place,
            region,
            body.source.proof_site(statement.source)?,
            sink,
        )?;
    }
    Ok(())
}
