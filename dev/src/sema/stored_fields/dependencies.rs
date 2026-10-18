//! Owned field discovery keeps the exact next normalization request across declaration waits.
use super::{Declarations, TypeId, graph::Registry};
use crate::types::{Kind, ListId, arguments::Argument};

#[derive(Default)]
pub(super) struct Dependencies {
    pub children: Vec<TypeId>,
    pub fields: Vec<(TypeId, TypeId)>,
    pub shape: Option<Vec<ListId>>,
}
struct Nominal {
    lists: Vec<ListId>,
    variant: usize,
    index: usize,
    fields: Vec<TypeId>,
    normalized: Vec<ListId>,
    projected: bool,
    pending: Option<TypeId>,
}
pub(super) struct Reader {
    edges: Dependencies,
    nominal: Option<Nominal>,
    arguments: Vec<TypeId>,
}
impl Reader {
    pub fn new(
        registry: &mut Registry,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<Self, &'static str> {
        let mut reader = Self {
            edges: Dependencies::default(),
            nominal: None,
            arguments: Vec::new(),
        };
        match facts
            .types
            .kind(ty)
            .ok_or("invalid storage dependency type")?
        {
            Kind::Adt(_, arguments) => {
                facts.aggregates.instantiate(&mut facts.types, ty)?;
                let projected = facts.types.contains_parameters(ty) == Some(false)
                    && facts.aggregates.has_projected_fields(ty);
                let lists = facts
                    .aggregates
                    .raw_shape(&facts.types, ty)?
                    .variants()
                    .map(|variant| variant.fields)
                    .collect::<Vec<_>>();
                registry.charge(lists.len())?;
                reader.nominal = Some(Nominal {
                    lists,
                    variant: 0,
                    index: 0,
                    fields: Vec::new(),
                    normalized: Vec::new(),
                    projected,
                    pending: None,
                });
                let arguments = facts
                    .types
                    .arguments(arguments)
                    .ok_or("invalid storage arguments")?;
                registry.charge(arguments.len())?;
                reader
                    .arguments
                    .extend(arguments.filter_map(|argument| match argument {
                        Argument::Type(ty) => Some(ty),
                        _ => None,
                    }));
            }
            Kind::Tuple(list) => {
                let fields = facts.types.list(list).ok_or("invalid storage tuple")?;
                registry.charge(fields.len())?;
                reader.edges.children.extend_from_slice(fields);
            }
            Kind::Array { element, .. }
            | Kind::ArrayTerm { element, .. }
            | Kind::Slice(element) => {
                reader.edges.children.push(element);
            }
            // Referents and callable signatures do not own their nested storage.
            _ => {}
        }
        Ok(reader)
    }
    pub fn pending(&self) -> bool {
        self.nominal
            .as_ref()
            .is_some_and(|nominal| nominal.pending.is_some())
    }
    pub fn advance(
        &mut self,
        registry: &mut Registry,
        facts: &mut Declarations,
    ) -> Result<Option<TypeId>, &'static str> {
        if let Some(nominal) = &mut self.nominal {
            if let Some(raw) = nominal.pending {
                return Ok(Some(raw));
            }
            while let Some(&list) = nominal.lists.get(nominal.variant) {
                let fields = facts.types.list(list).ok_or("missing raw storage fields")?;
                if !nominal.projected {
                    registry.charge(fields.len())?;
                    self.edges.children.extend_from_slice(fields);
                } else if let Some(&field) = fields.get(nominal.index) {
                    registry.charge(1)?;
                    nominal.pending = Some(field);
                    return Ok(Some(field));
                } else {
                    nominal
                        .normalized
                        .push(facts.types.intern_list(&nominal.fields)?);
                    nominal.fields.clear();
                }
                nominal.variant += 1;
                nominal.index = 0;
            }
            if nominal.projected {
                self.edges.shape = Some(std::mem::take(&mut nominal.normalized));
            }
            self.nominal = None;
        }
        self.edges.children.append(&mut self.arguments);
        Ok(None)
    }
    pub fn normalized(&mut self, facts: &Declarations, normal: TypeId) -> Result<(), &'static str> {
        let nominal = self
            .nominal
            .as_mut()
            .ok_or("storage field reader is inactive")?;
        let raw = nominal
            .pending
            .take()
            .ok_or("storage field has no pending normalization")?;
        if facts.types.contains_projections(normal) != Some(false) {
            return Err("concrete storage retains an associated projection");
        }
        if facts.types.contains_projections(raw) == Some(true) {
            self.edges.fields.push((raw, normal));
        }
        nominal.fields.push(normal);
        self.edges.children.push(normal);
        nominal.index += 1;
        Ok(())
    }
    pub fn finish(self) -> Dependencies {
        self.edges
    }
}
