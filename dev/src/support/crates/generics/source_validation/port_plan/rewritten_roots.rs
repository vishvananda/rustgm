//! One completed template node per exact type/application/context use.
use super::{
    super::{OriginalHeader, Target},
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
    /// Version thirteen: `[T; N]` follows its element child; its length is a
    /// term, not a type child.
    Array {
        child: Key,
        length: s::ArrayLengthRef,
    },
    /// Version thirteen: a pattern follows its one base child.
    Pattern {
        constraint: s::PatternConstraint,
        child: Key,
    },
    Function(g::Signature),
    Application(ArgumentOwner, g::Range),
    Closure(s::ClosureTypeRef),
    Projection(s::ProjectionRef),
    Tuple(g::Range),
}
#[derive(Clone, Copy)]
enum ArgumentOwner {
    Nominal(s::DeclarationAddress),
    Dynamic(s::DynamicTypeRef),
    OpaqueFuture(s::OpaqueFutureRef),
    OpaqueResult(s::OpaqueResultRef),
}
impl ArgumentOwner {
    const fn regions(self, declaration_policies: bool) -> Regions {
        match self {
            Self::Nominal(declaration) if !declaration_policies => Regions::of(declaration),
            Self::Nominal(_) | Self::Dynamic(_) | Self::OpaqueFuture(_) | Self::OpaqueResult(_) => {
                Regions::Local
            }
        }
    }
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
                        | Recipe::Array { child, .. }
                        | Recipe::Pattern { child, .. } => {
                            self.push(pending, Task::Enter(child), work)?;
                        }
                        Recipe::Application(owner, arguments) => {
                            self.nominal_children(
                                domains,
                                key,
                                arguments,
                                owner.regions(domains.declaration_policies),
                                pending,
                                work,
                            )?;
                        }
                        Recipe::Closure(reference) => {
                            self.closure_children(domains, key, reference, pending, work)?;
                        }
                        Recipe::Projection(reference) => {
                            self.projection_children(domains, key, reference, pending, work)?;
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
                        Recipe::Reference { .. }
                        | Recipe::Pointer { .. }
                        | Recipe::Slice(_)
                        | Recipe::Array { .. }
                        | Recipe::Pattern { .. } => {
                            self.single_child(domains, key, recipe, work)?
                        }
                        Recipe::Application(owner, arguments) => {
                            self.finish_application(domains, key, owner, arguments, work)?
                        }
                        Recipe::Closure(reference) => {
                            self.finish_closure(domains, key, reference, work)?
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
    fn projection_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        reference: s::ProjectionRef,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let projection = projection(domains, reference)?;
        let regions = if domains.declaration_policies {
            Regions::Local
        } else {
            Regions::Rejected
        };
        self.nominal_children(domains, key, projection.arguments, regions, pending, work)?;
        self.nominal_children(
            domains,
            key,
            projection.family,
            Regions::Local,
            pending,
            work,
        )?;
        let subject = uses::key(domains, projection.subject, key.rewrite, key.caller, work)?;
        self.push(pending, Task::Enter(subject), work)
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
    pub(super) fn const_node(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        term: s::ConstRef,
        rewrite: Rewrite,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let term = *domains
            .headers
            .rows()
            .consts()
            .get(term.0 as usize)
            .ok_or("source const argument has no term row")?;
        self.term_node(domains, term, rewrite, work)
    }
    /// A const argument's or an array length's term node.
    fn term_node(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        term: s::ConstTerm,
        rewrite: Rewrite,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        let rows = domains.headers.rows();
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
            s::ConstTerm::Variant(variant) => {
                return self.intern(Node::ConstVariant(variant.declaration), work);
            }
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
    fn finish_application(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        owner: ArgumentOwner,
        arguments: g::Range,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let arguments = self.argument_chain(
            domains,
            key,
            arguments,
            owner.regions(domains.declaration_policies),
            work,
        )?;
        let node = match owner {
            ArgumentOwner::Nominal(declaration) => Node::Nominal {
                declaration,
                arguments,
            },
            ArgumentOwner::Dynamic(reference) => {
                let recipe = dynamic_recipe(domains, reference)?;
                let canonical = self.canonical_dynamic(domains, reference, work)?;
                let bounds = self.intern(
                    Node::DynamicBounds {
                        recipe: canonical,
                        arguments,
                    },
                    work,
                )?;
                let region = self.rewritten_region(domains, recipe.region, key.rewrite, work)?;
                Node::Dynamic { bounds, region }
            }
            ArgumentOwner::OpaqueFuture(reference) => {
                let opaque = domains
                    .headers
                    .rows()
                    .opaque_futures
                    .get(reference.0 as usize)
                    .ok_or("source opaque future recipe is unknown")?;
                Node::OpaqueFuture {
                    declaration: opaque.declaration,
                    ordinal: opaque.ordinal,
                    arguments,
                }
            }
            ArgumentOwner::OpaqueResult(reference) => {
                let opaque = domains
                    .headers
                    .rows()
                    .cold()
                    .opaques
                    .applications
                    .get(reference.0 as usize)
                    .ok_or("source opaque result recipe is unknown")?;
                Node::OpaqueFuture {
                    declaration: opaque.declaration,
                    ordinal: opaque.ordinal,
                    arguments,
                }
            }
        };
        self.intern(node, work)
    }

    fn closure_children(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        reference: s::ClosureTypeRef,
        pending: &mut Vec<Task>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        let closure = closure_recipe(domains, reference)?;
        self.nominal_children(
            domains,
            key,
            closure.arguments,
            Regions::Local,
            pending,
            work,
        )?;
        let environment = uses::key(domains, closure.environment, key.rewrite, key.caller, work)?;
        self.push(pending, Task::Enter(environment), work)?;
        let signature = uses::key(domains, closure.signature, key.rewrite, key.caller, work)?;
        self.push(pending, Task::Enter(signature), work)
    }

    fn finish_closure(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        reference: s::ClosureTypeRef,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let closure = closure_recipe(domains, reference)?;
        let arguments =
            self.argument_chain(domains, key, closure.arguments, Regions::Local, work)?;
        let signature = uses::key(domains, closure.signature, key.rewrite, key.caller, work)?;
        let signature = self.child(signature, work)?;
        let environment = uses::key(domains, closure.environment, key.rewrite, key.caller, work)?;
        let environment = self.child(environment, work)?;
        self.intern(
            Node::Closure {
                closure: closure.closure,
                arguments,
                signature,
                environment,
            },
            work,
        )
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
        let projection = projection(domains, reference)?;
        let regions = if domains.declaration_policies {
            Regions::Local
        } else {
            Regions::Rejected
        };
        let tail = self.argument_chain(domains, key, projection.arguments, regions, work)?;
        let family = self.argument_chain(domains, key, projection.family, Regions::Local, work)?;
        let subject = uses::key(domains, projection.subject, key.rewrite, key.caller, work)?;
        let ty = self.child(subject, work)?;
        let arguments = self.intern(Node::NominalArgument { ty, next: tail }, work)?;
        self.intern(
            Node::Projection {
                member: projection.member,
                arguments,
                family,
            },
            work,
        )
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
                } => Recipe::Application(ArgumentOwner::Nominal(declaration), arguments),
                s::Type::Dynamic(reference) => Recipe::Application(
                    ArgumentOwner::Dynamic(reference),
                    dynamic_recipe(domains, reference)?.arguments,
                ),
                s::Type::Projection(reference) => Recipe::Projection(reference),
                row @ (s::Type::OpaqueFuture(_) | s::Type::OpaqueResult(_)) => {
                    opaque_recipe(domains, row)?
                }
                s::Type::Closure(reference) => {
                    if !domains.declaration_policies {
                        return Err("source closure transport requires version fourteen");
                    }
                    Recipe::Closure(reference)
                }
                s::Type::Scalar(scalar) => Recipe::Leaf(Node::Scalar(scalar)),
                s::Type::F16 => Recipe::Leaf(Node::F16),
                s::Type::F128 => Recipe::Leaf(Node::F128),
                s::Type::Foreign(declaration) => Recipe::Leaf(Node::Foreign(declaration)),
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
                s::Type::Array { element, length } => Recipe::Array {
                    child: uses::key(domains, element, key.rewrite, key.caller, work)?,
                    length,
                },
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
    /// A reference, raw pointer, slice, array or pattern node over its one
    /// completed child. An array's length is its term's node under the key's
    /// rewrite.
    fn single_child(
        &mut self,
        domains: &DomainIndex<'_, '_>,
        key: Key,
        recipe: Recipe,
        work: &mut Work,
    ) -> Result<NodeId, &'static str> {
        let node = match recipe {
            Recipe::Reference {
                mutable,
                region,
                child,
            } => Node::Reference {
                mutable,
                region,
                referent: self.child(child, work)?,
            },
            Recipe::Pointer { mutable, child } => Node::RawPointer {
                mutable,
                pointee: self.child(child, work)?,
            },
            Recipe::Slice(child) => Node::Slice(self.child(child, work)?),
            Recipe::Array { child, length } => {
                let term = *domains
                    .headers
                    .rows()
                    .array_lengths()
                    .get(length.0 as usize)
                    .ok_or("source array type has no length row")?;
                Node::Array {
                    element: self.child(child, work)?,
                    length: self.term_node(domains, term, key.rewrite, work)?,
                }
            }
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
        let bound_regions = domains
            .headers
            .rows()
            .binders
            .get(signature.binder.0 as usize)
            .ok_or("source Function has no actual callable frame")?
            .count;
        self.intern(
            Node::Function {
                signature: tail,
                bound_regions,
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

fn closure_recipe(
    domains: &DomainIndex<'_, '_>,
    reference: s::ClosureTypeRef,
) -> Result<s::ClosureType, &'static str> {
    domains
        .headers
        .rows()
        .closures()
        .and_then(|closures| closures.types.get(reference.0 as usize))
        .copied()
        .ok_or("source closure recipe is unknown")
}

fn opaque_recipe(domains: &DomainIndex<'_, '_>, row: s::Type) -> Result<Recipe, &'static str> {
    if !domains.declaration_policies {
        return Err(
            "declared result in ordinary body operations requires its complete application transport",
        );
    }
    let rows = domains.headers.rows();
    match row {
        s::Type::OpaqueFuture(reference) => {
            let opaque = rows
                .opaque_futures
                .get(reference.0 as usize)
                .ok_or("source opaque future recipe is unknown")?;
            Ok(Recipe::Application(
                ArgumentOwner::OpaqueFuture(reference),
                opaque.captures,
            ))
        }
        s::Type::OpaqueResult(reference) => {
            let opaque = rows
                .cold()
                .opaques
                .applications
                .get(reference.0 as usize)
                .ok_or("source opaque result recipe is unknown")?;
            Ok(Recipe::Application(
                ArgumentOwner::OpaqueResult(reference),
                opaque.captures,
            ))
        }
        _ => Err("source opaque recipe changed its form"),
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
        // Version thirteen: a nominal field's closed callable frame belongs to
        // exactly that nominal. Version fourteen admits higher-ranked and
        // lifetime-capturing nominal callable frames.
        Rewrite::Nominal { owner, .. } => {
            if !domains.declaration_policies
                && rows
                    .binders
                    .get(signature.binder.0 as usize)
                    .is_some_and(|frame| frame.count != 0)
            {
                return Err("source nominal callable field requires its wider binder profile");
            }
            (owner, nominal_frame(domains, owner, work)?)
        }
        Rewrite::Original => {
            let caller = key
                .caller
                .ok_or("source Function omitted its actual formation owner")?;
            let owner = match domains.original_header(caller.owner, work)? {
                OriginalHeader::Function(header) => header.declaration,
                OriginalHeader::Constant { .. } => caller.owner,
                OriginalHeader::Closure { header, .. } => header.owner,
                OriginalHeader::Anonymous { header, .. } => header.owner,
            };
            (owner, caller)
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
            match domains.target(application.declaration.declaration, work)? {
                Target::Function(header) => (
                    header.declaration,
                    Context {
                        owner: header.scope,
                        frame: Some(header.callable.binder),
                        body: None,
                    },
                ),
                // A called constructor's payload is its nominal's fields,
                // whose callable frames that nominal owns.
                Target::Constructor { enumeration, .. } => (
                    enumeration.declaration,
                    nominal_frame(domains, enumeration.declaration, work)?,
                ),
            }
        }
    };
    let frame = rows
        .binders
        .get(signature.binder.0 as usize)
        .ok_or("source Function has no actual callable frame")?;
    let valid_frame = if domains.declaration_policies {
        if let Some(parent) = frame.parent {
            match context.frame {
                Some(root) => domains.binders.ancestor(root, parent, work)?,
                None => false,
            }
        } else {
            domains.types.region_independent(key.root, work)?
        }
    } else {
        frame.count == 0
            && frame.parent.is_none()
            && domains.types.region_independent(key.root, work)?
    };
    if frame.kind != s::BinderKind::CallableType
        || !valid_frame
        || frame.owner != owner
        || signature.convention != g::Convention::Rust
        || signature.variadic
    {
        return Err("source Function input lost its exact zero-slot Rust formation domain");
    }
    let root_context = Context {
        frame: frame.parent.or(context.frame),
        ..context
    };
    domains
        .types
        .root(domains.headers, root_context, key.root, work)
}

/// A nominal's header context, which forms its field roots.
fn nominal_frame(
    domains: &DomainIndex<'_, '_>,
    owner: crate::support::crates::owner_catalog::DeclarationRef,
    work: &mut Work,
) -> Result<Context, &'static str> {
    Ok(Context {
        owner,
        frame: Some(domains.binders.declaration(owner, work)?),
        body: None,
    })
}
/// Complete trait and associated arguments remain independent, since family
/// lifetimes are substituted in the use's binder rather than the trait's.
struct Projection {
    member: s::DeclarationAddress,
    subject: g::TypeRef,
    arguments: g::Range,
    family: g::Range,
}

fn projection(
    domains: &DomainIndex<'_, '_>,
    reference: s::ProjectionRef,
) -> Result<Projection, &'static str> {
    let rows = domains.headers.rows();
    let projection = rows
        .projections
        .get(reference.0 as usize)
        .ok_or("source input projection is unknown")?;
    let application = rows
        .trait_applications
        .get(projection.trait_application.0 as usize)
        .ok_or("source input projection lost its trait application")?;
    let has_family = projection.arguments.start != projection.arguments.end;
    if application.subject != projection.subject
        || has_family && (!domains.declaration_policies || projection.declaration.artifact.0 != 0)
    {
        return Err("source input projection changed its Self or family arguments");
    }
    Ok(Projection {
        member: projection.declaration,
        subject: projection.subject,
        arguments: application.arguments,
        family: projection.arguments,
    })
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
fn dynamic_recipe(
    domains: &DomainIndex<'_, '_>,
    reference: s::DynamicTypeRef,
) -> Result<s::DynamicType, &'static str> {
    domains
        .headers
        .rows()
        .cold()
        .dynamics
        .objects
        .get(reference.0 as usize)
        .copied()
        .ok_or("source type port lost its checked dynamic recipe")
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

#[cfg(test)]
#[path = "../../../../../../tests/invariants/fielded_callable_rows.rs"]
mod tests;
