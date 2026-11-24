//! Required members retain absence; every actual provided body reserves one original.
mod formals;
use super::{
    DeclarationRef, Declarations, DefId, OwnerKind, Report, Stage, canonical_nodes::Context, g, s,
};
use crate::{
    support::crates::owner_catalog::{BodyKind, BodyOwner, BodyRef},
    support::{bindings::LangItem, constness::Constness, hash::HashMap, retained},
    types::Kind,
};

#[derive(Default)]
pub(super) struct Index {
    pub(super) bodies: HashMap<DefId, BodyRef>,
    pub(super) contexts: Vec<Context>,
    pub(super) originals: Vec<Option<(s::Original, s::BodyInputs)>>,
    pub(super) headers: HashMap<crate::types::ItemId, u32>,
    pub(super) exact: bool,
    pub(super) callable_effects: bool,
    pub(super) conversions: bool,
}

impl Stage {
    pub(crate) fn callable_binder(
        &mut self,
        item: crate::types::ItemId,
    ) -> Result<(g::BinderRef, u32), &'static str> {
        self.charge(3)?;
        let index = self
            .functions
            .headers
            .get(&item)
            .copied()
            .ok_or("source call target has no staged actual function header")?;
        let header = self
            .rows
            .functions
            .get(index as usize)
            .ok_or("source callable header index is stale")?;
        let binder = header.callable.binder;
        // Version thirteen: an inherent method's captured public slots come
        // from its application, so a Call opens only the slots after them.
        let count = self
            .rows
            .binders
            .get(binder.0 as usize)
            .ok_or("source callable frame is unknown")?
            .count
            .checked_sub(header.regions.captures)
            .ok_or("source callable captures exceed its public frame")?;
        Ok((binder, count))
    }
    pub(super) fn functions(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        identity: &crate::support::crates::CrateIdentity,
        constants: &[DefId],
    ) -> Result<(), &'static str> {
        let mut count = 0usize;
        let mut constants = constants.iter().copied().peekable();
        for index in 0..self.definitions.len() {
            self.charge(1)?;
            let definition =
                DefId(u32::try_from(index).map_err(|_| "source function identity overflow")?);
            if constants.peek() == Some(&definition) {
                constants.next();
                self.reserve_constant_original(facts, definition)?;
            }
            let Some(&index) = facts.function_indices.get(&definition) else {
                continue;
            };
            let function = *facts
                .functions
                .get(index)
                .ok_or("source function header index is stale")?;
            if function.definition != definition {
                return Err("source function header index names another declaration");
            }
            let named = facts
                .foreign
                .as_deref()
                .and_then(|registry| registry.function(definition))
                .is_some_and(|function| {
                    function.naming != crate::support::linkage::Naming::Rust
                        || function.linkage != crate::support::linkage::Linkage::Internal
                });
            let targeted = facts.target_feature_functions.contains_key(&definition);
            if named && (targeted || !self.exact_linkage()) || targeted && !self.body_nodes() {
                return Err(
                    "source named/foreign/target-feature function metadata requires its wider profile",
                );
            }
            if targeted {
                target_feature_form(facts, &function)?;
            }
            if facts
                .source_bindings
                .as_deref()
                .map(|c| c.intrinsic_kind(definition))
                .transpose()?
                .flatten()
                .is_some_and(|kind| {
                    matches!(
                        kind,
                        crate::support::bindings::SourceIntrinsic::ArithOffset
                            | crate::support::bindings::SourceIntrinsic::Declaration(_)
                            | crate::support::bindings::SourceIntrinsic::UncheckedSub
                            | crate::support::bindings::SourceIntrinsic::SubWithOverflow
                            | crate::support::bindings::SourceIntrinsic::ColdPath
                            | crate::support::bindings::SourceIntrinsic::TransmuteUnchecked
                    )
                })
            {
                return Err(
                    "source intrinsic declaration semantics require a wider metadata profile",
                );
            }
            let declaration = self.declaration(definition)?;
            if matches!(
                facts
                    .promotable_contract(report, definition)
                    .map_err(|_| "source promotion header validation failed")?,
                crate::sema::header_attributes::ContractKnowledge::KnownPresent(_)
            ) {
                return Err(
                    "source promotable declaration transport requires its wider metadata profile",
                );
            }
            self.prepare_signature_arena(facts, function.signature)?;
            self.function(report, facts, &function, declaration)?;
            self.exact_callable(report, facts, &function, declaration)?;
            if self.declaration_policies()
                && let Some(header) = self.rows.functions.last()
                && s::nested_original(&self.rows.declarations, header).is_some()
            {
                self.install_private_symbol(facts, identity, definition)?;
            }
            count = count
                .checked_add(1)
                .ok_or("source function count overflow")?;
        }
        if count != facts.functions.len() {
            return Err("source function index omitted an actual declaration header");
        }
        if constants.next().is_some() {
            return Err("source original index omitted an actual constant initializer");
        }
        Ok(())
    }

    fn function(
        &mut self,
        report: &Report,
        facts: &Declarations,
        function: &crate::sema::model::Function,
        declaration: DeclarationRef,
    ) -> Result<(), &'static str> {
        let scheme = facts
            .lifetimes
            .functions
            .get(&function.definition)
            .ok_or("source function has no completed lifetime scheme")?;
        // Captures come from the actual associated declaration. A method
        // never introduces another set of slots for the same lifetimes.
        let captured = self.body_nodes()
            && facts
                .generics
                .parents
                .get(&function.definition)
                .is_some_and(|&owner| {
                    declared_captures(facts, owner, self.declaration_policies())
                        == Some(scheme.captures)
                });
        if scheme.captures != 0 && !captured || !scheme.early.is_empty() {
            return Err(
                "source function early/captured callable frame transport is not implemented",
            );
        }
        let parent = Context {
            owner: declaration,
            frame: None,
            body: None,
        };
        let (original, context) = self.signature(facts, function.signature, parent, false)?;
        if self.rows.binders[original.binder.0 as usize].count != scheme.total {
            return Err("source original signature differs from its actual lifetime scheme");
        }
        let Some(Kind::FunctionItem {
            item, signature, ..
        }) = facts
            .definitions
            .get(function.definition.0 as usize)
            .copied()
            .flatten()
            .and_then(|ty| facts.types.kind(ty))
        else {
            return Err("source function has no actual public callable declaration");
        };
        if item.0 != function.definition.0 {
            return Err("source public callable belongs to another declaration");
        }
        let (callable, _) = self.signature(facts, signature, parent, true)?;
        let regions = self.function_regions(scheme, context)?;
        let inputs = self.function_inputs(facts, function.definition, context)?;
        let (formals, receiver) = self.formals(report, facts, function, context)?;
        let body = self.function_body(report, facts, function, context, original.result)?;
        // Lexical containment is already in Declaration.definition.parent.
        // This is the distinct actual associated owner, not the root module.
        let parent = self.associated_parent(facts, function.definition)?;
        let header = s::FunctionHeader {
            declaration,
            scope: declaration,
            parent,
            callable,
            original,
            regions,
            inputs,
            declared_inputs: u32::try_from(
                facts
                    .generics
                    .declared_predicates(function.definition)
                    .len(),
            )
            .map_err(|_| "source declared clause count overflow")?,
            formals,
            receiver,
            constness: if self.callable_effects() {
                self.charge(if self.conversion_roles() { 20 } else { 4 })?;
                let roles = RequiredRoles {
                    conversions: self.conversion_roles(),
                    callables: self.body_nodes(),
                    declaration_policies: self.declaration_policies(),
                };
                written_constness(report, facts, function, roles)?
            } else if facts.is_const_function(function.definition) {
                Constness::Const
            } else {
                Constness::Ordinary
            },
            body,
        };
        self.reserve(4 * size_of::<s::FunctionHeader>())?;
        self.reserve(retained::new_map::<crate::types::ItemId, u32>(1))?;
        let row = u32::try_from(self.rows.functions.len())
            .map_err(|_| "source function header table overflow")?;
        if self
            .functions
            .headers
            .insert(crate::types::ItemId(function.definition.0), row)
            .is_some()
        {
            return Err("source function header has duplicate actual identity");
        }
        self.rows.functions.push(header);
        Ok(())
    }

    fn associated_parent(
        &mut self,
        facts: &Declarations,
        definition: DefId,
    ) -> Result<Option<DeclarationRef>, &'static str> {
        facts
            .associated
            .owner(definition)
            .and_then(|owner| facts.associated.get(owner))
            .map(|owner| {
                let owner = owner.source_owner()?;
                match owner.kind {
                    OwnerKind::Trait { definition, .. } => Ok(definition),
                    OwnerKind::Impl { .. } => Ok(owner.self_definition),
                    OwnerKind::ImportedImpl(_) | OwnerKind::ImportedInherent(_) => {
                        Err("source function cannot acquire imported syntax ownership")
                    }
                }
            })
            .transpose()?
            .map(|owner| self.declaration(owner))
            .transpose()
    }
    fn function_inputs(
        &mut self,
        facts: &Declarations,
        definition: DefId,
        context: Context,
    ) -> Result<g::Range, &'static str> {
        let start =
            u32::try_from(self.rows.clauses.len()).map_err(|_| "source input table overflow")?;
        for input in crate::sema::declaration_inputs::own(
            &facts.generics,
            facts.crates.as_deref(),
            definition,
        )? {
            self.clause(facts, input, context, None)?;
        }
        Ok(g::Range {
            start,
            end: u32::try_from(self.rows.clauses.len())
                .map_err(|_| "source input table overflow")?,
        })
    }

    fn function_body(
        &mut self,
        report: &Report,
        facts: &Declarations,
        function: &crate::sema::model::Function,
        context: Context,
        future: g::TypeRef,
    ) -> Result<s::FunctionBody, &'static str> {
        let Some(source) = function.body else {
            let kind = facts
                .source_bindings
                .as_deref()
                .map(|catalog| catalog.intrinsic_kind(function.definition))
                .transpose()?
                .flatten();
            if let Some(operation) = s::BodylessIntrinsic::ALL
                .into_iter()
                .find(|operation| kind == Some(operation.kind()))
            {
                return if self.declaration_policies() {
                    Ok(s::FunctionBody::Intrinsic(operation))
                } else {
                    Err("source bodyless intrinsic requires version fourteen")
                };
            }
            return Ok(s::FunctionBody::Absent);
        };
        let body = self.reserve_original(function.definition, context, BodyKind::Function)?;
        if self.compiler_operations
            && let Some(recipe) = crate::sema::compiler_items::async_operation_recipe(
                report,
                facts,
                function.definition,
            )
            .map_err(
                |_| "source compiler operation failed its authenticated declaration/body contract",
            )?
        {
            if recipe.function.definition != function.definition
                || recipe.body != source
                || recipe.parameters.len() != 1
                || facts.types.kind(recipe.future)
                    != Some(Kind::Opaque(recipe.opaque, recipe.captures))
                || self.ty(facts, recipe.future, context)? != future
            {
                return Err(
                    "compiler operation changed its genuine checked body/constructor identity",
                );
            }
            let output = self.ty(facts, recipe.output, context)?;
            self.reserve(4 * size_of::<s::CompilerOperation>())?;
            let operation = s::CompilerOperationRef(
                u32::try_from(self.rows.compiler_operations.len())
                    .map_err(|_| "compiler operation table overflow")?,
            );
            self.rows.compiler_operations.push(s::CompilerOperation {
                body,
                declaration: context.owner(),
                future,
                output,
                kind: s::CompilerOperationKind::AsyncDropInPlace { target_formal: 0 },
            });
            return Ok(s::FunctionBody::CompilerOperation(operation));
        }
        Ok(s::FunctionBody::Original(body))
    }

    pub(super) fn reserve_original(
        &mut self,
        definition: DefId,
        context: Context,
        kind: BodyKind,
    ) -> Result<BodyRef, &'static str> {
        self.reserve(
            4 * (size_of::<BodyOwner>()
                + size_of::<Context>()
                + size_of::<Option<(s::Original, s::BodyInputs)>>())
                + retained::new_map::<DefId, BodyRef>(1),
        )?;
        let id = BodyRef(
            u32::try_from(self.rows.body_owners.len())
                .map_err(|_| "source original owner table overflow")?,
        );
        if self.functions.bodies.insert(definition, id).is_some() {
            return Err("source declaration has duplicate original bodies");
        }
        self.rows.body_owners.push(BodyOwner {
            declaration: context.owner(),
            parent: None,
            kind,
            ordinal: 0,
        });
        self.functions.contexts.push(Context {
            body: Some(id),
            ..context
        });
        self.functions.originals.push(None);
        Ok(id)
    }

    fn function_regions(
        &mut self,
        scheme: &crate::sema::lifetimes::FunctionRegions,
        context: Context,
    ) -> Result<s::RegionScheme, &'static str> {
        let start = u32::try_from(self.rows.region_slots.len())
            .map_err(|_| "source function slot table overflow")?;
        self.reserve(
            scheme
                .late
                .len()
                .saturating_mul(4 * size_of::<s::SlotRef>()),
        )?;
        for &index in &scheme.late {
            self.charge(1)?;
            let slot = self
                .inputs
                .regions
                .get(&(context.owner, u32::from(index)))
                .copied()
                .ok_or("source callable late slot is not declared by this original")?;
            self.rows.region_slots.push(slot);
        }
        let late = g::Range {
            start,
            end: u32::try_from(self.rows.region_slots.len())
                .map_err(|_| "source function slot table overflow")?,
        };
        Ok(s::RegionScheme {
            total: scheme.total,
            captures: scheme.captures,
            early: g::Range { start, end: start },
            late,
            bounds: self.region_bounds(&scheme.bounds, context)?,
        })
    }
}

fn declared_captures(facts: &Declarations, definition: DefId, modern: bool) -> Option<u32> {
    let owner = facts.associated.get(facts.associated.owner(definition)?)?;
    match owner.kind {
        OwnerKind::Impl { trait_type, .. }
            if owner.self_definition == definition && (modern || trait_type.is_none()) => {}
        OwnerKind::Trait {
            definition: actual, ..
        } if modern && actual == definition => {}
        _ => return None,
    }
    Some(facts.associated.schema(definition)?.captures)
}

/// Which const-trait roles may own a conditional required member: `From`
/// from version nine, the callable roles from thirteen and Div from fourteen.
#[derive(Clone, Copy)]
struct RequiredRoles {
    conversions: bool,
    callables: bool,
    declaration_policies: bool,
}
fn written_constness(
    report: &Report,
    facts: &Declarations,
    function: &crate::sema::model::Function,
    roles: RequiredRoles,
) -> Result<Constness, &'static str> {
    let ast = &report.package.unit(function.node.file).configured.ast;
    let crate::syntax::ast::Kind::Item(index) = ast.node(function.node.node).kind else {
        return Err("source callable effects require their actual parsed function declaration");
    };
    let crate::syntax::item::ItemKind::Function { flags, .. } = ast
        .items
        .get(index as usize)
        .ok_or("source callable declaration item is missing")?
        .kind
    else {
        return Err("source callable effect owner is not a function");
    };
    if roles.conversions && required_trait_member(report, facts, function, roles)? {
        if flags.constant || facts.is_const_function(function.definition) {
            return Err(
                "required conversion member changed its written/effective const eligibility",
            );
        }
        return Ok(Constness::Conditional);
    }
    let effective = facts.is_const_function(function.definition);
    if effective
        && !flags.constant
        && roles.declaration_policies
        && effective_impl_constness(report, facts, function.definition)?
    {
        return Ok(Constness::Const);
    }
    if flags.constant != effective {
        return Err("source effective const eligibility requires its actual marker transport");
    }
    Ok(if flags.constant {
        Constness::Const
    } else {
        Constness::Ordinary
    })
}

/// Only the authenticated const impl supplies a method's unwritten effect.
fn effective_impl_constness(
    report: &Report,
    facts: &Declarations,
    definition: DefId,
) -> Result<bool, &'static str> {
    use crate::sema::associated::ImplPolarity;
    let Some(owner) = facts
        .associated
        .owner(definition)
        .and_then(|id| facts.associated.get(id))
    else {
        return Ok(false);
    };
    let OwnerKind::Impl {
        constant: true,
        polarity,
        ..
    } = owner.kind
    else {
        return Ok(false);
    };
    if polarity == ImplPolarity::Reservation {
        return Ok(true);
    }
    if !polarity.is_positive() {
        return Ok(false);
    }
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("source effective const method lost its actual language catalog")?;
    catalog.require_provider_report(report)?;
    let [subtract, divide] = s::Profile::CONST_BINARY_ROLES.map(|role| catalog.language(role));
    let [deref, deref_mut] = s::Profile::CONST_DEREF_ROLES.map(|role| catalog.language(role));
    let roles = [subtract, divide, deref, deref_mut];
    if roles.iter().all(Option::is_none) {
        return Ok(false);
    }
    let implementations = facts
        .generics
        .implementations
        .as_deref()
        .ok_or("source effective const method lost its implementation registry")?;
    if !implementations.is_validated(owner.self_definition) {
        return Err("source effective const method requires its validated implementation");
    }
    let implementation =
        implementations.checked_entry(implementations.entry_key(owner.self_definition)?)?;
    let Some((application, Constness::Const)) = implementation.predicate.bound.trait_parts() else {
        return Err("source effective const method changed its actual impl effect");
    };
    Ok(implementation.source == owner.source
        && facts
            .types
            .trait_application(application)
            .is_some_and(|application| {
                roles
                    .into_iter()
                    .flatten()
                    .any(|role| role.definition.0 == application.item.0)
            }))
}

/// A required const-trait declaration is conditional, never a const body or an
/// executable impl. The indexed actual parent supplies the declaration fact;
/// a member's name and signature cannot create the language role.
fn required_trait_member(
    report: &Report,
    facts: &Declarations,
    function: &crate::sema::model::Function,
    roles: RequiredRoles,
) -> Result<bool, &'static str> {
    let Some(owner_id) = facts.associated.owner(function.definition) else {
        return Ok(false);
    };
    let owner = facts
        .associated
        .get(owner_id)
        .ok_or("conversion member lost its actual associated owner")?;
    let OwnerKind::Trait {
        definition,
        constant: true,
        ..
    } = owner.kind
    else {
        return Ok(false);
    };
    let catalog = facts
        .source_bindings
        .as_deref()
        .ok_or("conversion member lost its actual language catalog")?;
    catalog.require_provider_report(report)?;
    let mut selected = if roles.declaration_policies {
        // The exact language role owns this member's unwritten effect.
        s::Profile::CONST_BINARY_ROLES
            .into_iter()
            .chain(s::Profile::CONST_DEREF_ROLES)
            .find_map(|role| {
                catalog
                    .language(role)
                    .filter(|role| role.definition == definition)
            })
    } else {
        None
    };
    if selected.is_some() && facts.nonconst_trait_member(function.definition)? {
        return Err("source non-const operator method requires its actual policy transport");
    }
    if roles.declaration_policies
        && let Some((parent, [_, _, subtract])) =
            crate::sema::compiler_items::source_range_headers(facts)
        && subtract.definition == function.definition
    {
        if facts.nonconst_trait_member(function.definition)? {
            return Err("source RangeSub requires its actual conditional method effect");
        }
        selected = Some(parent);
    }
    if roles.callables {
        for role in [LangItem::FnOnce, LangItem::FnMut, LangItem::Fn] {
            if let Some(found) = catalog.header_role(role)?
                && found.definition == definition
            {
                selected = Some(found);
            }
        }
    }
    let drop = roles
        .declaration_policies
        .then(|| catalog.language(LangItem::Drop))
        .flatten()
        .filter(|role| role.definition == definition);
    if drop.is_some() {
        crate::sema::compiler_items::require_source_drop_contract(report, facts)
            .map_err(|_| "conditional Drop member lost its actual checked trait contract")?;
        if function.body.is_some() {
            return Err("provided Drop members require their original conditional body transport");
        }
        selected = drop;
    }
    let role = match selected {
        Some(role) => role,
        None => catalog
            .header_role(LangItem::From)?
            .ok_or("conditional member requires its actual From declaration")?,
    };
    if role.definition != definition {
        return Err("conditional member belongs to another actual trait");
    }
    if function.body.is_some() {
        return Err("conversion trait defaults require their actual body transport");
    }
    if role.parent.is_some()
        || owner.source != Some(role.origin)
        || facts.associated.owner(role.definition) != Some(owner_id)
    {
        return Err("conversion member changed its actual source trait owner");
    }
    let actual = report
        .resolution
        .definitions
        .get(function.definition.0 as usize)
        .ok_or("conversion member lost its actual declaration")?;
    if actual.kind != crate::resolve::model::DefKind::Function || actual.node != Some(function.node)
    {
        return Err("conversion member changed its actual source function header");
    }
    Ok(true)
}

#[cfg(test)]
mod effect_storage {
    #[test]
    fn callable_capability_occupies_existing_cold_index_padding() {
        use super::*;
        struct Previous {
            bodies: HashMap<DefId, BodyRef>,
            contexts: Vec<Context>,
            originals: Vec<Option<(s::Original, s::BodyInputs)>>,
            headers: HashMap<crate::types::ItemId, u32>,
            exact: bool,
        }
        let previous = Previous {
            bodies: HashMap::default(),
            contexts: Vec::new(),
            originals: Vec::new(),
            headers: HashMap::default(),
            exact: false,
        };
        assert!(
            previous.bodies.is_empty()
                && previous.contexts.is_empty()
                && previous.originals.is_empty()
                && previous.headers.is_empty()
                && !previous.exact
        );
        assert_eq!(size_of::<Previous>(), size_of::<Index>());
        println!(
            "IndexPrevious={} IndexCurrent={}",
            size_of::<Previous>(),
            size_of::<Index>()
        );
    }
}

#[cfg(test)]
#[path = "../../../../tests/invariants/required_exports.rs"]
mod conditional_tests;

/// Version thirteen writes declared target features only for a free or
/// inherent `unsafe fn` with the Rust ABI, in this order; the first failure
/// wins. Its canonical signature is always unsafe, so the declared safety is
/// read from the source row.
fn target_feature_form(
    facts: &Declarations,
    function: &crate::sema::model::Function,
) -> Result<(), &'static str> {
    if facts
        .associated
        .owner(function.definition)
        .and_then(|owner| facts.associated.get(owner))
        .is_some_and(|owner| {
            !matches!(
                owner.kind,
                OwnerKind::Impl {
                    trait_type: None,
                    ..
                }
            )
        })
    {
        return Err(
            "source trait member target features require their trait implementation transport",
        );
    }
    if !matches!(
        facts.types.kind(function.signature),
        Some(Kind::Function { convention, .. })
            if convention.abi == crate::types::callable::Abi::Rust
    ) || facts.is_const_function(function.definition)
    {
        return Err("source target-feature function form requires its wider profile");
    }
    if !crate::sema::target_features::declared_unsafe(facts, function.definition) {
        return Err(
            "source safe target-feature function export requires its contextual call policy",
        );
    }
    if crate::sema::target_features::declared_names(facts, function.definition)
        .is_none_or(|mut names| names.next().is_none())
    {
        return Err("unknown source target feature");
    }
    Ok(())
}
