//! Prove representation constraints without inventing concrete layouts for rigid parameters.
use super::layout::{Error, Layouts, validate_repr};
use crate::types::{
    Kind, Primitive, Store, TypeId,
    aggregate::{self, Definitions, Representation},
};
use std::collections::HashMap;

#[derive(Clone, Copy)]
struct Properties {
    known: bool,
    /// repr(C) anywhere by value prevents a 1-ZST being a transparent side field.
    transparent: bool,
}
#[derive(Clone, Copy)]
enum State {
    Active,
    Complete(Properties),
    Failed(&'static str),
}
/// Types and completed declaration templates stay immutable except for appended
/// canonical instances. Storage recursion and the declaration packing graph
/// must be checked before this query; neither is a property of one instance.
#[derive(Default)]
pub struct Templates {
    states: HashMap<TypeId, State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl Templates {
    /// Validate one declaration without expanding all of its symbolic instances.
    /// Ordinary generic fields are proved at their own declarations. Only a
    /// transparent side-field question needs a symbolic layout dependency walk.
    /// # Errors
    /// Rejects invalid representations and errors in known concrete field layouts.
    pub fn declaration(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<(), Error> {
        self.declaration_inner(types, definitions, layouts, root)
            .map_err(|message| Error { ty: root, message })
    }
    fn declaration_inner(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<(), &'static str> {
        let Some(Kind::Adt(item, _)) = types.kind(root) else {
            return Err("representation declaration requires a nominal type");
        };
        let definition = definitions
            .get(item)
            .ok_or("missing representation declaration")?;
        validate_repr(definition, definitions.variants(definition).len())?;
        let transparent = definition.repr.representation == Representation::Transparent;
        let lists: Vec<_> = definitions
            .variants(definition)
            .iter()
            .map(|v| v.fields)
            .collect();
        let mut nontrivial = false;
        for list in lists {
            let count = types
                .list(list)
                .ok_or("missing representation declaration fields")?
                .len();
            for index in 0..count {
                let field = types
                    .list(list)
                    .ok_or("missing representation declaration fields")?[index];
                if transparent && count > 1 {
                    self.demand(types, definitions, layouts, field)
                        .map_err(|e| e.message)?;
                    if !self.trivial(layouts, field)? && std::mem::replace(&mut nontrivial, true) {
                        return Err("transparent aggregate has more than one nontrivial field");
                    }
                } else if types.contains_parameters(field) == Some(false) {
                    definitions.prepare(types, field)?;
                    layouts
                        .demand(types, definitions, field)
                        .map_err(|e| e.message)?;
                }
            }
        }
        if types.contains_parameters(root) == Some(false) {
            definitions.prepare(types, root)?;
            layouts
                .demand(types, definitions, root)
                .map_err(|e| e.message)?;
        }
        Ok(())
    }
    /// Validate known sublayouts and every representation encountered by value.
    /// Unknown parameter size/metadata leaves the template without a layout.
    /// # Errors
    /// Rejects invalid representations, concrete overflow, cycles and exhausted work.
    pub fn demand(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<bool, Error> {
        match self.states.get(&root) {
            Some(State::Complete(properties)) => return Ok(properties.known),
            Some(State::Failed(message)) => return Err(Error { ty: root, message }),
            _ => {}
        }
        self.work.clear();
        self.work.push((root, false));
        while let Some((ty, finish)) = self.work.pop() {
            self.steps += 1;
            if self.steps.saturating_add(self.work.len()) > 1_000_000 {
                return Err(self.fail(root, ty, "template representation work budget exceeded"));
            }
            if matches!(self.states.get(&ty), Some(State::Complete(_))) {
                continue;
            }
            if let Some(State::Failed(message)) = self.states.get(&ty) {
                return Err(self.fail(root, ty, message));
            }
            let result = if finish {
                self.finish(types, definitions, layouts, ty)
            } else {
                self.begin(types, definitions, ty)
            };
            if let Err(message) = result {
                return Err(self.fail(root, ty, message));
            }
        }
        self.properties(root)
            .map(|p| p.known)
            .map_err(|message| Error { ty: root, message })
    }
    fn fail(&mut self, root: TypeId, ty: TypeId, message: &'static str) -> Error {
        for state in self.states.values_mut() {
            if matches!(state, State::Active) {
                *state = State::Failed(message);
            }
        }
        self.states.insert(root, State::Failed(message));
        self.work.clear();
        Error { ty, message }
    }
    fn properties(&self, ty: TypeId) -> Result<Properties, &'static str> {
        let Some(State::Complete(properties)) = self.states.get(&ty) else {
            return Err("missing template representation dependency");
        };
        Ok(*properties)
    }
    fn begin(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        if matches!(self.states.insert(ty, State::Active), Some(State::Active)) {
            return Err("recursive type has infinite size");
        }
        if matches!(types.kind(ty), Some(Kind::Adt(..))) {
            definitions.instantiate(types, ty)?;
        }
        self.work.push((ty, true));
        children(types, definitions, ty, |child| {
            if self.steps.saturating_add(self.work.len()) >= 1_000_000 {
                return Err("template representation pending work budget exceeded");
            }
            self.work.push((child, false));
            Ok(())
        })
    }
    fn finish(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
    ) -> Result<(), &'static str> {
        let kind = types
            .kind(ty)
            .ok_or("missing template representation type")?;
        let mut properties = Properties {
            known: true,
            transparent: true,
        };
        children(types, definitions, ty, |child| {
            let child = self.properties(child)?;
            properties.known &= child.known;
            properties.transparent &= child.transparent;
            Ok(())
        })?;
        match kind {
            Kind::Parameter(_) | Kind::ArrayTerm { .. } => properties.known = false,
            Kind::Pointer { pointee, .. }
            | Kind::Reference {
                referent: pointee, ..
            } if types.contains_parameters(pointee) == Some(true)
                && types.is_sized(pointee) != Some(true)
                && !matches!(types.kind(pointee), Some(Kind::Slice(_))) =>
            {
                properties.known = false;
            }
            Kind::Adt(..) => {
                let shape = definitions.shape(types, ty)?;
                validate_repr(shape.definition, shape.variants().len())?;
                properties.transparent &= shape.definition.repr.representation != Representation::C;
                if shape.definition.repr.representation == Representation::Transparent {
                    self.transparent(types, layouts, &shape)?;
                }
            }
            _ => {}
        }
        if properties.known {
            layouts
                .demand(types, definitions, ty)
                .map_err(|e| e.message)?;
        }
        self.states.insert(ty, State::Complete(properties));
        Ok(())
    }
    fn transparent(
        &self,
        types: &Store,
        layouts: &Layouts,
        shape: &aggregate::Shape<'_>,
    ) -> Result<(), &'static str> {
        let mut nontrivial = false;
        for variant in shape.variants() {
            for &field in types
                .list(variant.fields)
                .ok_or("missing transparent field templates")?
            {
                if !self.trivial(layouts, field)? && std::mem::replace(&mut nontrivial, true) {
                    return Err("transparent aggregate has more than one nontrivial field");
                }
            }
        }
        Ok(())
    }
    fn trivial(&self, layouts: &Layouts, field: TypeId) -> Result<bool, &'static str> {
        let properties = self.properties(field)?;
        Ok(properties.known
            && properties.transparent
            && layouts
                .get(field)
                .is_some_and(|layout| layout.size == 0 && layout.align == 1))
    }
}
fn children(
    types: &Store,
    definitions: &Definitions,
    ty: TypeId,
    mut visit: impl FnMut(TypeId) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    match types
        .kind(ty)
        .ok_or("invalid template representation type")?
    {
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } => visit(element)?,
        Kind::Tuple(list) => {
            for &child in types
                .list(list)
                .ok_or("invalid template representation tuple")?
                .iter()
                .rev()
            {
                visit(child)?;
            }
        }
        Kind::Adt(..) => {
            for variant in definitions.shape(types, ty)?.variants().rev() {
                for &child in types
                    .list(variant.fields)
                    .ok_or("invalid template representation fields")?
                    .iter()
                    .rev()
                {
                    visit(child)?;
                }
            }
        }
        Kind::Slice(_) | Kind::Primitive(Primitive::Str) => {
            return Err("unsized field has no storage layout");
        }
        _ => {}
    }
    Ok(())
}
