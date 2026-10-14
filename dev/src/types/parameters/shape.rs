//! Structural well-formedness and sizedness under one immutable bound environment.
use super::environment::Environment;
use crate::types::{Kind, Primitive, Store, TypeId};
use std::{collections::HashMap, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Properties {
    pub sized: bool,
    pub well_formed: bool,
}
pub struct Shapes {
    environment: Arc<Environment>,
    cache: HashMap<TypeId, Properties>,
    work: Vec<(TypeId, bool)>,
    steps: usize,
}
impl Shapes {
    #[must_use]
    pub fn new(environment: Arc<Environment>) -> Self {
        Self {
            environment,
            cache: HashMap::new(),
            work: Vec::new(),
            steps: 0,
        }
    }
    /// # Errors
    /// Rejects invalid/foreign parameter types and exhausted graph work.
    pub fn demand(&mut self, types: &Store, ty: TypeId) -> Result<Properties, &'static str> {
        if let Some(value) = self.cache.get(&ty) {
            return Ok(*value);
        }
        self.work.clear();
        self.work.push((ty, false));
        while let Some((ty, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err("generic shape work budget exceeded");
            }
            if self.cache.contains_key(&ty) {
                continue;
            }
            let kind = types.kind(ty).ok_or("invalid generic shape type")?;
            if types.contains_parameters(ty) == Some(false) {
                self.cache.insert(
                    ty,
                    Properties {
                        sized: types.is_sized(ty) == Some(true)
                            || self.environment.assumes_sized(ty),
                        well_formed: types.is_well_formed(ty) == Some(true),
                    },
                );
                continue;
            }
            if !finish {
                self.work.push((ty, true));
                children(types, kind, |child| {
                    if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                        return Err("generic shape work budget exceeded");
                    }
                    self.work.push((child, false));
                    Ok(())
                })?;
                continue;
            }
            let mut value = Properties {
                sized: true,
                well_formed: true,
            };
            let mut index = 0;
            let tuple_len = if let Kind::Tuple(list) = kind {
                types.list(list).ok_or("invalid shape tuple")?.len()
            } else {
                0
            };
            children(types, kind, |child| {
                let child = self.cache.get(&child).ok_or("missing child shape")?;
                value.well_formed &= child.well_formed;
                match kind {
                    Kind::Tuple(_) => {
                        value.sized &= child.sized;
                        value.well_formed &= child.sized || index + 1 == tuple_len;
                    }
                    Kind::Array { .. } | Kind::ArrayTerm { .. } | Kind::Slice(_) => {
                        value.sized &= child.sized;
                        value.well_formed &= child.sized;
                    }
                    _ => {}
                }
                index += 1;
                Ok(())
            })?;
            match kind {
                Kind::Parameter(id) => {
                    let abilities = self
                        .environment
                        .abilities(id)
                        .ok_or("shape parameter is outside its environment")?;
                    value.sized = types.is_sized(ty) == Some(true) || abilities.clone_;
                }
                Kind::Slice(_) | Kind::Primitive(Primitive::Str) => value.sized = false,
                _ => {}
            }
            value.sized |= self.environment.assumes_sized(ty);
            self.cache.insert(ty, value);
        }
        self.cache
            .get(&ty)
            .copied()
            .ok_or("missing generic shape result")
    }
}
fn children(
    types: &Store,
    kind: Kind,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match kind {
        Kind::Array { element, .. }
        | Kind::ArrayTerm { element, .. }
        | Kind::Slice(element)
        | Kind::Reference {
            referent: element, ..
        }
        | Kind::Pointer {
            pointee: element, ..
        } => visit(element)?,
        Kind::Tuple(list) => {
            for ty in types.list(list).ok_or("invalid shape tuple")? {
                visit(*ty)?;
            }
        }
        Kind::Function {
            parameters, result, ..
        } => {
            for ty in types.list(parameters).ok_or("invalid shape parameters")? {
                visit(*ty)?;
            }
            visit(result)?;
        }
        Kind::FunctionItem {
            signature,
            arguments,
            ..
        } => {
            visit(signature)?;
            for a in types
                .arguments(arguments)
                .ok_or("invalid shape arguments")?
            {
                if let crate::types::arguments::Argument::Type(ty) = a {
                    visit(ty)?;
                }
            }
        }
        Kind::Adt(_, args) => {
            // Field declarations prove the nominal's storage shape once. Uses
            // separately prove its substituted bounds; structural WF includes
            // every written type argument even when it is stored indirectly.
            for argument in types
                .arguments(args)
                .ok_or("invalid nominal shape arguments")?
            {
                if let crate::types::arguments::Argument::Type(ty) = argument {
                    visit(ty)?;
                }
            }
        }
        _ => {}
    }
    Ok(())
}
