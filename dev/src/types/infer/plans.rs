//! Compiled signature templates: each generic callable's signature is walked
//! once per store into postorder construction steps over its parameter-bearing
//! nodes, so every later call replays them without materializing the template.
use super::{
    Constructor, Context, Error, Instantiation, Store, TermId, TypeId, arguments::Argument,
};
use crate::support::hash::HashMap;
use crate::types::{
    ItemId, Kind, RegionId, arguments::Argument as Canonical, parameters::ParameterId,
};

#[derive(Clone, Copy)]
enum Step {
    /// A parameter-free subtree keeps its canonical identity.
    Known(TypeId),
    Parameter(ParameterId, u16),
    Construct(Constructor, u32),
    /// A nominal with lifetime and type arguments; its slots follow in `slots`.
    Nominal(ItemId, u32, u32),
}
#[derive(Clone, Copy)]
enum Slot {
    Lifetime(RegionId),
    Type,
}
/// Templates whose nodes are all context-free compile; others keep the
/// general instantiation, which materializes the template.
#[derive(Default)]
pub(crate) struct Plans {
    compiled: HashMap<TypeId, Option<(u32, u32)>>,
    steps: Vec<Step>,
    slots: Vec<Slot>,
}
impl Plans {
    fn plan(&mut self, store: &Store, template: TypeId) -> Option<(u32, u32)> {
        if let Some(&plan) = self.compiled.get(&template) {
            return plan;
        }
        let start = self.steps.len();
        let slots = self.slots.len();
        let plan = self.compile(store, template).and_then(|()| {
            Some((
                u32::try_from(start).ok()?,
                u32::try_from(self.steps.len()).ok()?,
            ))
        });
        if plan.is_none() {
            self.steps.truncate(start);
            self.slots.truncate(slots);
        }
        self.compiled.insert(template, plan);
        plan
    }
    /// Postorder steps; children precede their node, in edge order.
    fn compile(&mut self, store: &Store, template: TypeId) -> Option<()> {
        let mut work = vec![(template, 0_u16, false)];
        while let Some((ty, depth, finish)) = work.pop() {
            if !finish && !store.contains_parameters(ty)? {
                self.steps.push(Step::Known(ty));
                continue;
            }
            let kind = store.kind(ty)?;
            if finish {
                let step = self.node(store, kind)?;
                self.steps.push(step);
                continue;
            }
            work.push((ty, depth, true));
            match kind {
                Kind::Parameter(id) => {
                    work.pop();
                    self.steps.push(Step::Parameter(id, depth));
                }
                Kind::Reference {
                    referent: child, ..
                }
                | Kind::Pointer { pointee: child, .. }
                | Kind::Pattern { base: child, .. }
                | Kind::Slice(child)
                | Kind::Array { element: child, .. } => work.push((child, depth, false)),
                Kind::Tuple(list) => {
                    work.extend(
                        store
                            .list(list)?
                            .iter()
                            .rev()
                            .map(|&child| (child, depth, false)),
                    );
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let inner = depth.checked_add(1)?;
                    work.push((result, inner, false));
                    work.extend(
                        store
                            .list(parameters)?
                            .iter()
                            .rev()
                            .map(|&child| (child, inner, false)),
                    );
                }
                Kind::Adt(_, arguments) if store.region_args(arguments).is_none() => {
                    let mut types = Vec::new();
                    for argument in store.arguments(arguments)? {
                        match argument {
                            Canonical::Lifetime(_) => {}
                            Canonical::Type(child) => types.push(child),
                            Canonical::Const(_) => return None,
                        }
                    }
                    work.extend(types.into_iter().rev().map(|child| (child, depth, false)));
                }
                _ => return None,
            }
        }
        Some(())
    }
    /// The construction step of a node whose children are already emitted.
    fn node(&mut self, store: &Store, kind: Kind) -> Option<Step> {
        Some(match kind {
            Kind::Reference {
                mutable, region, ..
            } => Step::Construct(Constructor::Reference { mutable, region }, 1),
            Kind::Pointer { mutable, .. } => Step::Construct(Constructor::Pointer { mutable }, 1),
            Kind::Pattern { constraint, .. } => {
                Step::Construct(Constructor::Pattern(constraint), 1)
            }
            Kind::Slice(_) => Step::Construct(Constructor::Slice, 1),
            Kind::Array { length, .. } => Step::Construct(Constructor::Array(length), 1),
            Kind::Tuple(list) => Step::Construct(
                Constructor::Tuple,
                u32::try_from(store.list(list)?.len()).ok()?,
            ),
            Kind::Function {
                bound_regions,
                parameters,
                unsafe_,
                convention,
                ..
            } => {
                let parameters = u32::try_from(store.list(parameters)?.len()).ok()?;
                Step::Construct(
                    Constructor::Function {
                        bound_regions,
                        parameters,
                        unsafe_,
                        convention,
                    },
                    parameters.checked_add(1)?,
                )
            }
            Kind::Adt(item, arguments) => {
                let start = u32::try_from(self.slots.len()).ok()?;
                for argument in store.arguments(arguments)? {
                    self.slots.push(match argument {
                        Canonical::Lifetime(region) => Slot::Lifetime(region),
                        Canonical::Type(_) => Slot::Type,
                        Canonical::Const(_) => return None,
                    });
                }
                Step::Nominal(item, start, u32::try_from(self.slots.len()).ok()? - start)
            }
            _ => return None,
        })
    }
}

impl Context {
    /// Instantiate a callable signature through its compiled template, which
    /// builds the same terms as [`Self::instantiate_with`] for a fresh
    /// environment. Templates that do not compile take the general path.
    /// # Errors
    /// Rejects invalid identities and exhausted inference resources.
    pub fn instantiate_signature(
        &mut self,
        store: &mut Store,
        template: TypeId,
        substitution: &mut Instantiation,
    ) -> Result<TermId, Error> {
        if !substitution.fresh() {
            return self.instantiate_with(store, template, substitution);
        }
        let mut plans = store.take_plans();
        let result = match plans.plan(store, template) {
            Some((start, end)) => {
                let mut values = std::mem::take(&mut substitution.children);
                values.clear();
                let result = self.replay(store, &plans, start..end, substitution, &mut values);
                values.clear();
                substitution.children = values;
                Some(result)
            }
            None => None,
        };
        store.restore_plans(plans);
        match result {
            Some(result) => result,
            None => self.instantiate_with(store, template, substitution),
        }
    }
    fn replay(
        &mut self,
        store: &mut Store,
        plans: &Plans,
        steps: std::ops::Range<u32>,
        substitution: &mut Instantiation,
        values: &mut Vec<TermId>,
    ) -> Result<TermId, Error> {
        for index in steps {
            self.step()?;
            let step = plans.steps[index as usize];
            let term = match step {
                Step::Known(ty) => self.known(store, ty)?,
                Step::Parameter(id, depth) => {
                    if let Some(Argument::Type(value)) = substitution.binding(id) {
                        substitution.shifted_binding(self, store, value, depth)?
                    } else {
                        let term = self.construct(Constructor::Parameter(id), &[])?;
                        self.remember_structure(store, term)?;
                        term
                    }
                }
                Step::Construct(kind, arity) => {
                    let first = values
                        .len()
                        .checked_sub(arity as usize)
                        .ok_or(Error::InvalidIdentity)?;
                    self.account_work(arity as usize)?;
                    let term = self.construct(kind, &values[first..])?;
                    values.truncate(first);
                    self.remember_structure(store, term)?;
                    term
                }
                Step::Nominal(item, slots, count) => {
                    let mut arguments = Vec::with_capacity(count as usize);
                    let types = (slots..slots + count)
                        .filter(|&slot| matches!(plans.slots[slot as usize], Slot::Type))
                        .count();
                    let first = values
                        .len()
                        .checked_sub(types)
                        .ok_or(Error::InvalidIdentity)?;
                    let mut children = values[first..].iter();
                    for slot in slots..slots + count {
                        arguments.push(match plans.slots[slot as usize] {
                            Slot::Lifetime(region) => Argument::Lifetime(region),
                            Slot::Type => {
                                Argument::Type(*children.next().ok_or(Error::InvalidIdentity)?)
                            }
                        });
                    }
                    values.truncate(first);
                    self.account_work(arguments.len())?;
                    let term = self.nominal(item, &arguments)?;
                    self.remember_structure(store, term)?;
                    term
                }
            };
            values.push(term);
        }
        let result = values.pop().ok_or(Error::InvalidIdentity)?;
        if values.is_empty() {
            Ok(result)
        } else {
            Err(Error::InvalidIdentity)
        }
    }
}
