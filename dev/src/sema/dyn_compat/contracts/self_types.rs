//! Reuse canonical type tails while distinguishing Self from its associated results.
use super::{Error, charge};
use crate::types::{GenericArgs, Kind, Store, TypeId, arguments::Argument};
use std::collections::HashMap;

pub(super) struct Occurrences {
    self_type: TypeId,
    projections: bool,
    completed: HashMap<TypeId, bool>,
}
impl Occurrences {
    pub fn new(self_type: TypeId, projections: bool) -> Self {
        Self {
            self_type,
            projections,
            completed: HashMap::new(),
        }
    }
    pub fn arguments(
        &mut self,
        types: &Store,
        arguments: GenericArgs,
        steps: &mut usize,
    ) -> Result<bool, Error> {
        let arguments = types
            .arguments(arguments)
            .ok_or("invalid dyn Self arguments")?;
        charge(steps, arguments.len())?;
        for argument in arguments {
            if let Argument::Type(ty) = argument
                && self.forbidden(types, ty, steps)?
            {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub fn forbidden(
        &mut self,
        types: &Store,
        root: TypeId,
        steps: &mut usize,
    ) -> Result<bool, Error> {
        let mut work = vec![(root, false)];
        let mut children = Vec::new();
        while let Some((ty, finish)) = work.pop() {
            charge(steps, 1)?;
            if self.completed.contains_key(&ty) {
                continue;
            }
            if ty == self.self_type {
                self.completed.insert(ty, true);
                continue;
            }
            if !types
                .contains_parameters(ty)
                .ok_or("invalid dyn Self type")?
            {
                self.completed.insert(ty, false);
                continue;
            }
            children.clear();
            self.children(types, ty, &mut children)?;
            charge(steps, children.len())?;
            if finish {
                let mut found = false;
                for child in &children {
                    found |= self
                        .completed
                        .get(child)
                        .copied()
                        .ok_or("unfinished dyn Self dependency")?;
                }
                self.completed.insert(ty, found);
            } else {
                work.push((ty, true));
                work.extend(children.iter().rev().map(|&ty| (ty, false)));
            }
        }
        self.completed
            .get(&root)
            .copied()
            .ok_or(Error::Invalid("missing dyn Self result"))
    }
    fn children(&self, types: &Store, ty: TypeId, children: &mut Vec<TypeId>) -> Result<(), Error> {
        match types.kind(ty).ok_or("invalid dyn signature type")? {
            Kind::Closure {
                arguments,
                signature,
                environment,
                ..
            } => {
                children.extend([signature, environment]);
                argument_types(types, arguments, children)?;
            }
            Kind::Reference { referent, .. } => children.push(referent),
            Kind::Pointer { pointee, .. } => children.push(pointee),
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => children.push(element),
            Kind::Tuple(list) => {
                children.extend_from_slice(types.list(list).ok_or("invalid dyn tuple")?);
            }
            Kind::Function {
                parameters, result, ..
            } => {
                children.extend_from_slice(
                    types
                        .list(parameters)
                        .ok_or("invalid dyn callable inputs")?,
                );
                children.push(result);
            }
            Kind::FunctionItem {
                arguments,
                signature,
                ..
            } => {
                children.push(signature);
                argument_types(types, arguments, children)?;
            }
            Kind::Adt(_, args)
            | Kind::Opaque(_, args)
            | Kind::Dynamic {
                arguments: args, ..
            } => {
                argument_types(types, args, children)?;
            }
            Kind::Projection(application) => {
                if !self.projections || application.subject != self.self_type {
                    children.push(application.subject);
                }
                let parent = types
                    .trait_application(application.trait_application)
                    .ok_or("invalid dyn associated parent")?;
                argument_types(types, parent.arguments, children)?;
                argument_types(types, application.arguments, children)?;
            }
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
        }
        Ok(())
    }
}
fn argument_types(
    types: &Store,
    args: GenericArgs,
    children: &mut Vec<TypeId>,
) -> Result<(), Error> {
    children.extend(
        types
            .arguments(args)
            .ok_or("invalid dyn signature arguments")?
            .filter_map(|arg| {
                if let Argument::Type(ty) = arg {
                    Some(ty)
                } else {
                    None
                }
            }),
    );
    Ok(())
}
