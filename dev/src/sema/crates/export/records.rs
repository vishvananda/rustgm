//! Discover completed closed records once, then encode canonical shared roots.
use super::super::type_transport;
use super::{Declarations, DefId, Error, Prepared, PreparedFunction, Report, error};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    resolve::model::Access,
    sema::members::{Field, Name, VariantId},
    support::crates::{
        CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Nominals, Record,
        RecordField, RecordRef, TypeGraph,
    },
    syntax::item::Shape,
    types::{GenericArgs, ItemId, Kind, aggregate},
};

struct Declaration {
    item: ItemId,
    identity: ExternalDef,
    public: bool,
    variant: VariantId,
}

pub(super) fn finish(
    report: &Report,
    facts: &Declarations,
    identity: CrateIdentity,
    dependencies: Vec<CrateIdentity>,
    prepared: Prepared,
) -> Result<Interface, Error> {
    let Prepared {
        functions,
        constants,
        records: public,
        storage,
    } = prepared;
    let mut records = discover(report, facts, &identity, &functions, &public, &storage)?;
    if records.is_empty() && storage.entries.is_empty() {
        let functions = functions
            .into_iter()
            .map(|function| {
                Function::new(
                    function.definition,
                    type_transport::export(facts, function.signature)?,
                    function.symbol,
                )
                .map_err(error)
            })
            .collect::<Result<_, _>>()?;
        return Interface::with_constants(identity, dependencies, functions, constants)
            .map_err(error);
    }
    records.sort_unstable_by(|left, right| left.identity.cmp(&right.identity));
    let ordinals = records
        .iter()
        .enumerate()
        .map(|(index, record)| {
            u16::try_from(index)
                .map(|index| (record.item, RecordRef(index)))
                .map_err(|_| error("too many exported nominal identities"))
        })
        .collect::<Result<HashMap<_, _>, _>>()?;
    let mut graph = type_transport::Export::default();
    let signatures = functions
        .iter()
        .map(|function| {
            type_transport::export_roots(facts, function.signature, &mut graph, &ordinals)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let mut wire_records = Vec::with_capacity(records.len());
    for record in records {
        let mut wire_fields = Vec::new();
        for field in fields(facts, record.variant)? {
            let Name::Named(name) = field.name else {
                return Err(error("metadata record requires named fields"));
            };
            wire_fields.push(
                RecordField::new(
                    facts.members.names.get(name).to_owned(),
                    graph.root(facts, field.ty, &ordinals)?,
                    field.access == Access::Public,
                )
                .map_err(error)?,
            );
        }
        wire_records.push(Record::new(record.identity, record.public, wire_fields).map_err(error)?);
    }
    let storage = storage.encode(facts, &mut graph, &ordinals)?;
    let graph = TypeGraph::new(graph.nodes).map_err(error)?;
    let functions = functions
        .into_iter()
        .zip(signatures)
        .map(|(function, signature)| {
            Function::new(
                function.definition,
                signature.finish(graph.clone())?,
                function.symbol,
            )
            .map_err(error)
        })
        .collect::<Result<_, _>>()?;
    if let Some(storage) = storage {
        Interface::with_storage(
            identity,
            dependencies,
            functions,
            constants,
            Nominals::for_storage(graph, wire_records).map_err(error)?,
            storage,
        )
        .map_err(error)
    } else {
        Interface::with_nominals(
            identity,
            dependencies,
            functions,
            constants,
            Nominals::new(graph, wire_records).map_err(error)?,
        )
        .map_err(error)
    }
}

fn fields(facts: &Declarations, variant: VariantId) -> Result<&[Field], Error> {
    facts
        .members
        .fields(variant)
        .ok_or_else(|| error("missing exported record fields"))
}

fn discover(
    report: &Report,
    facts: &Declarations,
    identity: &CrateIdentity,
    functions: &[PreparedFunction],
    public: &[DefId],
    storage: &super::static_exports::Prepared,
) -> Result<Vec<Declaration>, Error> {
    let public_set: HashSet<_> = public.iter().copied().collect();
    let mut work: Vec<_> = functions
        .iter()
        .map(|function| function.signature)
        .collect();
    work.extend(storage.entries.iter().map(|entry| entry.ty));
    for definition in public {
        work.push(
            facts.definitions[definition.0 as usize]
                .ok_or_else(|| error("missing nominal export type"))?,
        );
    }
    let mut seen = HashSet::default();
    let mut records = Vec::new();
    let mut contracts = None;
    while let Some(ty) = work.pop() {
        if !seen.insert(ty) {
            continue;
        }
        if seen.len() > TypeGraph::MAX_NODES || work.len() > 100_000 {
            return Err(error(
                "metadata export type discovery exceeds profile budget",
            ));
        }
        match facts.types.kind(ty) {
            Some(Kind::Function {
                parameters, result, ..
            }) => {
                work.extend(
                    facts
                        .types
                        .list(parameters)
                        .ok_or_else(|| error("missing export parameters"))?,
                );
                work.push(result);
            }
            Some(Kind::Reference { referent, .. }) => work.push(referent),
            Some(Kind::Adt(item, GenericArgs::EMPTY)) => {
                if contracts.is_none() {
                    contracts = Some(Contracts::new(report, facts)?);
                }
                let contracts = contracts
                    .as_ref()
                    .ok_or_else(|| error("missing export contract index"))?;
                let variant = validate(facts, item, contracts)?;
                work.extend(fields(facts, variant)?.iter().map(|field| field.ty));
                records.push(Declaration {
                    item,
                    identity: path(report, identity, DefId(item.0), ExternalKind::Record)?,
                    public: public_set.contains(&DefId(item.0)),
                    variant,
                });
            }
            Some(Kind::Adt(_, _)) => {
                return Err(error("generic nominals are outside the metadata profile"));
            }
            _ => {
                super::scalar(facts, ty)?;
            }
        }
    }
    Ok(records)
}

pub(super) fn path(
    report: &Report,
    identity: &CrateIdentity,
    definition: DefId,
    kind: ExternalKind,
) -> Result<ExternalDef, Error> {
    let entry = &report.resolution.definitions[definition.0 as usize];
    let scope = &report.resolution.scopes[entry.scope.0 as usize];
    if entry.node.is_none()
        || report.resolution.module_scopes[scope.module.0 as usize] != entry.scope
    {
        return Err(error(
            "external or block-local nominals are outside the metadata profile",
        ));
    }
    let mut module = scope.module;
    let mut names = vec![report.package.names.get(entry.name).to_owned()];
    while let Some(parent) = report.package.modules[module.0 as usize].parent {
        if names.len() >= 64 {
            return Err(error("nominal export path exceeds profile budget"));
        }
        names.push(
            report
                .package
                .names
                .get(report.package.modules[module.0 as usize].name)
                .to_owned(),
        );
        module = parent;
    }
    names.reverse();
    ExternalDef::new(identity.clone(), names, kind).map_err(error)
}

fn validate(facts: &Declarations, item: ItemId, contracts: &Contracts) -> Result<VariantId, Error> {
    let owner = DefId(item.0);
    let definition = facts
        .aggregates
        .get(item)
        .ok_or_else(|| error("missing completed nominal export"))?;
    let variants = facts.members.variants(item);
    let [variant] = variants else {
        return Err(error("metadata requires one named-record field group"));
    };
    if definition.kind != aggregate::Kind::Struct
        || facts.members.variants[variant.0 as usize].shape != Shape::Record
        || definition.repr != aggregate::Repr::default()
        || facts.aggregates.shared_mutation(item) != Some(aggregate::SharedMutation::Ordinary)
        || facts.aggregates.destruction(item) != Some(aggregate::FieldDestruction::Fields)
    {
        return Err(error(
            "nominal shape or representation is outside the metadata profile",
        ));
    }
    if facts.generics.has_parameters(owner)
        || facts
            .generics
            .predicates
            .get(&owner)
            .is_some_and(|bounds| !bounds.is_empty())
        || !facts.generics.nominal_contracts.contains(&owner)
    {
        return Err(error(
            "nominal generic contract is outside the metadata profile",
        ));
    }
    if facts.derives.contains_key(&item)
        || facts.equality_derives.contains_key(&item)
        || facts.destructors.has_declared(item)
        || contracts.blanket
        || contracts.owners.contains(&item)
        || facts.members.external_open(item)
        || facts.members.variants[variant.0 as usize]
            .origin
            .source()
            .is_some_and(|key| contracts.open.contains(&key))
    {
        return Err(error(
            "nominal trait, member or non-exhaustive contract is outside the metadata profile",
        ));
    }
    Ok(*variant)
}

/// Index unsupported semantic owners once, only when a nominal is actually exported.
struct Contracts {
    owners: HashSet<ItemId>,
    blanket: bool,
    open: HashSet<crate::resolve::unit::NodeKey>,
}
impl Contracts {
    fn new(report: &Report, facts: &Declarations) -> Result<Self, Error> {
        let mut owners: HashSet<_> = facts
            .methods
            .inherent
            .keys()
            .map(|&(item, _)| item)
            .collect();
        let mut blanket = false;
        for implementation in facts.generics.implementations() {
            let mut subject = implementation.predicate.ty;
            while let Some(Kind::Reference { referent, .. }) = facts.types.kind(subject) {
                subject = referent;
            }
            match facts.types.kind(subject) {
                Some(Kind::Adt(item, _)) => {
                    owners.insert(item);
                }
                Some(Kind::Parameter(_) | Kind::Projection(_)) => blanket = true,
                _ => {}
            }
        }
        let mut open = HashSet::default();
        for (file, unit) in report.package.units.iter().enumerate() {
            let file = u32::try_from(file).map_err(|_| error("too many export source files"))?;
            for owner in &unit.configured.owners {
                if !owner.enabled {
                    continue;
                }
                if owner.attributes.clone().any(|index| {
                    let effect = &unit.configured.effects[index];
                    let forest = &unit.configured.ast.attributes[effect.capture as usize].forest;
                    effect.attribute.attribute.name(forest, unit.source.text())
                        == Some("non_exhaustive")
                }) {
                    let mut node = owner.node;
                    while let crate::syntax::ast::Kind::Attributed { value, .. } =
                        unit.configured.ast.node(node).kind
                    {
                        node = value;
                    }
                    open.insert(crate::resolve::unit::NodeKey {
                        file: crate::resolve::unit::FileId(file),
                        node,
                    });
                }
            }
        }
        Ok(Self {
            owners,
            blanket,
            open,
        })
    }
}
