//! Measure candidate containing layouts before changing any live body fields.
use super::catalog::CatalogSlot;
use super::*;
use crate::{
    body::{ir, regions},
    types::TypeId,
};
use std::sync::Arc;

struct OriginBody<O> {
    static_types: Option<Arc<crate::support::hash::HashMap<ir::DefId, TypeId>>>,
    mutable_statics: Option<Arc<crate::support::hash::HashSet<ir::DefId>>>,
    const_context: bool,
    instance: Option<crate::types::instance::Instance>,
    owned: Option<Box<crate::body::drops::values::Facts>>,
    storage: Option<crate::body::storage::Facts>,
    promotions: Option<crate::body::promote::Facts>,
    resolved_callables: Option<Box<ir::ResolvedCallables>>,
    closure_calls: Option<Box<crate::body::invocation_entries::Entries>>,
    temporaries: Vec<ir::Temporary>,
    guards: Vec<ir::Guard>,
    guard_places: Vec<ir::Place>,
    projections: Vec<ir::Projection>,
    owner: ir::Owner,
    origin: O,
    result: TypeId,
    locals: Vec<ir::Local>,
    values: Vec<ir::Value>,
    scalars: Vec<crate::sema::model::Scalar>,
    arguments: Vec<ir::ValueId>,
    statements: Vec<ir::Statement>,
    blocks: Vec<ir::Block>,
}
struct OriginContract<O> {
    key: ir::BodyKey,
    origin: O,
    signature: Option<TypeId>,
    universals: u32,
    assumptions: Vec<crate::types::outlives::Pair>,
    parameters: Vec<crate::types::parameters::ParameterId>,
    type_assumptions: Vec<crate::types::parameters::clauses::Clause>,
    obligations: Vec<regions::Obligation>,
    proofs: Vec<regions::Proof>,
    builtins: Option<Arc<crate::sema::model::BuiltinRequirements>>,
    output: Option<Arc<crate::sema::nested_outputs::Link>>,
    require_result_formation: bool,
}
struct SiteObligation<S> {
    site: S,
    kind: regions::Kind,
}
struct SiteProof<S> {
    site: S,
    sources: Vec<S>,
    proof: crate::types::lifetime_proof::Obligations,
}
struct SitePredicate<S> {
    site: S,
    value: crate::types::parameters::clauses::Clause,
}
struct LegacyImported {
    dependencies: Arc<crate::support::crates::Dependencies>,
    definitions: crate::support::hash::HashMap<crate::support::crates::ExternalDef, ir::DefId>,
    functions: Vec<(ir::DefId, usize, usize)>,
    constants: Vec<(ir::DefId, usize, usize)>,
    records: Vec<(ir::DefId, usize, usize)>,
    statics: Vec<(ir::DefId, usize, usize)>,
}
struct LegacyLinkage {
    identity: Option<crate::support::crates::CrateIdentity>,
    prefix: Option<String>,
    symbols: crate::support::hash::HashMap<ir::DefId, String>,
    imported_constants: crate::support::hash::HashSet<ir::DefId>,
    imported_records: crate::support::hash::HashSet<ir::DefId>,
    imported_storage: crate::support::hash::HashMap<ir::DefId, bool>,
    work: crate::sema::crates::ImportWork,
}

#[test]
fn frozen_catalog_sharing_adds_only_cold_import_owner_slots() {
    assert_eq!(
        size_of::<crate::resolve::crates::Imported>(),
        size_of::<LegacyImported>() + size_of::<usize>()
    );
    assert_eq!(
        size_of::<crate::sema::crates::Linkage>(),
        size_of::<LegacyLinkage>() + size_of::<usize>()
    );
    assert_eq!(size_of::<CatalogSlot>(), size_of::<usize>());
    assert_eq!(size_of::<ir::Value>(), 24);
    println!(
        "resolver_import={}/{} sema_linkage={}/{} catalog={} slot={} Rc_control={}",
        size_of::<LegacyImported>(),
        size_of::<crate::resolve::crates::Imported>(),
        size_of::<LegacyLinkage>(),
        size_of::<crate::sema::crates::Linkage>(),
        size_of::<Catalog>(),
        size_of::<CatalogSlot>(),
        2 * size_of::<usize>()
    );
}

#[test]
fn source_free_candidates_preserve_hot_value_and_lazy_catalog_layouts() {
    assert_eq!(size_of::<ir::Value>(), 24);
    assert_eq!(size_of::<CatalogSlot>(), size_of::<usize>());
    assert_eq!(size_of::<ImportedBodyId>(), 8);
    assert_eq!(size_of::<ImportedBodyOrigin>(), 12);
    assert_eq!(size_of::<BodyOrigin>(), 12);
    assert_eq!(size_of::<ProofSite>(), 12);
    assert_eq!(size_of::<Option<ImportedBodyId>>(), 8);
    assert_eq!(size_of::<BodySite>(), size_of::<NodeId>());
    assert_eq!(size_of::<Option<BodySite>>(), size_of::<Option<NodeId>>());
    assert_eq!(size_of::<OriginBody<NodeKey>>(), size_of::<ir::Body>());
    assert_eq!(align_of::<OriginBody<NodeKey>>(), align_of::<ir::Body>());
    assert_eq!(
        size_of::<OriginContract<NodeKey>>(),
        size_of::<regions::Contract>()
    );
    assert_eq!(
        size_of::<SiteObligation<NodeKey>>(),
        size_of::<regions::Obligation>()
    );
    assert_eq!(
        size_of::<SiteProof<NodeKey>>(),
        size_of::<crate::sema::model::LifetimeProof>()
    );
    assert_eq!(
        size_of::<SitePredicate<NodeKey>>(),
        size_of::<crate::sema::generics::Predicate>()
    );
    println!(
        "identity={} origin={} site={} proof_site={} catalog={} row={} artifact={} slot={} owner_row={} declaration_row={}",
        size_of::<ImportedBodyId>(),
        size_of::<BodyOrigin>(),
        size_of::<BodySite>(),
        size_of::<ProofSite>(),
        size_of::<Catalog>(),
        size_of::<catalog::Row>(),
        size_of::<catalog::Artifact>(),
        size_of::<CatalogSlot>(),
        size_of::<catalog::OwnerRow>(),
        size_of::<catalog::DeclarationRow>()
    );
    println!(
        "body={}/{} contract={}/{} obligation={}/{} lifetime_proof={}/{} Value={}",
        size_of::<ir::Body>(),
        size_of::<OriginBody<BodyOrigin>>(),
        size_of::<regions::Contract>(),
        size_of::<OriginContract<BodyOrigin>>(),
        size_of::<regions::Obligation>(),
        size_of::<SiteObligation<ProofSite>>(),
        size_of::<crate::sema::model::LifetimeProof>(),
        size_of::<SiteProof<ProofSite>>(),
        size_of::<ir::Value>()
    );
    println!(
        "canonical_predicate={} located_predicate={}/{} builtin_use={}/{}",
        size_of::<crate::types::parameters::predicates::Predicate>(),
        size_of::<crate::sema::generics::Predicate>(),
        size_of::<SitePredicate<ProofSite>>(),
        size_of::<(NodeKey, crate::types::parameters::predicates::Predicate)>(),
        size_of::<(ProofSite, crate::types::parameters::predicates::Predicate)>(),
    );
}
