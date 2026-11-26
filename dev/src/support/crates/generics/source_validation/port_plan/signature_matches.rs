//! Call openings participate in comparison identity, never in a paid witness.
use super::{
    DomainIndex, Work, g,
    nodes::{Arena, Node, NodeId, Region, Rewrite},
    s, uses,
};
use crate::support::{
    crates::owner_catalog::{BodyRef, DeclarationRef},
    retained,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum Opening {
    Header(DeclarationRef),
    /// Shape admission only; all regional coercion remains a fresh proof request.
    Relation {
        body: BodyRef,
    },
    /// Defining body result shape admission; owned opaques match their hidden type.
    DefiningResult {
        body: BodyRef,
    },
    Call {
        body: BodyRef,
        request: u32,
    },
    /// Reifying an item with late-bound regions into a higher-ranked fn pointer.
    Reify {
        binder: g::BinderRef,
        depth: u16,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) struct Pair {
    expected: NodeId,
    actual: NodeId,
    opening: Opening,
}
type ChildPair = (NodeId, NodeId);
type MatchedChildren = (ChildPair, Option<ChildPair>);

type ChildPairs = ((NodeId, NodeId), Option<(NodeId, NodeId)>);

#[derive(Clone, Copy)]
enum Task {
    Enter(Pair),
    Finish(Pair),
}
impl Arena {
    pub(super) fn same(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        expected: NodeId,
        actual: NodeId,
        opening: Opening,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.same_with_opaques(domains, (expected, actual), opening, &[], work)
    }
    pub(super) fn same_defining_result(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: (NodeId, NodeId),
        body: BodyRef,
        opaques: &[(NodeId, NodeId)],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.same_with_opaques(
            domains,
            pair,
            Opening::DefiningResult { body },
            opaques,
            work,
        )
    }
    fn same_with_opaques(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        (expected, actual): (NodeId, NodeId),
        opening: Opening,
        opaques: &[(NodeId, NodeId)],
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let pair = Pair {
            expected,
            actual,
            opening,
        };
        if expected == actual || self.matched.contains(&pair) {
            self.hits = self.hits.saturating_add(1);
            return Ok(());
        }
        let mut pending = Vec::new();
        let result = self.compare(domains, pair, opaques, &mut pending, work);
        self.set_scratch(0, work)?;
        result
    }
    fn compare(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        root: Pair,
        opaques: &[(NodeId, NodeId)],
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        self.push_pair(pending, Task::Enter(root), work)?;
        while let Some(task) = pending.pop() {
            work.charge(1)?;
            match task {
                Task::Finish(pair) => {
                    self.reserve(retained::new_map::<Pair, ()>(1), 0, work)?;
                    self.matched.insert(pair);
                }
                Task::Enter(pair) => {
                    if pair.expected == pair.actual || self.matched.contains(&pair) {
                        self.hits = self.hits.saturating_add(1);
                        continue;
                    }
                    if let Opening::DefiningResult { body } = pair.opening
                        && let Some(&(_, hidden)) =
                            opaques.iter().find(|&&(app, _)| app == pair.expected)
                    {
                        self.push_pair(pending, Task::Finish(pair), work)?;
                        self.push_pair(
                            pending,
                            Task::Enter(Pair {
                                expected: hidden,
                                actual: pair.actual,
                                opening: Opening::Relation { body },
                            }),
                            work,
                        )?;
                        continue;
                    }
                    let expected = *self
                        .nodes
                        .get(pair.expected.0 as usize)
                        .ok_or("source expected type node is absent")?;
                    let actual = *self
                        .nodes
                        .get(pair.actual.0 as usize)
                        .ok_or("source actual type node is absent")?;
                    self.push_pair(pending, Task::Finish(pair), work)?;
                    self.children(domains, pair, (expected, actual), pending, work)?;
                }
            }
        }
        Ok(())
    }
    /// A trait object keeps its outer lifetime in the enclosing frame; its
    /// bounds compare declaration identities before their inner-binder inputs.
    fn dynamic_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        nodes: (Node, Node),
        work: &mut Work,
    ) -> Result<(NodeId, NodeId), &'static str> {
        match nodes {
            (
                Node::Dynamic {
                    bounds: ac,
                    region: ar,
                },
                Node::Dynamic {
                    bounds: bc,
                    region: br,
                },
            ) => {
                if !self.same_region(domains, (ar, br), pair.opening, work)? {
                    return Err("source dynamic type changed its outer lifetime");
                }
                Ok((ac, bc))
            }
            (
                Node::DynamicBounds {
                    recipe: a,
                    arguments: ac,
                },
                Node::DynamicBounds {
                    recipe: b,
                    arguments: bc,
                },
            ) if a == b => Ok((ac, bc)),
            _ => Err("source signature changed its trait object bounds"),
        }
    }
    /// Compare a nominal's complete argument spine after opening its actual lifetimes.
    /// A type argument can itself contain a reference to an opened slot.
    fn children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        nodes: (Node, Node),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let mut opening = pair.opening;
        let (first, second) = match nodes {
            (
                Node::Reference {
                    mutable: a,
                    region: ar,
                    referent: ac,
                },
                Node::Reference {
                    mutable: b,
                    region: br,
                    referent: bc,
                },
            ) => {
                if a != b || !self.same_region(domains, (ar, br), pair.opening, work)? {
                    return Err("source signature reference mutability or exact lifetime differs");
                }
                ((ac, bc), None)
            }
            (Node::Dynamic { .. } | Node::DynamicBounds { .. }, _) => {
                if matches!(nodes.0, Node::DynamicBounds { .. }) {
                    opening = nested_opening(opening)?;
                }
                (self.dynamic_children(domains, pair, nodes, work)?, None)
            }
            (Node::RawPointer { .. } | Node::Slice(_) | Node::Pattern { .. }, _) => {
                (single_child(domains, nodes)?, None)
            }
            (Node::Function { .. } | Node::SignatureInput { .. } | Node::SignatureResult(_), _) => {
                if matches!(nodes.0, Node::Function { .. }) {
                    opening = nested_opening(opening)?;
                }
                callable_children(domains, pair.opening, nodes)?
            }
            (Node::Tuple { elements: a }, Node::Tuple { elements: b }) => ((a, b), None),
            (
                Node::AppliedFunctionItem {
                    declaration: a,
                    arguments: ac,
                },
                Node::AppliedFunctionItem {
                    declaration: b,
                    arguments: bc,
                },
            ) if a == b => ((ac, bc), None),
            (
                Node::Array {
                    element: ae,
                    length: al,
                },
                Node::Array {
                    element: be,
                    length: bl,
                },
            ) => ((ae, be), Some((al, bl))),
            (
                Node::Nominal { .. }
                | Node::OpaqueFuture { .. }
                | Node::NominalRegion { .. }
                | Node::NominalArgument { .. },
                _,
            ) => self.matched_nominal_children(domains, pair, nodes, work)?,
            (
                Node::Projection {
                    member: a,
                    arguments: ac,
                    family: af,
                },
                Node::Projection {
                    member: b,
                    arguments: bc,
                    family: bf,
                },
            ) if a == b => ((ac, bc), Some((af, bf))),
            (Node::Closure { .. }, Node::Closure { .. }) => {
                self.matched_closure_children(opening, nodes, pending, work)?
            }
            (a, b) if a == b => return Ok(()),
            (Node::Projection { .. }, _) | (_, Node::Projection { .. })
                if self.projection_match(domains, pair, nodes, pending, work)? =>
            {
                return Ok(());
            }
            _ => {
                return Err("source signature differs after complete application and late opening");
            }
        };
        for (expected, actual) in second.into_iter().chain([first]) {
            self.push_pair(
                pending,
                Task::Enter(Pair {
                    expected,
                    actual,
                    opening,
                }),
                work,
            )?;
        }
        Ok(())
    }

    fn matched_closure_children(
        &mut self,
        opening: Opening,
        nodes: (Node, Node),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<ChildPairs, &'static str> {
        let (
            Node::Closure {
                closure: ac,
                arguments: aa,
                signature: asig,
                environment: aenv,
            },
            Node::Closure {
                closure: bc,
                arguments: ba,
                signature: bsig,
                environment: benv,
            },
        ) = nodes
        else {
            return Err("source signature differs after complete application and late opening");
        };
        if ac != bc {
            return Err("source signature differs after complete application and late opening");
        }
        self.push_pair(
            pending,
            Task::Enter(Pair {
                expected: asig,
                actual: bsig,
                opening,
            }),
            work,
        )?;
        Ok(((aa, ba), Some((aenv, benv))))
    }
    fn matched_nominal_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        nodes: (Node, Node),
        work: &mut Work,
    ) -> Result<MatchedChildren, &'static str> {
        match nodes {
            (
                Node::Nominal {
                    declaration: a,
                    arguments: ac,
                },
                Node::Nominal {
                    declaration: b,
                    arguments: bc,
                },
            ) if a == b => Ok(((ac, bc), None)),
            (
                Node::OpaqueFuture {
                    declaration: ad,
                    ordinal: ao,
                    arguments: ac,
                },
                Node::OpaqueFuture {
                    declaration: bd,
                    ordinal: bo,
                    arguments: bc,
                },
            ) if ad == bd && ao == bo => Ok(((ac, bc), None)),
            (
                Node::NominalRegion {
                    region: ar,
                    next: an,
                },
                Node::NominalRegion {
                    region: br,
                    next: bn,
                },
            ) => {
                if !self.same_region(domains, (ar, br), pair.opening, work)? {
                    return Err("source signature nominal region argument differs");
                }
                Ok(((an, bn), None))
            }
            (
                Node::NominalArgument { ty: at, next: an },
                Node::NominalArgument { ty: bt, next: bn },
            ) => Ok(((at, bt), Some((an, bn)))),
            _ => Err("source signature differs after complete application and late opening"),
        }
    }
    fn projection_match(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        nodes: (Node, Node),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if domains.declaration_policies && !matches!(pair.opening, Opening::Header(_)) {
            return Ok(true);
        }
        if let (Node::Scalar(crate::support::crates::Scalar::Usize), projection)
        | (projection, Node::Scalar(crate::support::crates::Scalar::Usize)) = nodes
            && self.slice_metadata(domains, projection, work)?
        {
            return Ok(true);
        }
        if self.match_dynamic_binding(domains, pair, (nodes.0, pair.actual, true), pending, work)? {
            return Ok(true);
        }
        self.match_dynamic_binding(
            domains,
            pair,
            (nodes.1, pair.expected, false),
            pending,
            work,
        )
    }
    fn match_dynamic_binding(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        (projection, other, expected_projection): (Node, NodeId, bool),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let Node::Projection {
            member,
            arguments,
            family,
        } = projection
        else {
            return Ok(false);
        };
        if self.nodes.get(family.0 as usize) != Some(&Node::NominalEnd) {
            return Ok(false);
        }
        let Some(&Node::NominalArgument {
            ty: subject,
            next: proj_args,
        }) = self.nodes.get(arguments.0 as usize)
        else {
            return Ok(false);
        };
        let Some(&Node::Dynamic { bounds, .. }) = self.nodes.get(subject.0 as usize) else {
            return Ok(false);
        };
        let Some(&Node::DynamicBounds {
            recipe,
            arguments: mut cursor,
        }) = self.nodes.get(bounds.0 as usize)
        else {
            return Ok(false);
        };
        let rows = domains.headers.rows();
        let columns = &rows.cold().dynamics;
        let row = columns
            .objects
            .get(recipe.0 as usize)
            .ok_or("source dynamic shape is unknown")?;
        if rows.binders[row.binder.0 as usize].count != 0 {
            return Ok(false);
        }
        cursor = self.skip_arguments(cursor, row.principal_arguments, work)?;
        for binding in row.bindings.slice(&columns.bindings)? {
            work.charge(1)?;
            let binding_args = cursor;
            cursor = self.skip_arguments(cursor, binding.arguments, work)?;
            let Some(&Node::NominalArgument { ty: result, next }) =
                self.nodes.get(cursor.0 as usize)
            else {
                return Err("source dynamic binding lost its associated result node");
            };
            cursor = next;
            if binding.member != member {
                continue;
            }
            let saved = pending.len();
            if !self.queue_binding_arguments(
                domains,
                pair,
                (
                    binding_args,
                    proj_args,
                    binding.arguments,
                    expected_projection,
                ),
                pending,
                work,
            )? {
                pending.truncate(saved);
                return Ok(false);
            }
            let (expected, actual) = if expected_projection {
                (result, other)
            } else {
                (other, result)
            };
            self.push_pair(
                pending,
                Task::Enter(Pair {
                    expected,
                    actual,
                    ..pair
                }),
                work,
            )?;
            return Ok(true);
        }
        Ok(false)
    }
    fn queue_binding_arguments(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        pair: Pair,
        (mut binding_args, mut proj_args, count, expected_projection): (NodeId, NodeId, u32, bool),
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        for _ in 0..count {
            work.charge(1)?;
            let (Some(&left), Some(&right)) = (
                self.nodes.get(binding_args.0 as usize),
                self.nodes.get(proj_args.0 as usize),
            ) else {
                return Ok(false);
            };
            match (left, right) {
                (
                    Node::NominalArgument { ty: a, next: an },
                    Node::NominalArgument { ty: b, next: bn },
                ) => {
                    let (expected, actual) = if expected_projection { (a, b) } else { (b, a) };
                    self.push_pair(
                        pending,
                        Task::Enter(Pair {
                            expected,
                            actual,
                            ..pair
                        }),
                        work,
                    )?;
                    binding_args = an;
                    proj_args = bn;
                }
                (
                    Node::NominalRegion {
                        region: ar,
                        next: an,
                    },
                    Node::NominalRegion {
                        region: br,
                        next: bn,
                    },
                ) => {
                    let regions = if expected_projection {
                        (ar, br)
                    } else {
                        (br, ar)
                    };
                    if !self.same_region(domains, regions, pair.opening, work)? {
                        return Ok(false);
                    }
                    binding_args = an;
                    proj_args = bn;
                }
                _ => return Ok(false),
            }
        }
        Ok(self.nodes.get(proj_args.0 as usize) == Some(&Node::NominalEnd))
    }
    fn skip_arguments(
        &self,
        mut cursor: NodeId,
        count: u32,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        for _ in 0..count {
            work.charge(1)?;
            cursor = match self.nodes.get(cursor.0 as usize) {
                Some(&Node::NominalArgument { next, .. } | &Node::NominalRegion { next, .. }) => {
                    next
                }
                _ => return Err("source dynamic argument spine ended early"),
            };
        }
        Ok(cursor)
    }
    fn slice_metadata(
        &self,
        domains: &DomainIndex<'_, '_>,
        projection: Node,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        let Node::Projection {
            member,
            arguments,
            family,
        } = projection
        else {
            return Ok(false);
        };
        if domains.metadata_member != Some(member) {
            return Ok(false);
        }
        work.charge(3)?;
        let Some(Node::NominalArgument { ty, next }) = self.nodes.get(arguments.0 as usize) else {
            return Ok(false);
        };
        Ok(self.nodes.get(next.0 as usize) == Some(&Node::NominalEnd)
            && self.nodes.get(family.0 as usize) == Some(&Node::NominalEnd)
            && matches!(
                self.nodes.get(ty.0 as usize),
                Some(Node::Slice(_) | Node::Str)
            ))
    }

    pub(super) fn same_region(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        (expected, actual): (Region, Region),
        opening: Opening,
        work: &mut Work,
    ) -> Result<bool, &'static str> {
        if matches!(
            opening,
            Opening::Relation { .. } | Opening::DefiningResult { .. }
        ) {
            return Ok(true);
        }
        let expected = opened_region(domains, self, expected, opening, work)?;
        let actual = opened_region(domains, self, actual, opening, work)?;
        if expected == actual {
            return Ok(true);
        }
        if let Opening::Call { body, .. } = opening
            && domains.declaration_policies
        {
            // Imported originals replay this caller-owned equality in their regional proof.
            return self.same_call_region(domains, body, (expected, actual), work);
        }
        Ok(false)
    }
    fn push_pair(
        &mut self,
        tasks: &mut Vec<Task>,
        task: Task,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if tasks.len() == tasks.capacity() {
            let capacity = tasks.capacity().saturating_mul(2).max(4);
            self.set_scratch(
                size_of::<Vec<Task>>().saturating_add(capacity.saturating_mul(size_of::<Task>())),
                work,
            )?;
            work.charge(tasks.len())?;
            tasks
                .try_reserve_exact(capacity.saturating_sub(tasks.len()))
                .map_err(|_| "source signature comparison work-stack allocation failed")?;
            self.set_scratch(
                size_of::<Vec<Task>>()
                    .saturating_add(tasks.capacity().saturating_mul(size_of::<Task>())),
                work,
            )?;
        }
        tasks.push(task);
        Ok(())
    }
}
fn opened_region(
    domains: &DomainIndex<'_, '_>,
    arena: &Arena,
    region: Region,
    context: Opening,
    work: &mut Work,
) -> Result<Region, &'static str> {
    work.charge(1)?;
    let Region::Late { binder, slot } = region else {
        return Ok(region);
    };
    if let Opening::Reify {
        binder: expected,
        depth,
    } = context
    {
        if binder != expected {
            return Err("source reification uses another target's late binder");
        }
        return Ok(Region::ObjectBound { depth, slot });
    }
    let Opening::Call { body, request } = context else {
        return Err("source callable late template has no actual Call opening");
    };
    let rows = domains.headers.rows();
    let original = rows
        .originals
        .get(domains.headers.original_index(body)?)
        .ok_or("source signature comparison has an unknown caller body")?;
    let request = original
        .calls
        .get(request as usize)
        .ok_or("source signature comparison has an unknown Call request")?;
    let opening = request
        .opening
        .ok_or("source signature comparison omitted the actual late opening")?;
    if opening.binder != binder {
        return Err("source signature comparison uses another target's late opening");
    }
    let region = *opening
        .regions
        .slice(&original.openings)?
        .get(slot as usize)
        .ok_or("source signature comparison omitted a complete late argument")?;
    uses::region(domains, arena, region, Rewrite::Original, work)
}
fn nested_opening(opening: Opening) -> Result<Opening, &'static str> {
    match opening {
        Opening::Reify { binder, depth } => Ok(Opening::Reify {
            binder,
            depth: depth
                .checked_add(1)
                .ok_or("source reification binder depth overflow")?,
        }),
        other => Ok(other),
    }
}
fn callable_children(
    domains: &DomainIndex<'_, '_>,
    opening: Opening,
    nodes: (Node, Node),
) -> Result<MatchedChildren, &'static str> {
    match nodes {
        (
            Node::Function {
                signature: ac,
                bound_regions: ab,
                unsafe_: au,
            },
            Node::Function {
                signature: bc,
                bound_regions: bb,
                unsafe_: bu,
            },
        ) if au == bu
            && (ab == bb
                || (domains.declaration_policies
                    && matches!(
                        opening,
                        Opening::Relation { .. } | Opening::DefiningResult { .. }
                    ))) =>
        {
            Ok(((ac, bc), None))
        }
        (Node::SignatureInput { ty: at, next: an }, Node::SignatureInput { ty: bt, next: bn }) => {
            Ok(((at, bt), Some((an, bn))))
        }
        (Node::SignatureResult(ar), Node::SignatureResult(br)) => Ok(((ar, br), None)),
        _ => Err("source signature differs after complete application and late opening"),
    }
}
/// Version thirteen's raw pointer, slice and pattern nodes compare their own
/// form, then their one child.
fn single_child(
    domains: &DomainIndex<'_, '_>,
    nodes: (Node, Node),
) -> Result<(NodeId, NodeId), &'static str> {
    match nodes {
        (
            Node::RawPointer {
                mutable: a,
                pointee: ac,
            },
            Node::RawPointer {
                mutable: b,
                pointee: bc,
            },
        ) => {
            if a != b {
                return Err("source signature raw-pointer mutability differs");
            }
            Ok((ac, bc))
        }
        (Node::Slice(ac), Node::Slice(bc)) => Ok((ac, bc)),
        (
            Node::Pattern {
                base: ac,
                constraint: a,
            },
            Node::Pattern {
                base: bc,
                constraint: b,
            },
        ) => {
            if !same_constraint(domains, a, b) {
                return Err("source signature pattern constraint differs");
            }
            Ok((ac, bc))
        }
        _ => Err("source signature differs after complete application and late opening"),
    }
}
/// Two pattern constraints agree when both are `!null` or both name the same
/// canonical interval, whichever range rows carry it.
fn same_constraint(
    domains: &DomainIndex<'_, '_>,
    first: s::PatternConstraint,
    second: s::PatternConstraint,
) -> bool {
    let ranges = domains.headers.rows().pattern_ranges();
    match (first, second) {
        (s::PatternConstraint::NonNull, s::PatternConstraint::NonNull) => true,
        (s::PatternConstraint::Range(a), s::PatternConstraint::Range(b)) => {
            let (a, b) = (ranges.get(a.0 as usize), ranges.get(b.0 as usize));
            a.is_some() && a == b
        }
        _ => false,
    }
}
