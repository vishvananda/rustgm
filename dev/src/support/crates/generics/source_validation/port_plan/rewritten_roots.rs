//! One completed template node per exact type/application/context use.
use super::{
    DomainIndex, Work,
    nodes::{Arena, Key, Node, NodeId, Rewrite},
    s, uses,
};

#[derive(Clone, Copy)]
enum Recipe {
    Leaf(Node),
    Alias(Key),
    Reference {
        mutable: bool,
        region: super::nodes::Region,
        child: Key,
    },
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
                    if let Recipe::Alias(child) | Recipe::Reference { child, .. } = recipe {
                        self.push(pending, Task::Enter(child), work)?;
                    }
                }
                Task::Finish(key, recipe) => {
                    let node = match recipe {
                        Recipe::Leaf(node) => self.intern(node, work)?,
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
                    };
                    self.remember(key, node, work)?;
                }
            }
        }
        self.child(root, work)
    }
    fn recipe(
        &self,
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
                s::Type::Scalar(scalar) => Recipe::Leaf(Node::Scalar(scalar)),
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
                    region: uses::region(domains, self, region, key.rewrite, work)?,
                    child: uses::key(domains, referent, key.rewrite, key.caller, work)?,
                },
                s::Type::FunctionItem(application) if key.rewrite == Rewrite::Original => {
                    Recipe::Leaf(Node::FunctionItem(application))
                }
                s::Type::FunctionItem(_) => {
                    return Err(
                        "source nested function-item substitution requires a wider profile",
                    );
                }
                s::Type::Function(_) | s::Type::ApplicationSignature(_) => {
                    return Err("source higher-order type inputs require a complete wider profile");
                }
            },
        )
    }
    fn child(&self, key: Key, work: &mut Work) -> Result<NodeId, &'static str> {
        work.charge(1)?;
        self.completed
            .get(&key)
            .copied()
            .ok_or("source input type child was not completed")
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
