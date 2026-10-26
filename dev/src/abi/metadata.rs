//! Pointer metadata follows completed storage tails, independently of value layout.
use crate::support::hash::HashMap;
use crate::types::{Kind, ListId, Primitive, Store, TypeId, aggregate};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Metadata {
    Thin,
    /// Element count for slices, byte count for str; inherited unchanged by DST tails.
    Length,
    /// A trait-object tail carries the vtable for its complete object bounds.
    VTable(TypeId),
    /// A rigid or unnormalized tail needs a caller's proof or concrete substitution.
    Unknown(TypeId),
}

#[derive(Clone, Copy)]
enum State {
    Active,
    Complete(Result<Metadata, &'static str>),
}
pub(crate) enum Step {
    Tail(TypeId),
    Complete(Metadata),
}
#[derive(Clone, Copy)]
enum Fields {
    Physical,
    Declared,
}

/// Declaration data only: callers must prepare the real aggregate instance and
/// retain semantic normalization/formation proof separately. A sized hint on a
/// rigid tail is not a caller's scoped Sized evidence.
pub(crate) fn declared_step(
    types: &Store,
    definitions: &aggregate::Definitions,
    ty: TypeId,
) -> Result<Step, &'static str> {
    inspect(types, definitions, ty, Fields::Declared)
}

/// One query belongs to stable completed definitions and prepared field instances.
/// It classifies representation, not well-formedness or an environment's Sized proof.
/// Shared tails are visited once; errors are retained without rescanning other types.
#[derive(Default)]
pub struct Queries {
    states: HashMap<TypeId, State>,
    work: Vec<TypeId>,
    pub steps: usize,
}
impl Queries {
    /// Arrays, references and pointers stop traversal: their targets are not tails.
    /// Nominal fields must have been instantiated and concrete projections normalized.
    /// # Errors
    /// Rejects missing storage facts, cyclic tails and exhausted graph work.
    pub fn demand(
        &mut self,
        types: &Store,
        definitions: &aggregate::Definitions,
        root: TypeId,
    ) -> Result<Metadata, &'static str> {
        if let Some(State::Complete(result)) = self.states.get(&root) {
            return *result;
        }
        let result = self.follow(types, definitions, root);
        for ty in self.work.drain(..).rev() {
            self.states.insert(ty, State::Complete(result));
        }
        result
    }
    fn follow(
        &mut self,
        types: &Store,
        definitions: &aggregate::Definitions,
        mut ty: TypeId,
    ) -> Result<Metadata, &'static str> {
        loop {
            self.steps = self
                .steps
                .checked_add(1)
                .filter(|&steps| steps <= 1_000_000)
                .ok_or("pointer metadata work budget exceeded")?;
            match self.states.get(&ty) {
                Some(State::Complete(result)) => return *result,
                Some(State::Active) => return Err("recursive unsized storage tail"),
                None => {}
            }
            self.states.insert(ty, State::Active);
            self.work.push(ty);
            match inspect(types, definitions, ty, Fields::Physical)? {
                Step::Tail(tail) => ty = tail,
                Step::Complete(metadata) => return Ok(metadata),
            }
        }
    }
}

fn inspect(
    types: &Store,
    definitions: &aggregate::Definitions,
    ty: TypeId,
    fields: Fields,
) -> Result<Step, &'static str> {
    Ok(
        match types.kind(ty).ok_or("invalid pointer metadata type")? {
            Kind::Slice(_) | Kind::Primitive(Primitive::Str) => Step::Complete(Metadata::Length),
            Kind::Dynamic { .. } => Step::Complete(Metadata::VTable(ty)),
            Kind::Tuple(fields) => tail(types, fields)?,
            Kind::Adt(item, _) => {
                let definition = definitions
                    .get(item)
                    .ok_or("missing pointer metadata definition")?;
                if definition.kind == aggregate::Kind::Enum
                    || matches!(fields, Fields::Declared)
                        && definition.kind != aggregate::Kind::Struct
                {
                    Step::Complete(Metadata::Thin)
                } else {
                    let shape = match fields {
                        Fields::Physical => definitions.shape(types, ty)?,
                        Fields::Declared => definitions.raw_shape(types, ty)?,
                    };
                    let fields = shape
                        .variants()
                        .next()
                        .ok_or("missing pointer metadata fields")?
                        .fields;
                    tail(types, fields)?
                }
            }
            Kind::Parameter(_) | Kind::Projection(_) | Kind::Opaque(..) => {
                Step::Complete(if matches!(fields, Fields::Physical)
                    && types.is_sized(ty) == Some(true)
                {
                    Metadata::Thin
                } else {
                    Metadata::Unknown(ty)
                })
            }
            Kind::Never
            | Kind::Primitive(_)
            | Kind::Array { .. }
            | Kind::ArrayTerm { .. }
            | Kind::Reference { .. }
            | Kind::Pointer { .. }
            | Kind::Function { .. }
            | Kind::FunctionItem { .. }
            | Kind::Closure { .. } => Step::Complete(Metadata::Thin),
        },
    )
}
fn tail(types: &Store, fields: ListId) -> Result<Step, &'static str> {
    Ok(types
        .list(fields)
        .ok_or("invalid pointer metadata fields")?
        .last()
        .map_or(Step::Complete(Metadata::Thin), |&ty| Step::Tail(ty)))
}
