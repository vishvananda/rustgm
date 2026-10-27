//! Callable and object bounds own independent fold caches and universal binder identities.
use super::{Action, Bound, HashMap, HashSet, Key, Kind, Normalizer, ScopeId, Store};

pub(super) struct Frame {
    pub fold: Bound,
    pub active: HashSet<Key>,
    pub redirects: HashMap<Key, Key>,
    pub scope: ScopeId,
}
impl Frame {
    pub fn new(scope: ScopeId, steps: usize) -> Self {
        let mut fold = Bound::new(Vec::new());
        fold.action = Action::Normalize;
        fold.steps = steps;
        Self {
            fold,
            active: HashSet::default(),
            redirects: HashMap::default(),
            scope,
        }
    }
}
pub(super) struct Suspended {
    frame: Frame,
    source: Key,
    signature: Kind,
}
impl Normalizer {
    /// Parent identity and local lifetime count in this normalizer's binder tree.
    /// The root has no local callable binder. Identities belong to this instance.
    #[must_use]
    pub fn binder(&self, scope: ScopeId) -> Option<(ScopeId, u32)> {
        scope
            .0
            .checked_sub(1)
            .and_then(|id| self.binders.get(id as usize))
            .copied()
    }

    pub(super) fn check_work(&self) -> Result<(), &'static str> {
        // The ordinary fold already checks its own queue; only suspended scopes
        // need the additional combined-storage gate.
        if self.binders.is_empty() {
            return Ok(());
        }
        if self
            .frame
            .fold
            .steps
            .saturating_add(self.frame.fold.work.len())
            .saturating_add(self.suspended_work)
            .saturating_add(self.binders.len())
            > super::super::WORK_LIMIT
        {
            return Err("normalization scope work budget exhausted");
        }
        Ok(())
    }

    pub(super) fn enter_binder<const CHARGED: bool>(
        &mut self,
        store: &Store,
        source: Key,
        signature: Kind,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let (Kind::Function { bound_regions, .. } | Kind::Dynamic { bound_regions, .. }) =
            signature
        else {
            return Err("normalization scope requires a type binder");
        };
        if self.suspended.len() >= 0x7fff {
            return Err("normalization binder depth exhausted");
        }
        if CHARGED {
            self.pay_binder(charge)?;
        }
        self.binders.push((self.frame.scope, bound_regions));
        let scope = ScopeId(
            u32::try_from(self.binders.len()).map_err(|_| "too many normalization scopes")?,
        );
        let frame = Frame::new(scope, self.frame.fold.steps);
        let parent = std::mem::replace(&mut self.frame, frame);
        self.suspended_work = self.suspended_work.saturating_add(parent.fold.work.len());
        self.suspended.push(Suspended {
            frame: parent,
            source,
            signature,
        });
        if CHARGED {
            self.pay_node(store, signature, false, charge)?;
        }
        match signature {
            Kind::Function {
                parameters, result, ..
            } => {
                let parameters = store
                    .list(parameters)
                    .ok_or("invalid callable parameter list")?;
                self.frame
                    .fold
                    .account_arguments(parameters.len().saturating_add(1))?;
                self.frame.fold.queue(result, 0)?;
                for &ty in parameters {
                    self.frame.fold.queue(ty, 0)?;
                }
            }
            Kind::Dynamic { arguments, .. } => {
                let arguments = store
                    .arguments(arguments)
                    .ok_or("invalid object normalization inputs")?;
                self.frame.fold.account_arguments(arguments.len())?;
                for argument in arguments {
                    if let crate::types::arguments::Argument::Type(ty) = argument {
                        self.frame.fold.queue(ty, 0)?;
                    }
                }
            }
            _ => return Err("invalid normalization binder"),
        }
        self.check_work()
    }

    pub(super) fn finish_binder<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<bool, &'static str> {
        let Some(mut parent) = self.suspended.pop() else {
            return Ok(false);
        };
        if CHARGED {
            self.pay_node(store, parent.signature, true, charge)?;
            let growth = crate::support::retained::Growth::map::<Key, crate::types::TypeId>(
                parent.frame.fold.completed.len(),
                parent.frame.fold.completed.capacity(),
                1,
            );
            charge(growth.work, growth.bytes)?;
        }
        let kind = match parent.signature {
            Kind::Function {
                bound_regions,
                parameters,
                result,
                unsafe_,
                convention,
            } => Kind::Function {
                bound_regions,
                parameters: self.frame.fold.list(store, parameters, 0)?,
                result: self.frame.fold.child(store, result, 0)?,
                unsafe_,
                convention,
            },
            Kind::Dynamic {
                shape,
                arguments,
                bound_regions,
                region,
            } => Kind::Dynamic {
                shape,
                arguments: self.frame.fold.arguments(store, arguments, 0)?,
                bound_regions,
                region,
            },
            _ => return Err("invalid suspended normalization binder"),
        };
        let ty = store.intern(kind)?;
        self.suspended_work = self
            .suspended_work
            .checked_sub(parent.frame.fold.work.len())
            .ok_or("invalid suspended normalization work count")?;
        parent.frame.fold.steps = self.frame.fold.steps;
        self.frame = parent.frame;
        self.complete(parent.source, ty);
        Ok(true)
    }
}
