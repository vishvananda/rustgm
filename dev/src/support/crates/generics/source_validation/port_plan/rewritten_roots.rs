//! One completed template node per exact type/application/context use.
use super::{
    Context, DomainIndex, Work, g,
    nodes::{Arena, Key, Node, NodeId, Rewrite},
    s, uses,
};

#[derive(Clone, Copy)]
enum Recipe {
    Leaf(Node),
    Existing(NodeId),
    Alias(Key),
    Reference {
        mutable: bool,
        region: super::nodes::Region,
        child: Key,
    },
    /// Version thirteen: `*const T`, `*mut T` and `[T]` follow one child.
    Pointer {
        mutable: bool,
        child: Key,
    },
    Slice(Key),
    /// Version thirteen: a pattern follows its one base child.
    Pattern {
        constraint: s::PatternConstraint,
        child: Key,
    },
    Function(g::Signature),
    Nominal(s::DeclarationAddress, g::Range),
    Projection(s::ProjectionRef),
    Tuple(g::Range),
}
#[derive(Clone, Copy)]
enum Task {
    Enter(Key),
    Finish(Key, Recipe),
}
impl Arena {
    pub(super) fn root(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        if let Some(&node) = self.completed.get(&key) {
            self.hits = self.hits.saturating_add(1);
            return Ok(node);
        }
        let mut pending = Vec::new();
        let result = self.expand(domains, key, &mut pending, work);
        self.set_scratch(0, work)?;
        result
    }
    fn expand(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        root: Key,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        self.push(pending, Task::Enter(root), work)?;
        while let Some(task) = pending.pop() {
            work.charge(1)?;
            match task {
                Task::Enter(key) => {
                    if self.completed.contains_key(&key) {
                        self.hits = self.hits.saturating_add(1);
                        continue;
                    }
                    let recipe = self.recipe(domains, key, work)?;
                    self.push(pending, Task::Finish(key, recipe), work)?;
                    match recipe {
                        Recipe::Alias(child)
                        | Recipe::Reference { child, .. }
                        | Recipe::Pointer { child, .. }
                        | Recipe::Slice(child)
                        | Recipe::Pattern { child, .. } => {
                            self.push(pending, Task::Enter(child), work)?;
                        }
                        Recipe::Nominal(declaration, arguments) => {
                            let regions = Regions::of(declaration);
                            self.nominal_children(domains, key, arguments, regions, pending, work)?;
                        }
                        Recipe::Projection(reference) => {
                            let (_, subject, arguments) = projection(domains, reference)?;
                            let regions = Regions::Rejected;
                            self.nominal_children(domains, key, arguments, regions, pending, work)?;
                            let subject =
                                uses::key(domains, subject, key.rewrite, key.caller, work)?;
                            self.push(pending, Task::Enter(subject), work)?;
                        }
                        Recipe::Function(signature) => {
                            self.function_children(domains, key, signature, pending, work)?;
                        }
                        Recipe::Tuple(elements) => {
                            for &element in
                                elements.slice(&domains.headers.rows().roots)?.iter().rev()
                            {
                                let child =
                                    uses::key(domains, element, key.rewrite, key.caller, work)?;
                                self.push(pending, Task::Enter(child), work)?;
                            }
                        }
                        Recipe::Leaf(_) | Recipe::Existing(_) => {}
                    }
                }
                Task::Finish(key, recipe) => {
                    let node = match recipe {
                        Recipe::Leaf(node) => self.intern(node, work)?,
                        Recipe::Existing(node) => node,
                        Recipe::Alias(child) => self.child(child, work)?,
                        Recipe::Reference {
                            mutable,
                            region,
                            child,
                        } => {
                            let referent = self.child(child, work)?;
                            self.intern(
                                Node::Reference {
                                    mutable,
                                    region,
                                    referent,
                                },
                                work,
                            )?
                        }
                        Recipe::Pointer { .. } | Recipe::Slice(_) | Recipe::Pattern { .. } => {
                            self.single_child(recipe, work)?
                        }
                        Recipe::Nominal(declaration, arguments) => {
                            let regions = Regions::of(declaration);
                            let arguments =
                                self.argument_chain(domains, key, arguments, regions, work)?;
                            self.intern(
                                Node::Nominal {
                                    declaration,
                                    arguments,
                                },
                                work,
                            )?
                        }
                        Recipe::Projection(reference) => {
                            self.finish_projection(domains, key, reference, work)?
                        }
                        Recipe::Function(signature) => {
                            self.finish_function(domains, key, signature, work)?
                        }
                        Recipe::Tuple(elements) => {
                            self.finish_tuple(domains, key, elements, work)?
                        }
                    };
                    self.remember(key, node, work)?;
                }
            }
        }
        self.child(root, work)
    }
    fn nominal_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        arguments: g::Range,
        regions: Regions,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        for argument in arguments
            .slice(&domains.headers.rows().arguments)?
            .iter()
            .rev()
        {
            if let Some(child) = nominal_child(domains, key, *argument, regions, work)? {
                self.push(pending, Task::Enter(child), work)?;
            }
        }
        Ok(())
    }
    fn argument_chain(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        arguments: g::Range,
        regions: Regions,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let mut tail = self.intern(Node::NominalEnd, work)?;
        for argument in arguments
            .slice(&domains.headers.rows().arguments)?
            .iter()
            .rev()
        {
            let node = match *argument {
                s::Argument::Region(region) if regions == Regions::Local => Node::NominalRegion {
                    region: self.rewritten_region(domains, region, key.rewrite, work)?,
                    next: tail,
                },
                s::Argument::Const(term) => Node::NominalArgument {
                    ty: self.const_node(domains, term, key.rewrite, work)?,
                    next: tail,
                },
                _ => {
                    let child = nominal_child(domains, key, *argument, regions, work)?
                        .ok_or("source nominal type argument lost its child")?;
                    Node::NominalArgument {
                        ty: self.child(child, work)?,
                        next: tail,
                    }
                }
            };
            tail = self.intern(node, work)?;
        }
        Ok(tail)
    }
    /// Version thirteen: a const argument's node. A value is canonical by
    /// validation; a rigid term follows its slot's rewrite as a type
    /// parameter does, and an applied term is its caller's original term.
    fn const_node(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        term: s::ConstRef,
        rewrite: Rewrite,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        let rows = domains.headers.rows();
        let term = *rows
            .consts()
            .get(term.0 as usize)
            .ok_or("source const argument has no term row")?;
        let slot = match term {
            s::ConstTerm::Value(value) => {
                let scalar = s::const_value(&rows.types, value)?;
                let bytes = value.bits.to_le_bytes();
                let mut bits = [0; 4];
                for (lane, chunk) in bits.iter_mut().zip(bytes.as_chunks::<4>().0) {
                    *lane = u32::from_le_bytes(*chunk);
                }
                return self.intern(Node::ConstValue { scalar, bits }, work);
            }
            s::ConstTerm::Parameter(slot) => slot,
        };
        match rewrite {
            Rewrite::Nominal { owner, arguments } => {
                let position = domains
                    .headers
                    .scopes()
                    .type_argument_position(owner, slot, work)?;
                self.nominal_argument(arguments, position, work)
            }
            Rewrite::Application(_) => {
                match uses::applied_argument(domains, rewrite, slot, work)? {
                    s::Argument::Const(actual) => {
                        self.const_node(domains, actual, Rewrite::Original, work)
                    }
                    _ => Err("source signature application changed a const argument's kind"),
                }
            }
            Rewrite::Original | Rewrite::Public(_) => self.intern(Node::Parameter(slot), work),
        }
    }
    fn finish_tuple(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        elements: g::Range,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let mut tail = self.intern(Node::NominalEnd, work)?;
        for &element in elements.slice(&domains.headers.rows().roots)?.iter().rev() {
            let child = uses::key(domains, element, key.rewrite, key.caller, work)?;
            let ty = self.child(child, work)?;
            tail = self.intern(Node::NominalArgument { ty, next: tail }, work)?;
        }
        self.intern(Node::Tuple { elements: tail }, work)
    }
    fn finish_projection(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        reference: s::ProjectionRef,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let (member, subject, arguments) = projection(domains, reference)?;
        let tail = self.argument_chain(domains, key, arguments, Regions::Rejected, work)?;
        let subject = uses::key(domains, subject, key.rewrite, key.caller, work)?;
        let ty = self.child(subject, work)?;
        let arguments = self.intern(Node::NominalArgument { ty, next: tail }, work)?;
        self.intern(Node::Projection { member, arguments }, work)
    }
    fn recipe(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        work: &mut Work,
    ) -> Result<Recipe, &'static str> {
        work.charge(1)?;
        Ok(
            match *domains
                .headers
                .rows()
                .types
                .get(key.root.0 as usize)
                .ok_or("source input type root is unknown")?
            {
                s::Type::Nominal {
                    declaration,
                    arguments,
                } => Recipe::Nominal(declaration, arguments),
                s::Type::Projection(reference) => {
                    projection(domains, reference)?;
                    Recipe::Projection(reference)
                }
                s::Type::OpaqueFuture(_) => {
                    return Err(
                        "declared result in ordinary body operations requires its complete application transport",
                    );
                }
                s::Type::Scalar(scalar) => Recipe::Leaf(Node::Scalar(scalar)),
                s::Type::Str => Recipe::Leaf(Node::Str),
                s::Type::Never => Recipe::Leaf(Node::Never),
                s::Type::Tuple(elements) => Recipe::Tuple(elements),
                s::Type::Parameter(slot) if matches!(key.rewrite, Rewrite::Nominal { .. }) => {
                    let Rewrite::Nominal { owner, arguments } = key.rewrite else {
                        return Err("source nominal parameter lost its actual rewrite");
                    };
                    let position = domains
                        .headers
                        .scopes()
                        .type_argument_position(owner, slot, work)?;
                    Recipe::Existing(self.nominal_argument(arguments, position, work)?)
                }
                s::Type::Parameter(slot) => match uses::argument(domains, key, slot, work)? {
                    Some(child) => Recipe::Alias(child),
                    None => Recipe::Leaf(Node::Parameter(slot)),
                },
                s::Type::Reference {
                    mutable,
                    region,
                    referent,
                } => Recipe::Reference {
                    mutable,
                    region: self.rewritten_region(domains, region, key.rewrite, work)?,
                    child: uses::key(domains, referent, key.rewrite, key.caller, work)?,
                },
                s::Type::RawPointer { mutable, pointee } => Recipe::Pointer {
                    mutable,
                    child: uses::key(domains, pointee, key.rewrite, key.caller, work)?,
                },
                s::Type::Slice(element) => {
                    Recipe::Slice(uses::key(domains, element, key.rewrite, key.caller, work)?)
                }
                s::Type::Pattern { base, constraint } => Recipe::Pattern {
                    constraint,
                    child: uses::key(domains, base, key.rewrite, key.caller, work)?,
                },
                s::Type::FunctionItem(application) if key.rewrite == Rewrite::Original => {
                    Recipe::Leaf(Node::FunctionItem(application))
                }
                s::Type::FunctionItem(_) => {
                    return Err(
                        "source nested function-item substitution requires a wider profile",
                    );
                }
                s::Type::Function(signature) => {
                    function_domain(domains, key, signature, work)?;
                    Recipe::Function(signature)
                }
                s::Type::ApplicationSignature(_) => {
                    return Err("source higher-order type inputs require a complete wider profile");
                }
            },
        )
    }
    /// Version thirteen's raw pointer, slice and pattern nodes over their one
    /// completed child.
    fn single_child(&mut self, recipe: Recipe, work: &mut Work) -> Result<NodeId, &'static str> {
        let node = match recipe {
            Recipe::Pointer { mutable, child } => Node::RawPointer {
                mutable,
                pointee: self.child(child, work)?,
            },
            Recipe::Slice(child) => Node::Slice(self.child(child, work)?),
            Recipe::Pattern { constraint, child } => Node::Pattern {
                base: self.child(child, work)?,
                constraint,
            },
            _ => return Err("source single-child recipe changed its form"),
        };
        self.intern(node, work)
    }
    fn child(&self, key: Key, work: &mut Work) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        self.completed
            .get(&key)
            .copied()
            .ok_or("source input type child was not completed")
    }
    fn function_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        signature: g::Signature,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let parameters = signature.parameters.slice(&domains.headers.rows().roots)?;
        let children = parameters.len().saturating_add(1);
        work.charge(children)?;
        let capacity = pending.capacity().max(
            pending
                .len()
                .saturating_add(children)
                .saturating_mul(2)
                .max(4),
        );
        self.reserve(
            0,
            size_of::<Vec<Task>>().saturating_add(capacity.saturating_mul(size_of::<Task>())),
            work,
        )?;
        let result = uses::key(domains, signature.result, key.rewrite, key.caller, work)?;
        self.push(pending, Task::Enter(result), work)?;
        for &root in parameters.iter().rev() {
            let child = uses::key(domains, root, key.rewrite, key.caller, work)?;
            self.push(pending, Task::Enter(child), work)?;
        }
        Ok(())
    }
    fn finish_function(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        signature: g::Signature,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let parameters = signature.parameters.slice(&domains.headers.rows().roots)?;
        work.charge(parameters.len().saturating_add(1))?;
        let result = uses::key(domains, signature.result, key.rewrite, key.caller, work)?;
        let result = self.child(result, work)?;
        let mut tail = self.intern(Node::SignatureResult(result), work)?;
        for &root in parameters.iter().rev() {
            let child = uses::key(domains, root, key.rewrite, key.caller, work)?;
            let ty = self.child(child, work)?;
            tail = self.intern(Node::SignatureInput { ty, next: tail }, work)?;
        }
        self.intern(
            Node::Function {
                signature: tail,
                unsafe_: signature.unsafe_,
            },
            work,
        )
    }
    fn push(
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
                .map_err(|_| "source input type work-stack allocation failed")?;
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

fn function_domain(
    domains: &DomainIndex<'_, '_>,
    key: Key,
    signature: g::Signature,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    let rows = domains.headers.rows();
    let (owner, context) = match key.rewrite {
        Rewrite::Nominal { .. } => {
            return Err("source nominal callable field requires its wider binder profile");
        }
        Rewrite::Original => {
            let caller = key
                .caller
                .ok_or("source Function omitted its actual formation owner")?;
            let header = domains.function(caller.owner, work)?;
            (header.declaration, caller)
        }
        Rewrite::Public(owner) => {
            let header = domains.function(owner, work)?;
            (
                header.declaration,
                Context {
                    owner: header.scope,
                    frame: Some(header.original.binder),
                    body: None,
                },
            )
        }
        Rewrite::Application(application) => {
            let application = rows
                .function_applications
                .get(application.0 as usize)
                .ok_or("source Function rewrite has no actual application")?;
            let header = domains.function(application.declaration.declaration, work)?;
            (
                header.declaration,
                Context {
                    owner: header.scope,
                    frame: Some(header.callable.binder),
                    body: None,
                },
            )
        }
    };
    let frame = rows
        .binders
        .get(signature.binder.0 as usize)
        .ok_or("source Function has no actual callable frame")?;
    if frame.kind != s::BinderKind::CallableType
        || frame.count != 0
        || frame.parent.is_some()
        || frame.owner != owner
        || signature.convention != g::Convention::Rust
        || signature.variadic
        || !domains.types.region_independent(key.root, work)?
    {
        return Err("source Function input lost its exact zero-slot Rust formation domain");
    }
    domains.types.root(domains.headers, context, key.root, work)
}

/// The projection's Self and complete trait arguments; the member alone names
/// its trait because every projection row was checked against its header.
fn projection(
    domains: &DomainIndex<'_, '_>,
    reference: s::ProjectionRef,
) -> Result<(s::DeclarationAddress, g::TypeRef, g::Range), &'static str> {
    let rows = domains.headers.rows();
    let projection = rows
        .projections
        .get(reference.0 as usize)
        .ok_or("source input projection is unknown")?;
    let application = rows
        .trait_applications
        .get(projection.trait_application.0 as usize)
        .ok_or("source input projection lost its trait application")?;
    if projection.arguments.start != projection.arguments.end
        || application.subject != projection.subject
    {
        return Err("source input projection changed its Self or family arguments");
    }
    Ok((
        projection.declaration,
        projection.subject,
        application.arguments,
    ))
}

/// Version thirteen: only a local nominal's leading region arguments enter a
/// spine. Imported nominal and projection region arguments stay rejected.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Regions {
    Local,
    Rejected,
}
impl Regions {
    const fn of(declaration: s::DeclarationAddress) -> Self {
        if declaration.artifact.0 == 0 {
            Self::Local
        } else {
            Self::Rejected
        }
    }
}
fn nominal_child(
    domains: &DomainIndex<'_, '_>,
    key: Key,
    argument: s::Argument,
    regions: Regions,
    work: &mut Work,
) -> Result<Option<Key>, &'static str> {
    match argument {
        s::Argument::Type(ty) => uses::key(domains, ty, key.rewrite, key.caller, work).map(Some),
        s::Argument::Region(_) if regions == Regions::Local => Ok(None),
        s::Argument::Region(_) => {
            Err("source nominal input planning requires its wider regional profile")
        }
        // A const term's type is a primitive row: it plans no input.
        s::Argument::Const(_) => Ok(None),
    }
}
