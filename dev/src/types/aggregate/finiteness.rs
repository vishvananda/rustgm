//! Finite storage follows declaration fields and stored parameter positions, never expanded instances.
mod conditions;
use super::Definitions;
use crate::support::hash::{HashMap, HashSet};
use crate::types::{
    GenericArgs, ItemId, Kind, Store, TypeId, arguments::Argument, parameters::ParameterId,
};
use conditions::{Conditions, Term};

#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    Invalid(&'static str),
    Recursive(ItemId),
}
impl From<&'static str> for Error {
    fn from(message: &'static str) -> Self {
        Self::Invalid(message)
    }
}
#[derive(Clone, Copy)]
enum Slot {
    Type(Term),
    Lifetime,
    Const(TypeId),
}
#[derive(Default)]
struct Graph {
    slots: HashMap<ItemId, Vec<Slot>>,
    parameters: HashMap<ParameterId, Term>,
    equations: Conditions,
    edges: Vec<(ItemId, ItemId, Term)>,
    work: Vec<(TypeId, Term)>,
    seen: HashSet<(TypeId, Term)>,
}
/// Stored type-argument positions include zero-length arrays' alignment requirements.
/// Reference/raw/function pointer targets never create storage dependencies.
pub struct Storage {
    slots: HashMap<ItemId, Vec<bool>>,
    pub steps: usize,
}
impl Storage {
    /// Infer stored argument positions and reject every by-value declaration cycle.
    /// Finite applications such as `Wrap<Wrap<u8>>` are not declaration cycles.
    /// # Errors
    /// Rejects malformed schemas, recursive storage and bounded graph exhaustion.
    pub fn infer(
        types: &Store,
        definitions: &Definitions,
        schemas: &[(ItemId, GenericArgs)],
    ) -> Result<Self, Error> {
        let mut graph = Graph::default();
        for &(item, arguments) in schemas {
            graph.declare(types, item, arguments)?;
        }
        for &(item, _) in schemas {
            let definition = definitions
                .get(item)
                .ok_or("missing storage field template")?;
            graph.seen.clear();
            for variant in definitions.variants(definition) {
                graph.equations.charge(1)?;
                for &ty in types.list(variant.fields).ok_or("invalid storage fields")? {
                    graph.visit(types, item, ty)?;
                }
            }
        }
        graph.equations.solve()?;
        graph.check_cycles(schemas)?;
        let slots = graph
            .slots
            .into_iter()
            .map(|(item, slots)| {
                let values = slots
                    .into_iter()
                    .map(|slot| matches!(slot, Slot::Type(term) if graph.equations.value(term)))
                    .collect();
                (item, values)
            })
            .collect();
        Ok(Self {
            slots,
            steps: graph.equations.steps,
        })
    }
    #[must_use]
    pub fn arguments(&self, item: ItemId) -> Option<&[bool]> {
        self.slots.get(&item).map(Vec::as_slice)
    }
}
impl Graph {
    fn declare(
        &mut self,
        types: &Store,
        item: ItemId,
        arguments: GenericArgs,
    ) -> Result<(), &'static str> {
        self.equations.charge(1)?;
        if self.slots.contains_key(&item) {
            return Err("duplicate storage schema");
        }
        let mut slots = Vec::new();
        for (index, argument) in types
            .arguments(arguments)
            .ok_or("invalid storage schema")?
            .enumerate()
        {
            self.equations.charge(1)?;
            slots.push(match argument {
                Argument::Type(ty) => {
                    let Some(Kind::Parameter(id)) = types.kind(ty) else {
                        return Err("storage schema requires rigid type parameters");
                    };
                    let parameter = types.parameter(id).ok_or("invalid storage parameter")?;
                    if parameter.owner != item || parameter.index as usize != index {
                        return Err("storage parameter has the wrong owner or slot");
                    }
                    let term = self.equations.variable()?;
                    self.parameters.insert(id, term);
                    Slot::Type(term)
                }
                Argument::Lifetime(_) => Slot::Lifetime,
                Argument::Const(id) => Slot::Const(
                    types
                        .const_type(id)
                        .ok_or("invalid storage const argument")?,
                ),
            });
        }
        self.slots.insert(item, slots);
        Ok(())
    }
    fn visit(&mut self, types: &Store, owner: ItemId, root: TypeId) -> Result<(), &'static str> {
        self.work.push((root, Term::ALWAYS));
        while let Some((ty, condition)) = self.work.pop() {
            self.equations.charge(1)?;
            if !self.seen.insert((ty, condition)) {
                continue;
            }
            match types.kind(ty).ok_or("invalid storage type")? {
                Kind::Parameter(id) => {
                    if types.parameter(id).is_none_or(|p| p.owner != owner) {
                        return Err("storage field parameter is outside its declaration");
                    }
                    let target = *self
                        .parameters
                        .get(&id)
                        .ok_or("missing storage parameter schema")?;
                    self.equations.require(target, condition)?;
                }
                Kind::Tuple(fields) => {
                    let fields = types.list(fields).ok_or("invalid storage tuple")?;
                    self.equations.charge(fields.len())?;
                    self.work
                        .extend(fields.iter().rev().map(|&ty| (ty, condition)));
                }
                Kind::Array { element, .. }
                | Kind::ArrayTerm { element, .. }
                | Kind::Slice(element) => {
                    self.work.push((element, condition));
                }
                Kind::Adt(item, arguments) => {
                    self.edges.push((owner, item, condition));
                    self.arguments(types, item, arguments, condition)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    fn arguments(
        &mut self,
        types: &Store,
        item: ItemId,
        arguments: GenericArgs,
        condition: Term,
    ) -> Result<(), &'static str> {
        let slots = self
            .slots
            .get(&item)
            .ok_or("missing referenced storage schema")?;
        let arguments = types
            .arguments(arguments)
            .ok_or("invalid storage arguments")?;
        if arguments.len() != slots.len() {
            return Err("storage argument arity mismatch");
        }
        for (&slot, argument) in slots.iter().zip(arguments) {
            self.equations.charge(1)?;
            match (slot, argument) {
                (Slot::Type(term), Argument::Type(ty)) => {
                    let product = self.equations.product(condition, term)?;
                    self.work.push((ty, product));
                }
                (Slot::Lifetime, Argument::Lifetime(_)) => {}
                (Slot::Const(ty), Argument::Const(id)) if types.const_type(id) == Some(ty) => {}
                _ => return Err("storage argument kind mismatch"),
            }
        }
        Ok(())
    }
    fn check_cycles(&mut self, schemas: &[(ItemId, GenericArgs)]) -> Result<(), Error> {
        let mut edges: HashMap<ItemId, Vec<ItemId>> = HashMap::default();
        let mut seen = HashSet::default();
        for &(owner, dependency, condition) in &self.edges {
            self.equations.charge(1)?;
            if self.equations.value(condition) && seen.insert((owner, dependency)) {
                edges.entry(owner).or_default().push(dependency);
            }
        }
        let mut states = HashMap::default();
        let mut work = Vec::new();
        for &(root, _) in schemas {
            work.push((root, false));
            while let Some((item, finish)) = work.pop() {
                self.equations.charge(1)?;
                if states.get(&item) == Some(&true) {
                    continue;
                }
                if finish {
                    states.insert(item, true);
                    continue;
                }
                if states.insert(item, false).is_some() {
                    return Err(Error::Recursive(item));
                }
                work.push((item, true));
                if let Some(dependencies) = edges.get(&item) {
                    self.equations.charge(dependencies.len())?;
                    work.extend(dependencies.iter().rev().map(|&item| (item, false)));
                }
            }
        }
        Ok(())
    }
}
