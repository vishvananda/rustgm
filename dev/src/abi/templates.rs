//! Prove representation constraints without inventing concrete layouts for rigid parameters.
use super::layout::{Error, Layouts, validate_repr};
use crate::support::{
    hash::HashMap,
    retained::{Growth, Payment},
};
use crate::types::{
    Kind, Primitive, Store, TypeId,
    aggregate::{self, Definitions, Representation},
    opaque::OpaqueId,
};
#[cfg(test)]
#[path = "../../tests/invariants/trailing_objects.rs"]
mod tests;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Readiness {
    Ready,
    Symbolic,
    Pending(OpaqueId),
}
impl Readiness {
    fn combine(self, child: Self) -> Self {
        match (self, child) {
            (Self::Ready, _) | (Self::Symbolic, Self::Pending(_)) => child,
            _ => self,
        }
    }

    fn dependency(self) -> Option<OpaqueId> {
        match self {
            Self::Pending(id) => Some(id),
            _ => None,
        }
    }
}
#[derive(Clone, Copy)]
struct Properties {
    readiness: Readiness,
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
struct Deferral<'a> {
    eligible: &'a dyn Fn(OpaqueId) -> Result<bool, &'static str>,
    pay: Payment<'a>,
    missing: Vec<OpaqueId>,
}

/// Check declaration templates with a separate, explicitly configured early mode.
#[derive(Default)]
pub struct Templates<'a> {
    deferral: Option<Box<Deferral<'a>>>,
    states: HashMap<TypeId, State>,
    work: Vec<(TypeId, bool)>,
    pub steps: usize,
}
impl<'a> Templates<'a> {
    pub(crate) fn deferring(
        eligible: &'a dyn Fn(OpaqueId) -> Result<bool, &'static str>,
        pay: Payment<'a>,
    ) -> Result<Self, &'static str> {
        pay(1, size_of::<Deferral<'_>>())?;
        Ok(Self {
            deferral: Some(Box::new(Deferral {
                eligible,
                pay,
                missing: Vec::new(),
            })),
            ..Self::default()
        })
    }

    pub(crate) fn missing(self) -> Vec<OpaqueId> {
        self.deferral.map_or_else(Vec::new, |mode| mode.missing)
    }
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
        self.declaration_pending(types, definitions, layouts, root)
            .map(drop)
    }

    pub(crate) fn declaration_pending(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<Option<OpaqueId>, Error> {
        self.declaration_inner(types, definitions, layouts, root)
            .map_err(|message| Error { ty: root, message })
    }

    fn declaration_inner(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        root: TypeId,
    ) -> Result<Option<OpaqueId>, &'static str> {
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
        let mut pending = None;
        for list in lists {
            let count = types
                .list(list)
                .ok_or("missing representation declaration fields")?
                .len();
            for index in 0..count {
                let field = types
                    .list(list)
                    .ok_or("missing representation declaration fields")?[index];
                let side_field = transparent && count > 1;
                if side_field || types.contains_parameters(field) == Some(false) {
                    self.demand(types, definitions, layouts, field)
                        .map_err(|e| e.message)?;
                    if let Some(id) = self.properties(field)?.readiness.dependency() {
                        pending.get_or_insert(id);
                    } else if side_field
                        && !self.trivial(layouts, field)?
                        && std::mem::replace(&mut nontrivial, true)
                    {
                        return Err("transparent aggregate has more than one nontrivial field");
                    }
                }
            }
        }
        if types.contains_parameters(root) == Some(false) {
            self.demand(types, definitions, layouts, root)
                .map_err(|error| error.message)?;
            pending = pending.or(self.properties(root)?.readiness.dependency());
        }
        Ok(pending)
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
            Some(State::Complete(properties)) => {
                return Ok(properties.readiness == Readiness::Ready);
            }
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
            .map(|p| p.readiness == Readiness::Ready)
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
        if let Some(
            Kind::Reference {
                referent: pointee, ..
            }
            | Kind::Pointer { pointee, .. },
        ) = types.kind(ty)
            && types.is_sized(pointee) != Some(true)
            && !definitions.prepare_metadata(types, pointee)?
        {
            self.states.insert(
                ty,
                State::Complete(Properties {
                    readiness: Readiness::Symbolic,
                    transparent: true,
                }),
            );
            return Ok(());
        }
        if let Some(mode) = &mut self.deferral
            && let Some(Kind::Opaque(id, _)) = types.kind(ty)
            && types.storage_head(ty).is_err()
            && (mode.eligible)(id)?
        {
            let growth = Growth::vector::<OpaqueId>(mode.missing.len(), mode.missing.capacity(), 1);
            (mode.pay)(1 + growth.work, growth.bytes)?;
            mode.missing
                .try_reserve(1)
                .map_err(|_| "pending opaque layout allocation failed")?;
            mode.missing.push(id);
            self.states.insert(
                ty,
                State::Complete(Properties {
                    readiness: Readiness::Pending(id),
                    transparent: true,
                }),
            );
            return Ok(());
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
            readiness: Readiness::Ready,
            transparent: true,
        };
        children(types, definitions, ty, |child| {
            let child = self.properties(child)?;
            properties.readiness = properties.readiness.combine(child.readiness);
            properties.transparent &= child.transparent;
            Ok(())
        })?;
        match kind {
            // A trait object, like a slice or str, has no fixed layout. A
            // declaration ending in one has no static layout to demand.
            Kind::Foreign(_)
            | Kind::Parameter(_)
            | Kind::Projection(_)
            | Kind::ArrayTerm { .. }
            | Kind::Slice(_)
            | Kind::Dynamic { .. }
            | Kind::Primitive(Primitive::Str) => {
                properties.readiness = properties.readiness.combine(Readiness::Symbolic);
            }
            Kind::Pointer { pointee, .. }
            | Kind::Reference {
                referent: pointee, ..
            } if (types.contains_parameters(pointee) == Some(true)
                || types.contains_projections(pointee) == Some(true))
                && types.is_sized(pointee) != Some(true)
                && !matches!(types.kind(pointee), Some(Kind::Slice(_))) =>
            {
                properties.readiness = properties.readiness.combine(Readiness::Symbolic);
            }
            Kind::Adt(..) => {
                let shape = definitions.raw_shape(types, ty)?;
                validate_repr(shape.definition, shape.variants().len())?;
                properties.transparent &= shape.definition.repr.representation != Representation::C;
                if shape.definition.repr.representation == Representation::Transparent {
                    self.transparent(types, layouts, &shape)?;
                }
            }
            _ => {}
        }
        if properties.readiness == Readiness::Ready {
            layouts
                .demand(types, definitions, ty)
                .map_err(|e| e.message)?;
        }
        self.states.insert(ty, State::Complete(properties));
        Ok(())
    }
    /// Expose a real transparent field using the same side-field proof as declarations.
    pub(crate) fn storage_field(
        &mut self,
        types: &mut Store,
        definitions: &mut Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
    ) -> Result<Option<TypeId>, &'static str> {
        self.steps = self.steps.saturating_add(1);
        if self.steps > 1_000_000 {
            return Err("template representation work budget exceeded");
        }
        let Some(Kind::Adt(item, _)) = types.kind(ty) else {
            return Ok(None);
        };
        let declaration = definitions
            .get(item)
            .ok_or("transparent storage requires completed fields")?;
        if declaration.kind != aggregate::Kind::Struct
            || declaration.repr.representation != Representation::Transparent
        {
            return Ok(None);
        }
        definitions.instantiate(types, ty)?;
        let fields = {
            let shape = definitions.raw_shape(types, ty)?;
            validate_repr(shape.definition, shape.variants().len())?;
            let mut variants = shape.variants();
            let fields = variants
                .next()
                .ok_or("transparent storage requires one variant")?
                .fields;
            if variants.next().is_some() {
                return Err("transparent storage requires one variant");
            }
            fields
        };
        let count = types
            .list(fields)
            .ok_or("missing transparent storage fields")?
            .len();
        self.steps = self.steps.saturating_add(count);
        if self.steps > 1_000_000 {
            return Err("template representation work budget exceeded");
        }
        if count == 1 {
            return Ok(types
                .list(fields)
                .and_then(|fields| fields.first())
                .copied());
        }
        let mut selected = None;
        for index in 0..count {
            let field = types
                .list(fields)
                .ok_or("missing transparent storage fields")?[index];
            self.demand(types, definitions, layouts, field)
                .map_err(|error| error.message)?;
            if !self.trivial(layouts, field)? && selected.replace(field).is_some() {
                return Err("transparent aggregate has more than one nontrivial field");
            }
        }
        Ok(selected)
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
                if self.properties(field)?.readiness.dependency().is_none()
                    && !self.trivial(layouts, field)?
                    && std::mem::replace(&mut nontrivial, true)
                {
                    return Err("transparent aggregate has more than one nontrivial field");
                }
            }
        }
        Ok(())
    }
    fn trivial(&self, layouts: &Layouts, field: TypeId) -> Result<bool, &'static str> {
        let properties = self.properties(field)?;
        Ok(properties.readiness == Readiness::Ready
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
        Kind::Pattern { base, .. } => visit(base)?,
        Kind::Closure { environment, .. } => visit(environment)?,
        Kind::Future {
            identity,
            arguments,
        } => {
            let fields = types.future_application(identity, arguments)?.storage;
            for &child in types
                .list(fields)
                .ok_or("future template lost its actual capture storage")?
                .iter()
                .rev()
            {
                visit(child)?;
            }
        }
        Kind::Opaque(..) => visit(types.storage_head(ty)?)?,
        Kind::Array { element, .. } | Kind::ArrayTerm { element, .. } | Kind::Slice(element) => {
            visit(element)?;
        }
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
            for variant in definitions.raw_shape(types, ty)?.variants().rev() {
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
        _ => {}
    }
    Ok(())
}
