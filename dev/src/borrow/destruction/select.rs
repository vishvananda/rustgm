//! Select sparse conditional cleanup without reconstructing source or expanding arrays.
use super::{
    Cleanup, Condition, Field, Flag, Interval, Node, NodeId, Operation, Plan, Variant, glue,
};
use crate::{
    body::ir::{Body, Place, Selection},
    borrow::ownership::{
        Component, Error, PathId, Paths,
        initialization::{self, Candidate, Event, Point, Status},
    },
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
enum Tracking {
    Path(PathId),
    Bit(u32),
    External,
}
enum Work {
    Enter {
        node: NodeId,
        recipe: glue::Id,
        tracked: Tracking,
    },
    Fields {
        node: NodeId,
        fields: glue::Range,
        tracked: Tracking,
    },
    Finish(NodeId),
}
struct Builder<'a, 'b> {
    catalog: &'a mut glue::Catalog<'b>,
    paths: &'a Paths,
    fields: HashMap<(PathId, Selection), PathId>,
    plan: Plan,
    work: Vec<Work>,
    retained: usize,
}
/// Select destruction operations from proved initialization and selected methods.
/// This requires the custom-destructor field-move rule even though executable
/// permission still depends on flag updates and destructor-aware loan checking.
/// # Errors
/// Rejects invalid destructor moves, typed facts, or bounded planning resources.
pub fn select(
    catalog: &mut glue::Catalog<'_>,
    facts: &initialization::Facts<'_>,
) -> Result<Plan, Error> {
    facts.destructor_moves(catalog.types(), |item| catalog.custom(item))?;
    let body = facts.body();
    let error = |message| Error {
        node: body.source,
        message,
    };
    let mut builder = Builder {
        catalog,
        paths: &facts.ownership.paths,
        fields: HashMap::new(),
        plan: Plan::default(),
        work: Vec::new(),
        retained: 0,
    };
    builder.charge(builder.paths.entries.len()).map_err(error)?;
    for (index, path) in builder.paths.entries.iter().enumerate() {
        if let (Some(parent), Some(Component::Field(field))) = (path.parent, path.component) {
            builder
                .retain::<((PathId, Selection), PathId)>(2)
                .map_err(error)?;
            builder
                .fields
                .insert((parent, field), PathId(compact(index).map_err(error)?));
        }
    }
    facts
        .visit_events(|event, point| {
            builder.charge(1)?;
            if let Event::Cleanup(candidate) = event {
                builder.cleanup(body, candidate, point)?;
            }
            Ok(())
        })
        .map_err(error)?;
    builder.flags().map_err(error)?;
    super::flags::elaborate(&mut builder.plan, facts, builder.retained).map_err(error)?;
    Ok(builder.plan)
}
impl Builder<'_, '_> {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.plan.steps = self
            .plan
            .steps
            .checked_add(count)
            .filter(|n| *n <= 32_000_000)
            .ok_or("destruction planning work budget exceeded")?;
        Ok(())
    }
    fn retain<T>(&mut self, count: usize) -> Result<(), &'static str> {
        self.retained = count
            .checked_mul(size_of::<T>())
            .and_then(|n| self.retained.checked_add(n))
            .filter(|n| *n <= 64_000_000)
            .ok_or("destruction plan exceeds 64 MB budget")?;
        Ok(())
    }
    fn node(&mut self) -> Result<NodeId, &'static str> {
        self.retain::<Node>(1)?;
        let id = NodeId(compact(self.plan.nodes.len())?);
        self.plan.nodes.push(Node {
            condition: Condition::Always,
            operation: Operation::Empty,
        });
        Ok(id)
    }
    fn cleanup(
        &mut self,
        body: &Body,
        candidate: Candidate,
        point: &mut Point<'_>,
    ) -> Result<(), &'static str> {
        let ty = match candidate.place {
            Place::Local(id) => body.locals.get(id.0 as usize).map(|l| l.ty),
            Place::Temporary(id) => body.values.get(id.0 as usize).map(|v| v.ty),
            Place::Projection(id) => body.projections.get(id.0 as usize).map(|p| p.ty),
            Place::Static(_) => None,
        }
        .ok_or("invalid or unsupported destruction place")?;
        let tracked = candidate.path.map_or(Tracking::External, Tracking::Path);
        let mut cleanup = Cleanup {
            candidate,
            node: None,
            live_type: None,
        };
        if let Some(recipe) = self.catalog.demand(ty)?
            && self.status(tracked, point)? != Status::Never
        {
            cleanup.live_type = Some(ty);
            let node = self.node()?;
            self.work.push(Work::Enter {
                node,
                recipe,
                tracked,
            });
            self.run(point)?;
            if !self.empty(node) {
                cleanup.node = Some(node);
            }
        }
        self.retain::<Cleanup>(1)?;
        self.plan.cleanups.push(cleanup);
        Ok(())
    }
    fn status(&self, tracked: Tracking, point: &mut Point<'_>) -> Result<Status, &'static str> {
        match tracked {
            Tracking::Path(id) => point.status(
                self.paths
                    .entries
                    .get(id.0 as usize)
                    .ok_or("invalid cleanup move path")?,
            ),
            Tracking::Bit(bit) => point.bit(bit),
            Tracking::External => Ok(Status::Always),
        }
    }
    fn shallow(&self, tracked: Tracking) -> Result<u32, &'static str> {
        match tracked {
            Tracking::Path(id) => self
                .paths
                .entries
                .get(id.0 as usize)
                .map(|p| p.start)
                .ok_or("invalid cleanup move path"),
            Tracking::Bit(bit) => Ok(bit),
            Tracking::External => Err("external cleanup has no initialization flag"),
        }
    }
    fn condition(
        &self,
        tracked: Tracking,
        point: &mut Point<'_>,
    ) -> Result<Condition, &'static str> {
        let bit = self.shallow(tracked)?;
        match point.bit(bit)? {
            Status::Always => Ok(Condition::Always),
            Status::Conditional => Ok(Condition::Initialized(Flag(bit))),
            Status::Never => Err("partly initialized object has no initialized structure"),
        }
    }
    fn child(&self, parent: Tracking, field: Selection) -> Result<Tracking, &'static str> {
        if let Tracking::Path(parent) = parent
            && let Some(&child) = self.fields.get(&(parent, field))
        {
            return Ok(Tracking::Path(child));
        }
        Ok(Tracking::Bit(self.shallow(parent)?))
    }
    fn run(&mut self, point: &mut Point<'_>) -> Result<(), &'static str> {
        while let Some(work) = self.work.pop() {
            self.charge(1)?;
            match work {
                Work::Enter {
                    node,
                    recipe,
                    tracked,
                } => self.enter(node, recipe, tracked, point)?,
                Work::Fields {
                    node,
                    fields,
                    tracked,
                } => self.fields(node, fields, tracked)?,
                Work::Finish(node) => self.finish(node)?,
            }
        }
        Ok(())
    }
    fn enter(
        &mut self,
        node: NodeId,
        recipe: glue::Id,
        tracked: Tracking,
        point: &mut Point<'_>,
    ) -> Result<(), &'static str> {
        let status = self.status(tracked, point)?;
        if status == Status::Never {
            return Ok(());
        }
        let data = self.catalog.get(recipe).ok_or("missing cleanup recipe")?;
        if status == Status::Always
            || data.method.is_some()
            || !matches!(tracked, Tracking::Path(_))
        {
            let condition = if status == Status::Always {
                Condition::Always
            } else {
                self.condition(tracked, point)?
            };
            self.plan.nodes[node.0 as usize] = Node {
                condition,
                operation: Operation::Glue(recipe),
            };
            return Ok(());
        }
        match data.kind {
            glue::Shape::Fields(fields) => self.fields(node, fields, tracked)?,
            glue::Shape::Variants(variants) => {
                self.plan.nodes[node.0 as usize].condition = self.condition(tracked, point)?;
                let start = compact(self.plan.variants.len())?;
                self.work.push(Work::Finish(node));
                for index in variants.start..variants.end {
                    self.charge(1)?;
                    let variant = *self
                        .catalog
                        .variants(variants)
                        .and_then(|v| v.get((index - variants.start) as usize))
                        .ok_or("invalid cleanup variant")?;
                    let child = self.node()?;
                    self.retain::<Variant>(1)?;
                    self.plan.variants.push(Variant {
                        selection: variant.selection,
                        node: child,
                    });
                    self.work.push(Work::Fields {
                        node: child,
                        fields: variant.fields,
                        tracked,
                    });
                }
                self.plan.nodes[node.0 as usize].operation = Operation::Variants(glue::Range {
                    start,
                    end: compact(self.plan.variants.len())?,
                });
            }
            glue::Shape::Array { element, .. } => {
                let Tracking::Path(path) = tracked else {
                    return Err("missing array cleanup path");
                };
                if self.paths.array_segments(path).is_some() {
                    self.array(node, path, element)?;
                } else {
                    self.plan.nodes[node.0 as usize] = Node {
                        condition: self.condition(tracked, point)?,
                        operation: Operation::Glue(recipe),
                    };
                }
            }
        }
        Ok(())
    }
    fn fields(
        &mut self,
        node: NodeId,
        fields: glue::Range,
        tracked: Tracking,
    ) -> Result<(), &'static str> {
        self.work.push(Work::Finish(node));
        let start = compact(self.plan.fields.len())?;
        for index in fields.start..fields.end {
            self.charge(1)?;
            let field = *self
                .catalog
                .fields(fields)
                .and_then(|f| f.get((index - fields.start) as usize))
                .ok_or("invalid cleanup field")?;
            let child = self.node()?;
            self.retain::<Field>(1)?;
            self.plan.fields.push(Field {
                selection: field.selection,
                node: child,
            });
            self.work.push(Work::Enter {
                node: child,
                recipe: field.recipe,
                tracked: self.child(tracked, field.selection)?,
            });
        }
        self.plan.nodes[node.0 as usize].operation = Operation::Fields(glue::Range {
            start,
            end: compact(self.plan.fields.len())?,
        });
        Ok(())
    }
    fn array(&mut self, node: NodeId, path: PathId, element: glue::Id) -> Result<(), &'static str> {
        self.work.push(Work::Finish(node));
        let start = compact(self.plan.intervals.len())?;
        let count = self
            .paths
            .array_segments(path)
            .ok_or("missing array cleanup segments")?
            .len();
        for index in 0..count {
            self.charge(1)?;
            let segment = &self
                .paths
                .array_segments(path)
                .ok_or("missing array cleanup segments")?[index];
            let (start, end) = (segment.start, segment.end);
            let tracked = segment
                .element
                .map_or(Tracking::Bit(segment.bit), Tracking::Path);
            let child = self.node()?;
            self.retain::<Interval>(1)?;
            self.plan.intervals.push(Interval {
                start,
                end,
                condition: Condition::Always,
                element: child,
            });
            self.work.push(Work::Enter {
                node: child,
                recipe: element,
                tracked,
            });
        }
        self.plan.nodes[node.0 as usize].operation = Operation::Array(glue::Range {
            start,
            end: compact(self.plan.intervals.len())?,
        });
        Ok(())
    }
    fn empty(&self, id: NodeId) -> bool {
        matches!(self.plan.nodes[id.0 as usize].operation, Operation::Empty)
    }
    fn finish(&mut self, node: NodeId) -> Result<(), &'static str> {
        let mut empty = true;
        match self.plan.nodes[node.0 as usize].operation {
            Operation::Fields(range) => {
                self.charge((range.end - range.start) as usize)?;
                for field in &self.plan.fields[range.start as usize..range.end as usize] {
                    empty &= self.empty(field.node);
                }
            }
            Operation::Variants(range) => {
                self.charge((range.end - range.start) as usize)?;
                for variant in &self.plan.variants[range.start as usize..range.end as usize] {
                    empty &= self.empty(variant.node);
                }
            }
            Operation::Array(range) => {
                self.charge((range.end - range.start) as usize)?;
                for index in range.start..range.end {
                    let interval = &mut self.plan.intervals[index as usize];
                    let child = &mut self.plan.nodes[interval.element.0 as usize];
                    empty &= matches!(child.operation, Operation::Empty);
                    interval.condition = child.condition;
                    child.condition = Condition::Always;
                }
            }
            _ => return Err("invalid composite cleanup completion"),
        }
        if empty {
            self.plan.nodes[node.0 as usize] = Node {
                condition: Condition::Always,
                operation: Operation::Empty,
            };
        }
        Ok(())
    }
    fn flags(&mut self) -> Result<(), &'static str> {
        self.charge(self.plan.nodes.len() + self.plan.intervals.len())?;
        let count = self
            .plan
            .nodes
            .iter()
            .filter(|n| matches!(n.condition, Condition::Initialized(_)))
            .count()
            + self
                .plan
                .intervals
                .iter()
                .filter(|r| matches!(r.condition, Condition::Initialized(_)))
                .count();
        self.retain::<Flag>(count)?;
        for node in &self.plan.nodes {
            if let Condition::Initialized(flag) = node.condition {
                self.plan.flags.push(flag);
            }
        }
        for interval in &self.plan.intervals {
            if let Condition::Initialized(flag) = interval.condition {
                self.plan.flags.push(flag);
            }
        }
        let count = self.plan.flags.len();
        self.charge(count.saturating_mul(count.max(1).ilog2() as usize + 1))?;
        self.plan.flags.sort_unstable();
        self.plan.flags.dedup();
        Ok(())
    }
}
fn compact(value: usize) -> Result<u32, &'static str> {
    u32::try_from(value).map_err(|_| "too many destruction plan entries")
}
