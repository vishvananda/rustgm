//! Collect occurrences of one enclosing binder from already elaborated types.
use super::{GenericArgs, Kind, RegionId, Store, TypeId, arguments::Argument};
use std::collections::HashSet;

/// Validate the completed signature, including references introduced by local
/// type inference. Captured outer lifetimes are distinct from this binder.
///
/// # Errors
/// Rejects malformed binders and output-only late lifetimes.
pub fn check_pointer(store: &Store, signature: TypeId) -> Result<(), &'static str> {
    let Some(Kind::Function {
        parameters,
        result,
        bound_regions,
        ..
    }) = store.kind(signature)
    else {
        return Err("missing function pointer signature");
    };
    let mut inputs = Uses::default();
    for ty in store
        .list(parameters)
        .ok_or("missing function pointer inputs")?
    {
        inputs.collect(store, *ty)?;
    }
    let mut output = Uses::default();
    output.collect(store, result)?;
    if !output.indices.is_subset(&inputs.indices) {
        return Err("function pointer output lifetime is unconstrained by its inputs");
    }
    if inputs
        .indices
        .iter()
        .any(|index| u32::from(*index) >= bound_regions)
    {
        return Err("function pointer references an unbound lifetime");
    }
    Ok(())
}

#[derive(Default)]
pub struct Uses {
    pub indices: HashSet<u16>,
    seen: HashSet<(TypeId, u16)>,
    work: Vec<(TypeId, u16)>,
    pub steps: usize,
}
impl Uses {
    /// Accumulate several roots from the same binder and canonical store.
    /// Nested binders and alias-erased arguments do not constrain this binder.
    ///
    /// # Errors
    /// Rejects invalid canonical data and exhausted traversal resources.
    pub fn collect(&mut self, store: &Store, root: TypeId) -> Result<(), &'static str> {
        if store
            .escaping_binders(root)
            .ok_or("invalid bound-use type")?
            == 0
        {
            return Ok(());
        }
        self.push(root, 0)?;
        while let Some((ty, depth)) = self.work.pop() {
            self.steps += 1;
            if store
                .escaping_binders(ty)
                .ok_or("invalid bound-use dependency")?
                <= depth
                || !self.seen.insert((ty, depth))
            {
                continue;
            }
            match store.kind(ty).ok_or("invalid bound-use type")? {
                Kind::Reference {
                    region, referent, ..
                } => {
                    self.region(region, depth);
                    self.push(referent, depth)?;
                }
                Kind::Adt(_, arguments) => {
                    self.arguments(store, arguments, depth)?;
                }
                Kind::Tuple(list) => {
                    for ty in store.list(list).ok_or("invalid bound-use tuple")? {
                        self.push(*ty, depth)?;
                    }
                }
                Kind::Array { element: ty, .. }
                | Kind::Slice(ty)
                | Kind::Pointer { pointee: ty, .. } => self.push(ty, depth)?,
                Kind::FunctionItem {
                    signature,
                    arguments,
                    ..
                } => {
                    self.arguments(store, arguments, depth)?;
                    self.push(signature, depth)?;
                }
                Kind::Function {
                    parameters, result, ..
                } => {
                    let depth = depth
                        .checked_add(1)
                        .ok_or("bound-use binder depth overflow")?;
                    for ty in store
                        .list(parameters)
                        .ok_or("invalid bound-use signature")?
                    {
                        self.push(*ty, depth)?;
                    }
                    self.push(result, depth)?;
                }
                Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        store: &Store,
        args: GenericArgs,
        depth: u16,
    ) -> Result<(), &'static str> {
        let arguments = store.arguments(args).ok_or("invalid bound-use arguments")?;
        self.reserve(arguments.len())?;
        self.steps += arguments.len();
        for argument in arguments {
            match argument {
                Argument::Lifetime(region) => self.region(region, depth),
                Argument::Type(ty) => self.push(ty, depth)?,
                Argument::Const(_) => {}
            }
        }
        Ok(())
    }
    fn reserve(&self, amount: usize) -> Result<(), &'static str> {
        if self
            .steps
            .saturating_add(self.work.len())
            .saturating_add(amount)
            > 1_000_000
        {
            return Err("bound lifetime occurrence work budget exceeded");
        }
        Ok(())
    }
    fn push(&mut self, ty: TypeId, depth: u16) -> Result<(), &'static str> {
        self.reserve(1)?;
        self.work.push((ty, depth));
        Ok(())
    }
    fn region(&mut self, region: RegionId, depth: u16) {
        if let Some((binder, index)) = region.bound_parts()
            && binder == depth
        {
            self.indices.insert(index);
        }
    }
}
