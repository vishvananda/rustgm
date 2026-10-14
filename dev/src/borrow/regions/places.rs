//! Borrow limits follow typed places and share persistent reborrow ancestry.
use crate::{
    body::{
        ir::{Action, Body, NodeKey, Place, ProjectionKind, Rvalue, ValueId},
        regions::{Kind as Bound, Obligation},
    },
    types::{Kind, RegionId, Store},
};
const LIMIT: usize = 1_000_000;

#[derive(Clone, Copy)]
enum Origin {
    Local,
    Static,
    Temporary(ValueId),
    Reference(usize),
}
struct Link {
    region: RegionId,
    parent: Option<usize>,
}
struct Places<'a> {
    promotions: Option<&'a crate::body::promote::Facts>,
    extended: std::collections::HashSet<ValueId>,
    projections: Vec<Origin>,
    links: Vec<Link>,
    work: usize,
}
impl<'a> Places<'a> {
    fn new(body: &'a Body) -> Result<Self, &'static str> {
        let mut this = Self {
            promotions: body.promotions.as_ref(),
            extended: std::collections::HashSet::new(),
            projections: Vec::new(),
            links: Vec::new(),
            work: 0,
        };
        for temporary in &body.temporaries {
            this.step()?;
            if temporary.lifetime == crate::body::scopes::Lifetime::Static {
                this.extended.insert(temporary.value);
            }
        }
        for projection in &body.projections {
            this.step()?;
            let base = this.origin(projection.base)?;
            let origin = if let ProjectionKind::Dereference { region, mutable } = projection.kind {
                let parent = match base {
                    Origin::Reference(id) if mutable => Some(id),
                    _ => None,
                };
                let id = this.links.len();
                this.links.push(Link { region, parent });
                Origin::Reference(id)
            } else {
                base
            };
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
    fn step(&mut self) -> Result<(), &'static str> {
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
        source: NodeKey,
        output: &mut Vec<Obligation>,
    ) -> Result<(), &'static str> {
        self.step()?;
        let mut link = match self.origin(place)? {
            Origin::Reference(id) => Some(id),
            Origin::Static => None,
            Origin::Local => {
                output.push(Obligation {
                    source,
                    kind: Bound::LocalBorrow(region),
                });
                None
            }
            Origin::Temporary(id) => {
                let facts = self
                    .promotions
                    .ok_or("temporary borrow requires promotion facts")?;
                if !facts.contains(id) {
                    // This is only the body-extent upper bound. Precise storage
                    // liveness remains a separate mandatory execution proof.
                    output.push(Obligation {
                        source,
                        kind: Bound::LocalBorrow(region),
                    });
                }
                None
            }
        };
        while let Some(id) = link {
            self.step()?;
            let ancestor = &self.links[id];
            output.push(Obligation {
                source,
                kind: Bound::Outlives((ancestor.region, region)),
            });
            link = ancestor.parent;
        }
        Ok(())
    }
}
pub(super) fn bounds(types: &Store, body: &Body) -> Result<Vec<Obligation>, &'static str> {
    if !body
        .values
        .iter()
        .any(|v| matches!(v.kind, Rvalue::Borrow { .. }))
    {
        return Ok(Vec::new());
    }
    let mut places = Places::new(body)?;
    let mut output = Vec::new();
    for statement in &body.statements {
        places.step()?;
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
            NodeKey {
                node: statement.source,
                ..body.source
            },
            &mut output,
        )?;
    }
    Ok(output)
}
