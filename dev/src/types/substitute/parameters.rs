//! Joint parameter replacement shares the lifetime folder and its capture rules.
use super::{Action, Bound};
use crate::support::hash::HashMap;
use crate::types::{
    GenericArgs, ItemId, Store, TypeId,
    arguments::{Argument, ConstId, ConstKind},
    parameters::{ParameterId, ParameterKind},
};

enum Arguments {
    All {
        owner: ItemId,
        values: Vec<Argument>,
    },
    Mapped(HashMap<ParameterId, Argument>),
    Single(ParameterId, Argument),
}
pub(super) struct Bindings {
    arguments: Arguments,
}
impl Bound {
    /// Replace explicit parameter identities simultaneously, preserving lifetime binders.
    /// Callable argument order need not match declaration slots containing late lifetimes.
    /// # Errors
    /// Rejects duplicate/invalid parameters, argument kind mismatches and excessive work.
    pub fn parameters(
        store: &Store,
        arguments: &[(ParameterId, Argument)],
    ) -> Result<Self, &'static str> {
        let mut bound = Self::parameters_captured(store, arguments, Vec::new())?;
        bound.action = Action::Shift(0);
        Ok(bound)
    }
    /// Replace explicit parameters while removing a separately supplied captured binder.
    /// # Errors
    /// Reports the same invalid mappings as `parameters` and oversized lifetime mappings.
    pub fn parameters_captured(
        store: &Store,
        arguments: &[(ParameterId, Argument)],
        regions: Vec<crate::types::RegionId>,
    ) -> Result<Self, &'static str> {
        let mut bound = Self::new(regions);
        bound.account_arguments(arguments.len().saturating_add(bound.arguments.len()))?;
        let mut mappings = HashMap::with_capacity_and_hasher(
            arguments.len(),
            crate::support::hash::BuildFxHasher::default(),
        );
        for &(id, argument) in arguments {
            let parameter = store
                .parameter(id)
                .ok_or("invalid substitution parameter")?;
            let valid = match (parameter.kind, argument) {
                (ParameterKind::Type { .. }, Argument::Type(ty)) => store.kind(ty).is_some(),
                (ParameterKind::Const { ty }, Argument::Const(id)) => {
                    store.const_type(id) == Some(ty)
                }
                _ => false,
            };
            if !valid || mappings.insert(id, argument).is_some() {
                return Err("invalid or duplicate parameter substitution");
            }
        }
        bound.parameters = Some(Box::new(Bindings {
            arguments: Arguments::Mapped(mappings),
        }));
        Ok(bound)
    }
    /// Start an append-only argument environment for resolving declaration defaults.
    /// Successful substitutions depend only on existing slots, so their cached
    /// results remain valid when later slots are appended. Inserted arguments are
    /// simultaneous replacements and are never substituted a second time.
    #[must_use]
    pub fn prefix(owner: ItemId) -> Self {
        let mut substitution = Self::new(Vec::new());
        substitution.parameters = Some(Box::new(Bindings {
            arguments: Arguments::All {
                owner,
                values: Vec::new(),
            },
        }));
        substitution
    }
    /// Extend the fixed prefix without changing any previous argument or cache key.
    /// The caller validates declaration arity, defaults and bounds.
    /// # Errors
    /// Rejects missing parameter environments, invalid identities, nonleading
    /// lifetime arguments and exhausted work. Existing slots are never replaced.
    pub fn push_argument(&mut self, store: &Store, argument: Argument) -> Result<(), &'static str> {
        let bindings = self
            .parameters
            .as_ref()
            .ok_or("missing parameter substitution")?;
        let Arguments::All {
            values: arguments, ..
        } = &bindings.arguments
        else {
            return Err("single-parameter substitution cannot be extended");
        };
        let valid = match argument {
            Argument::Lifetime(_) => arguments.len() == self.arguments.len(),
            Argument::Type(ty) => store.kind(ty).is_some(),
            Argument::Const(id) => store.const_kind(id).is_some(),
        };
        if !valid {
            return Err("invalid appended generic argument");
        }
        self.account_arguments(1)?;
        let bindings = self
            .parameters
            .as_mut()
            .ok_or("missing parameter substitution")?;
        let Arguments::All {
            values: arguments, ..
        } = &mut bindings.arguments
        else {
            return Err("single-parameter substitution cannot be extended");
        };
        arguments.push(argument);
        if let Argument::Lifetime(region) = argument {
            self.arguments.push(region);
        }
        Ok(())
    }

    /// Substitute a single argument in this template's outermost binder.
    /// # Errors
    /// Rejects invalid identities, mismatches, missing arguments and work limits.
    pub fn apply_argument(
        &mut self,
        store: &mut Store,
        argument: Argument,
    ) -> Result<Argument, &'static str> {
        self.apply_argument_inside(store, argument, 0)
    }

    /// Map a complete argument list in this folder's shared lifetime environment.
    /// # Errors
    /// Rejects invalid arguments and exhausted substitution work.
    pub fn apply_arguments(
        &mut self,
        store: &mut Store,
        args: GenericArgs,
    ) -> Result<GenericArgs, &'static str> {
        self.apply_arguments_observed::<false>(store, args, 0, &|_, _| Ok(()))
    }
    pub(super) fn apply_arguments_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        args: GenericArgs,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<GenericArgs, &'static str> {
        let source = store
            .arguments(args)
            .ok_or("invalid substituted arguments")?;
        let length = source.len();
        if CHARGED {
            charge(length, length.saturating_mul(size_of::<Argument>()))?;
        }
        self.account_arguments(length)?;
        let scratch = self.reserve_region_scratch(super::bounded::growth::<Argument>(length))?;
        let mut arguments: Vec<_> = source.collect();
        let result = (|| {
            for argument in &mut arguments {
                *argument =
                    self.apply_argument_observed::<CHARGED>(store, *argument, depth, charge)?;
            }
            store.intern_arguments(&arguments)
        })();
        self.release_region_scratch(scratch)?;
        result
    }
    pub(crate) fn apply_argument_inside(
        &mut self,
        store: &mut Store,
        argument: Argument,
        depth: u16,
    ) -> Result<Argument, &'static str> {
        self.region_work(1)?;
        Ok(match argument {
            Argument::Lifetime(region) => Argument::Lifetime(self.region(region, depth)?),
            Argument::Type(ty) => Argument::Type(self.apply_inside(store, ty, depth)?),
            Argument::Const(id) => Argument::Const(self.constant(store, id)?),
        })
    }
    /// Replace this owner's type/const parameters and remove its lifetime binder
    /// in one traversal. Arguments live in the destination context. The declaration
    /// checker owns full arity/default/bound validation, including unused parameters.
    ///
    /// # Errors
    /// Rejects invalid argument identities. Applying the substitution also rejects
    /// missing used arguments, kind/type mismatches, binder overflow and work limits.
    pub fn joint(
        store: &Store,
        owner: ItemId,
        arguments: GenericArgs,
    ) -> Result<Self, &'static str> {
        let arguments = store
            .arguments(arguments)
            .ok_or("invalid substitution arguments")?;
        if arguments.len() > super::WORK_LIMIT {
            return Err("generic substitution argument budget exceeded");
        }
        let arguments: Vec<_> = arguments.collect();
        let regions = arguments
            .iter()
            .filter_map(|argument| match argument {
                Argument::Lifetime(region) => Some(*region),
                _ => None,
            })
            .collect();
        let mut substitution = Self::new(regions);
        substitution.parameters = Some(Box::new(Bindings {
            arguments: Arguments::All {
                owner,
                values: arguments,
            },
        }));
        Ok(substitution)
    }

    /// Replace named parameters while mapping a separate captured lifetime binder.
    /// Anonymous impl lifetimes have binder slots but no named argument slot.
    /// # Errors
    /// Rejects malformed argument lists and excessive captured binders.
    pub fn joint_captured(
        store: &Store,
        owner: ItemId,
        arguments: GenericArgs,
        regions: Vec<crate::types::RegionId>,
    ) -> Result<Self, &'static str> {
        let mut substitution = Self::joint(store, owner, arguments)?;
        substitution.account_arguments(regions.len())?;
        substitution.arguments = regions;
        Ok(substitution)
    }

    /// Replace exactly one rigid parameter while preserving every lifetime binder
    /// and other parameter. Closed default checks must not compose defaults from
    /// different slots, which would strengthen the declaration's requirements.
    /// # Errors
    /// Rejects unknown identities and mismatched parameter/argument kinds.
    pub fn parameter_only(
        store: &Store,
        id: ParameterId,
        argument: Argument,
    ) -> Result<Self, &'static str> {
        let parameter = store
            .parameter(id)
            .ok_or("invalid substitution parameter")?;
        let valid = match (parameter.kind, argument) {
            (ParameterKind::Type { .. }, Argument::Type(ty)) => store.kind(ty).is_some(),
            (ParameterKind::Const { ty }, Argument::Const(id)) => store.const_type(id) == Some(ty),
            _ => false,
        };
        if !valid {
            return Err("generic argument kind or const type mismatch");
        }
        let mut substitution = Self::new(Vec::new());
        substitution.action = Action::Shift(0);
        substitution.parameters = Some(Box::new(Bindings {
            arguments: Arguments::Single(id, argument),
        }));
        Ok(substitution)
    }

    pub(super) fn parameter_observed<const CHARGED: bool>(
        &mut self,
        store: &mut Store,
        id: ParameterId,
        depth: u16,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Option<TypeId>, &'static str> {
        let Some(Argument::Type(ty)) = self.argument(store, id)? else {
            return Ok(None);
        };
        if depth == 0 || store.escaping_binders(ty) == Some(0) {
            return Ok(Some(ty));
        }
        if !CHARGED {
            let (result, work) = store.shift_type(ty, depth)?;
            self.account_arguments(work)?;
            return Ok(Some(result));
        }
        let before = store.capture_shift_steps();
        let result = store.shift_type_charged(ty, depth, charge);
        self.account_arguments(store.capture_shift_steps().saturating_sub(before))?;
        result.map(|(ty, _)| Some(ty))
    }
    pub(super) fn constant(&self, store: &Store, id: ConstId) -> Result<ConstId, &'static str> {
        let kind = store
            .const_kind(id)
            .ok_or("invalid substitution constant")?;
        if let ConstKind::Parameter(parameter) = kind
            && let Some(Argument::Const(argument)) = self.argument(store, parameter)?
        {
            return Ok(argument);
        }
        Ok(id)
    }
    pub(super) fn argument(
        &self,
        store: &Store,
        id: ParameterId,
    ) -> Result<Option<Argument>, &'static str> {
        let Some(bindings) = &self.parameters else {
            return Ok(None);
        };
        let (owner, arguments) = match &bindings.arguments {
            Arguments::Single(parameter, argument) => {
                return Ok((*parameter == id).then_some(*argument));
            }
            Arguments::Mapped(arguments) => return Ok(arguments.get(&id).copied()),
            Arguments::All { owner, values } => (*owner, values),
        };
        let parameter = store
            .parameter(id)
            .ok_or("invalid substitution parameter")?;
        if parameter.owner != owner {
            return Ok(None);
        }
        let argument = *arguments
            .get(parameter.index as usize)
            .ok_or("missing generic argument")?;
        let valid = match (parameter.kind, argument) {
            (ParameterKind::Type { .. }, Argument::Type(_)) => true,
            (ParameterKind::Const { ty }, Argument::Const(id)) => store.const_type(id) == Some(ty),
            _ => false,
        };
        if !valid {
            return Err("generic argument kind or const type mismatch");
        }
        Ok(Some(argument))
    }
}

impl Bindings {
    pub(super) fn allocation(&self) -> crate::support::retained::Allocation {
        let mut total = crate::support::retained::Allocation::default();
        match &self.arguments {
            Arguments::All { values, .. } => total.vector(values),
            Arguments::Mapped(values) => total.map(values),
            Arguments::Single(..) => {}
        }
        total
    }
}
