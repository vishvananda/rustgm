//! Authenticated future storage data stays under its actual opaque owner.
use super::{GenericArgs, Kind, ListId, Properties, Property, Store, TypeId, opaque::OpaqueId};
use crate::support::{hash::HashMap, retained::Growth};
use std::{cell::Cell, rc::Rc};

/// Generated operations have no invented function declaration or closure identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    Poll,
    Drop,
}

/// Full semantic application; lifetime erasure is a separate ABI operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntryKey {
    pub future: TypeId,
    pub operation: Operation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Application {
    pub captures: ListId,
    pub storage: ListId,
    pub output: TypeId,
}
#[derive(Clone, Copy)]
enum Cached {
    Active,
    Ready(Application),
    Failed(&'static str),
}

#[derive(Default)]
struct Budget {
    steps: Cell<usize>,
    bytes: Cell<usize>,
    failed: Cell<Option<&'static str>>,
}
impl Budget {
    fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        if let Some(error) = self.failed.get() {
            return Err(error);
        }
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > 32_000_000 || self.bytes.get() > 64 * 1024 * 1024 {
            let error = "future application work/storage budget exceeded";
            self.failed.set(Some(error));
            return Err(error);
        }
        Ok(())
    }
}
pub(super) struct Shape {
    application: TypeId,
    signature: TypeId,
    body_signature: TypeId,
    output: TypeId,
    applications: HashMap<GenericArgs, Cached>,
    budget: Rc<Budget>,
}
impl Store {
    pub(crate) fn authored_future_template(
        &self,
        identity: OpaqueId,
    ) -> Result<Option<(TypeId, TypeId, TypeId, TypeId)>, &'static str> {
        let Some(shape) = self
            .opaques
            .as_ref()
            .and_then(|rows| rows.get(identity.0 as usize))
            .and_then(|entry| entry.future.as_deref())
        else {
            return Ok(None);
        };
        self.require_future_shape(identity, shape)?;
        Ok(Some((
            shape.application,
            shape.signature,
            shape.body_signature,
            shape.output,
        )))
    }
    /// Provenance for a real verifier; the handle supplies no body or runtime readiness.
    pub(crate) fn future_proof_arena(
        &self,
        future: TypeId,
    ) -> Result<super::arena::Identity, &'static str> {
        let Some(Kind::Future {
            identity,
            arguments,
        }) = self.kind(future)
        else {
            return Err("future proof requires its actual complete type application");
        };
        self.future_application(identity, arguments)?;
        let shape = self
            .opaques
            .as_ref()
            .and_then(|rows| rows.get(identity.0 as usize))
            .and_then(|entry| entry.future.as_deref())
            .ok_or("future proof lost its actual source shape")?;
        shape
            .budget
            .charge(1, size_of::<Option<super::arena::Identity>>())?;
        self.existing_arena_identity()
            .cloned()
            .ok_or("future proof requires its prepaid source type arena")
    }
    pub(crate) const fn future_shape_allocation() -> usize {
        size_of::<Shape>() + size_of::<Budget>() + 2 * size_of::<usize>()
    }
    /// This is typed storage data only. The actual checked body remains in Prepared.
    pub(crate) fn declare_future_shape(
        &mut self,
        identity: OpaqueId,
        application: TypeId,
        signature: TypeId,
        output: TypeId,
        body_output: TypeId,
    ) -> Result<(), &'static str> {
        let budget = Budget::default();
        let arena = if self.existing_arena_identity().is_none() {
            super::arena::Identity::allocation_bytes()
        } else {
            0
        };
        budget.charge(1, Self::future_shape_allocation().saturating_add(arena))?;
        self.arena_identity();
        let Some(Kind::Function {
            bound_regions,
            parameters,
            unsafe_,
            convention,
            ..
        }) = self.kind(signature)
        else {
            return Err("future shape lost its actual constructor signature");
        };
        let body_signature = self.intern_observed::<true>(
            Kind::Function {
                bound_regions,
                parameters,
                result: body_output,
                unsafe_,
                convention,
            },
            &|steps, bytes| budget.charge(steps, bytes),
        )?;
        let shape = Shape {
            application,
            signature,
            body_signature,
            output,
            applications: HashMap::default(),
            budget: Rc::new(budget),
        };
        self.require_future_shape(identity, &shape)?;
        let entry = self
            .opaques
            .as_mut()
            .and_then(|rows| rows.get_mut(identity.0 as usize))
            .ok_or("future shape lost its actual opaque entry")?;
        if let Some(previous) = &entry.future {
            if (
                previous.application,
                previous.signature,
                previous.body_signature,
                previous.output,
            ) != (application, signature, body_signature, output)
            {
                return Err("future shape changed its actual defining contract");
            }
            return Ok(());
        }
        entry.future = Some(Box::new(shape));
        Ok(())
    }
    fn require_future_shape(
        &self,
        identity: OpaqueId,
        shape: &Shape,
    ) -> Result<(GenericArgs, ListId), &'static str> {
        let contract = self
            .opaque_declaration(identity)
            .ok_or("future shape has no actual opaque declaration")?;
        let Some(Kind::Opaque(actual, captures)) = self.kind(shape.application) else {
            return Err("future shape lost its canonical formal application");
        };
        let Some(Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        }) = self.kind(shape.signature)
        else {
            return Err("future shape lost its actual input signature");
        };
        let Some(Kind::Function {
            result: body_output,
            ..
        }) = self.kind(shape.body_signature)
        else {
            return Err("future shape lost its actual authored signature");
        };
        if self.kind(shape.body_signature)
            != Some(Kind::Function {
                bound_regions,
                parameters,
                result: body_output,
                unsafe_,
                convention,
            })
        {
            return Err("future shape changed its authored input or qualifier contract");
        }
        if actual != identity
            || result != shape.application
            || !contract.sized
            || contract.copies != super::copy::Properties::default()
            || self.arguments(captures).is_none()
            || self.list(parameters).is_none()
            || self.kind(shape.output).is_none()
        {
            return Err("future shape changed its actual formals, output or declared schema");
        }
        shape.budget.charge(1, 0)?;
        Ok((captures, parameters))
    }
    pub(crate) fn future_application(
        &self,
        identity: OpaqueId,
        arguments: GenericArgs,
    ) -> Result<Application, &'static str> {
        let shape = self
            .opaques
            .as_ref()
            .and_then(|rows| rows.get(identity.0 as usize))
            .and_then(|entry| entry.future.as_deref())
            .ok_or("future application requires its genuine authored shape")?;
        self.require_future_shape(identity, shape)?;
        match shape.applications.get(&arguments) {
            Some(Cached::Ready(value)) => Ok(*value),
            Some(Cached::Failed(error)) => Err(error),
            Some(Cached::Active) => Err("recursive future application preparation"),
            None => Err("future application requires its prepared complete captures"),
        }
    }
    pub(super) fn prepare_future<const CHARGED: bool>(
        &mut self,
        identity: OpaqueId,
        arguments: GenericArgs,
        external: &dyn Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        let shape = self
            .opaques
            .as_ref()
            .and_then(|rows| rows.get(identity.0 as usize))
            .and_then(|entry| entry.future.as_deref())
            .ok_or("future type requires its genuine authored shape")?;
        let (formal, parameters) = self.require_future_shape(identity, shape)?;
        if shape.applications.contains_key(&arguments) {
            return self.future_application(identity, arguments).map(|_| ());
        }
        if shape.applications.len() >= 100_000 {
            return Err("future application count exceeds100000");
        }
        let (output, budget) = (shape.output, Rc::clone(&shape.budget));
        let charge = |steps, bytes| {
            budget.charge(steps, bytes)?;
            if CHARGED {
                external(steps, bytes)?;
            }
            Ok(())
        };
        let growth = Growth::map::<GenericArgs, Cached>(
            shape.applications.len(),
            shape.applications.capacity(),
            1,
        );
        charge(growth.work, growth.bytes)?;
        self.future_cache(identity)?
            .insert(arguments, Cached::Active);
        let result = self.map_future(formal, arguments, parameters, output, &charge);
        self.future_cache(identity)?.insert(
            arguments,
            match result {
                Ok(value) => Cached::Ready(value),
                Err(error) => Cached::Failed(error),
            },
        );
        result.map(|_| ())
    }
    fn future_cache(
        &mut self,
        identity: OpaqueId,
    ) -> Result<&mut HashMap<GenericArgs, Cached>, &'static str> {
        self.opaques
            .as_mut()
            .and_then(|rows| rows.get_mut(identity.0 as usize))
            .and_then(|entry| entry.future.as_deref_mut())
            .map(|shape| &mut shape.applications)
            .ok_or("future preparation lost its actual entry cache")
    }
    fn map_future(
        &mut self,
        formal: GenericArgs,
        arguments: GenericArgs,
        parameters: ListId,
        output: TypeId,
        charge: &impl Fn(usize, usize) -> Result<(), &'static str>,
    ) -> Result<Application, &'static str> {
        let inputs = self
            .list(parameters)
            .ok_or("future lost its actual parameter types")?;
        charge(
            inputs.len().saturating_mul(2),
            inputs.len().saturating_mul(8 * size_of::<TypeId>()),
        )?;
        let inputs = inputs.to_vec();
        let mut fields = Vec::with_capacity(inputs.len());
        let mut fold =
            super::substitute::Bound::captured_arguments_charged(self, formal, arguments, charge)?;
        for input in inputs {
            fields.push(fold.apply_inside_observed::<true>(self, input, 0, charge)?);
        }
        let output = fold.apply_inside_observed::<true>(self, output, 0, charge)?;
        let captures = self.intern_list_observed::<true>(&fields, charge)?;
        // A real anonymous sequence, not a canonical tuple or nominal identity.
        charge(
            fields.len().saturating_add(1),
            fields
                .len()
                .saturating_add(1)
                .saturating_mul(size_of::<TypeId>()),
        )?;
        let mut storage = Vec::with_capacity(fields.len().saturating_add(1));
        storage.push(TypeId::primitive(super::Primitive::U8));
        storage.extend(fields);
        let storage = self.intern_list_observed::<true>(&storage, charge)?;
        Ok(Application {
            captures,
            storage,
            output,
        })
    }
    pub(super) fn future_properties(
        &self,
        identity: OpaqueId,
        arguments: GenericArgs,
    ) -> Result<Properties, &'static str> {
        let application = self.future_application(identity, arguments)?;
        let mut result = Properties::default();
        for &field in self
            .list(application.captures)
            .ok_or("future captures disappeared")?
        {
            result.include(
                *self
                    .properties
                    .get(field.0 as usize)
                    .ok_or("future capture type is unknown")?,
            );
        }
        self.argument_properties(arguments, &mut result)?;
        result.set(Property::Sized, true);
        Ok(result)
    }
}
