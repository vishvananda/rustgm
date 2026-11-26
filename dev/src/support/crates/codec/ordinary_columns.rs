//! Draft ordinary-source columns. Public descriptor admission stays disconnected.
mod allocation_columns;
mod availability_wire;
mod cfg_columns;
mod child_closure_wire;
mod compiler_wire;
#[cfg(test)]
#[path = "../../../../tests/invariants/term_codec.rs"]
mod const_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/evaluated_constant_rows.rs"]
mod constant_tests;
mod contract_columns;
mod enums_wire;
mod exemption_wire;
mod existential_wire;
#[cfg(test)]
#[path = "../../../../tests/invariants/enabled_headers.rs"]
mod feature_tests;
mod free_constants_wire;
mod global_wire;
#[path = "ordinary_columns/annotation_rows.rs"]
mod header_facts;
mod instruction_columns;
mod invoked_rows;
mod ir_nodes;
mod namespace_rows;
#[path = "ordinary_columns/record_wire.rs"]
mod nominal_rows;
mod notice_wire;
mod operation_columns;
mod owner_rows;
#[path = "ordinary_columns/supplied_extensions.rs"]
mod provided_extensions;
mod recipe_columns;
#[cfg(test)]
#[path = "../../../../tests/invariants/register_wire.rs"]
mod register_tests;
mod relation_columns;
mod results_wire;
mod sparse_defaults_wire;
#[cfg(test)]
#[path = "../../../../tests/invariants/supplied_packets.rs"]
mod supplied_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/packet_roundtrip.rs"]
mod tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/vector_headers.rs"]
mod vector_tests;
mod verbatim_exports;
use super::{Reader, Writer};
use crate::support::{
    crates::{
        self,
        generics::{self as g, source as s},
        owner_catalog::{BodyRef, DeclarationRef},
    },
    edition::Edition,
};

pub(super) fn encode(out: &mut Writer, rows: s::ValidatedView<'_>) -> Result<(), &'static str> {
    if !matches!(
        rows.descriptor().profile(),
        s::Profile::Source2024Functions | s::Profile::Source2024FunctionContracts
    ) {
        return Err("legacy source encoder requires its actual v0/v1 descriptor");
    }
    encode_profile(out, rows, false)
}
pub(super) fn encode_provided(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    encode_profile(out, rows, true)
}
pub(super) fn encode_contracts(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    encode(out, rows)?;
    header_facts::encode(
        out,
        rows.header_attributes()
            .ok_or("source contract columns are missing")?,
    )
}
pub(super) fn encode_module_contracts(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    let modules = rows.module_contracts()?;
    encode_profile(out, rows, false)?;
    header_facts::encode(
        out,
        rows.header_attributes()
            .ok_or("version-two advisory columns are missing")?,
    )?;
    namespace_rows::encode(out, modules.rows())
}
pub(super) fn encode_module_availability(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    encode_module_contracts(out, rows)?;
    availability_wire::encode(
        out,
        rows.availability()?,
        rows.descriptor().profile().module_compatibility(),
    )?;
    if rows.descriptor().profile().definitions() {
        verbatim_exports::encode(out, &rows.exact_callables)?;
    }
    if rows.descriptor().profile().nominals() {
        nominal_rows::encode(out, rows)?;
    }
    if rows.descriptor().profile().body_nodes() {
        results_wire::encode(out, rows)?;
        enums_wire::encode(out, rows)?;
        free_constants_wire::encode(out, rows)?;
    }
    let availability = rows.availability()?;
    let const_bodies = availability.const_bodies.as_deref();
    let profile = rows.descriptor().profile();
    let aliases = rows.type_aliases();
    let trait_aliases = rows.trait_aliases();
    let alias_normalizations = &rows.cold().alias_normalizations;
    let statics = rows.statics();
    if profile.extended_columns() {
        out.u8(u8::from(const_bodies.is_some())
            | (u8::from(availability.deprecations.is_some()) << 1)
            | (u8::from(!aliases.is_empty()) << 2)
            | (u8::from(statics.is_some()) << 3)
            | (u8::from(!trait_aliases.is_empty()) << 4)
            | (u8::from(!alias_normalizations.is_empty()) << 5)
            | (u8::from(!rows.cold().nominal_region_bounds.is_empty()) << 6)
            | (u8::from(!rows.cold().const_defaults.is_empty()) << 7))?;
    }
    exemption_wire::encode(out, const_bodies)?;
    if let Some(deprecations) = &availability.deprecations {
        notice_wire::encode(out, deprecations)?;
    }
    if !aliases.is_empty() {
        write(out, aliases, |out, row| {
            id(out, row.declaration.0)?;
            id(out, row.target.0)
        })?;
    }
    if let Some(statics) = statics {
        global_wire::encode(out, statics)?;
    }
    if !trait_aliases.is_empty() {
        write(out, trait_aliases, |out, row| {
            id(out, row.declaration.0)?;
            id(out, row.self_definition.0)?;
            id(out, row.self_type.0)?;
            range(out, row.inputs)
        })?;
    }
    if !alias_normalizations.is_empty() {
        write(out, alias_normalizations, |out, row| {
            id(out, row.declaration.0)?;
            id(out, row.target.0)
        })?;
    }
    if !rows.cold().nominal_region_bounds.is_empty() {
        write(out, &rows.cold().nominal_region_bounds, |out, row| {
            id(out, row.declaration.0)?;
            range(out, row.bounds)
        })?;
    }
    if !rows.cold().const_defaults.is_empty() {
        sparse_defaults_wire::encode(out, &rows.cold().const_defaults)?;
    }
    if profile.extended_columns() {
        if profile.body_nodes() {
            compiler_wire::encode_operations(out, rows)?;
            let opaques = &rows.cold().opaques;
            if !opaques.applications.is_empty() {
                compiler_wire::encode_opaque_declarations(out, &opaques.declarations)?;
            } else if !opaques.declarations.is_empty() {
                return Err("source opaque declaration lacks its canonical application recipe");
            }
            encode_body_trailer(out, rows, profile)?;
        } else {
            compiler_wire::encode(out, rows)?;
        }
    }
    Ok(())
}
fn encode_caller_sites(out: &mut Writer, sites: &s::CallerSiteColumns) -> Result<(), &'static str> {
    if sites.files.is_empty() || sites.rows.is_empty() {
        return Err("source caller site trailer requires actual entries");
    }
    write(out, &sites.files, |out, file| out.string(file))?;
    write(out, &sites.rows, |out, row| {
        id(out, row.body.0)?;
        id(out, row.value)?;
        id(out, row.file)?;
        id(out, row.line)?;
        id(out, row.column)
    })
}
fn decode_caller_sites(
    read: &mut Input<'_, '_>,
) -> Result<Box<s::CallerSiteColumns>, &'static str> {
    read.reserve(size_of::<s::CallerSiteColumns>())?;
    let files = read.list(crates::MAX_COMPONENTS, 4, operation_columns::text_read)?;
    let rows = read.list(crates::MAX_COMPONENTS, 20, |read| {
        Ok(s::CallerSiteRow {
            body: BodyRef(read.id()?),
            value: read.id()?,
            file: read.id()?,
            line: read.id()?,
            column: read.id()?,
        })
    })?;
    if files.is_empty() || rows.is_empty() {
        return Err("source caller site trailer requires actual entries");
    }
    Ok(Box::new(s::CallerSiteColumns { files, rows }))
}
pub(super) struct AvailabilityPacket {
    pub rows: s::SourceRows,
    pub attributes: s::HeaderAttributeColumns,
    pub modules: s::ModuleContractRows,
    pub availability: s::AvailabilityRows,
}
pub(super) fn decode_module_availability(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleAvailability)
}
pub(super) fn decode_module_feature_predicates(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleFeaturePredicates)
}
pub(super) fn decode_module_definitions(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleDefinitions)
}
pub(super) fn decode_module_compatibility(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleCompatibility)
}
pub(super) fn decode_module_callables(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleCallables)
}
pub(super) fn decode_compiler_operations(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleCompilerOperations)
}
pub(super) fn decode_module_nominals(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleNominals)
}
pub(super) fn decode_module_branches(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleBranches)
}
pub(super) fn decode_module_body_operations(
    reader: &mut Reader<'_>,
    base: usize,
    profile: s::Profile,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, profile)
}
pub(super) fn decode_module_bodies(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleBodies)
}
pub(super) fn decode_module_conversions(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleConversions)
}
pub(super) fn decode_module_safety(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<AvailabilityPacket, &'static str> {
    decode_availability_profile(reader, base, s::Profile::Source2024ModuleSafety)
}
fn decode_availability_profile(
    reader: &mut Reader<'_>,
    base: usize,
    profile: s::Profile,
) -> Result<AvailabilityPacket, &'static str> {
    let (mut rows, bytes) = decode_profile_with(reader, base, profile)?;
    let mut read = Input {
        reader,
        bytes,
        base,
        profile,
    };
    let attributes = header_facts::decode(&mut read)?;
    let modules = namespace_rows::decode(&mut read)?;
    let mut availability = availability_wire::decode(&mut read, profile.module_compatibility())?;
    if profile.definitions() {
        rows.exact_callables = verbatim_exports::decode(&mut read)?;
    }
    if profile.nominals() {
        rows.nominals = read.list(crates::MAX_COMPONENTS, 43, nominal_rows::nominal_read)?;
        rows.fields = read.list(crates::MAX_COMPONENTS, 21, nominal_rows::field_read)?;
    }
    if profile.body_nodes() {
        results_wire::decode(&mut read, &mut rows)?;
        enums_wire::decode(&mut read, &mut rows)?;
        free_constants_wire::decode(&mut read, &mut rows)?;
    }
    decode_availability_trailers(&mut read, &mut rows, &mut availability)?;
    if profile.extended_columns() {
        if profile.body_nodes() {
            compiler_wire::decode_operations(&mut read, &mut rows)?;
            if !rows.cold().opaques.applications.is_empty() {
                compiler_wire::decode_opaque_declarations(&mut read, &mut rows)?;
            }
            decode_body_trailer(&mut read, &mut rows)?;
        } else {
            compiler_wire::decode(&mut read, &mut rows)?;
        }
    }
    Ok(AvailabilityPacket {
        rows,
        attributes,
        modules,
        availability,
    })
}

fn encode_body_trailer(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
    profile: s::Profile,
) -> Result<(), &'static str> {
    let caller_sites = rows.cold().caller_sites.as_deref();
    let closures = rows.closures().filter(|c| !c.headers.is_empty());
    let anonymous = rows.closures().filter(|c| !c.anonymous.is_empty());
    let specialization = rows.cold().specialization.as_deref();
    out.u8(u8::from(caller_sites.is_some())
        | (u8::from(closures.is_some()) << 1)
        | (u8::from(anonymous.is_some()) << 2)
        | (u8::from(specialization.is_some()) << 3))?;
    if let Some(sites) = caller_sites {
        encode_caller_sites(out, sites)?;
    }
    if let Some(closures) = closures {
        child_closure_wire::encode(out, closures)?;
    }
    if let Some(anonymous) = anonymous {
        child_closure_wire::encode_anonymous(out, anonymous)?;
    }
    if let Some(graph) = specialization {
        if !profile.declaration_policies() || graph.parents.is_empty() {
            return Err(
                "source specialization trailer requires checked parents and the extended profile",
            );
        }
        write(out, &graph.parents, |out, edge| {
            id(out, edge.child.0)?;
            id(out, edge.parent.0)?;
            range(out, edge.arguments)
        })?;
    }
    Ok(())
}

fn decode_body_trailer(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
) -> Result<(), &'static str> {
    let body_flags = read.reader.u8()?;
    let max_flags = if read.profile.declaration_policies() {
        15
    } else {
        7
    };
    if body_flags > max_flags {
        return Err("unknown v14 body extension trailer flag");
    }
    if body_flags & 1 != 0 {
        let sites = decode_caller_sites(read)?;
        read.cold(rows)?.caller_sites = Some(sites);
    }
    if body_flags & 2 != 0 {
        let types = rows
            .cold
            .as_mut()
            .and_then(|cold| cold.closures.as_mut())
            .map(|closures| std::mem::take(&mut closures.types))
            .unwrap_or_default();
        let closures = child_closure_wire::decode(read, types)?;
        read.cold(rows)?.closures = Some(closures);
    }
    if body_flags & 4 != 0 {
        let cold = read.cold(rows)?;
        if cold.closures.is_none() {
            read.reserve(size_of::<s::ClosureColumns>())?;
        }
        let closures = read
            .cold(rows)?
            .closures
            .get_or_insert_with(|| Box::new(s::ClosureColumns::default()));
        child_closure_wire::decode_anonymous(read, closures)?;
    }
    if body_flags & 8 != 0 {
        let parents = decode_specialization(read)?;
        read.cold(rows)?.specialization = Some(parents);
    }
    Ok(())
}

fn decode_specialization(
    read: &mut Input<'_, '_>,
) -> Result<Box<s::SpecializationColumns>, &'static str> {
    let parents = read.list(crates::MAX_COMPONENTS, 16, |read| {
        Ok(s::SpecializationParent {
            child: DeclarationRef(read.id()?),
            parent: DeclarationRef(read.id()?),
            arguments: read.range()?,
        })
    })?;
    if parents.is_empty() {
        return Err("source specialization trailer requires checked parents");
    }
    read.reserve(size_of::<s::SpecializationColumns>())?;
    Ok(Box::new(s::SpecializationColumns { parents }))
}

fn decode_availability_trailers(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
    availability: &mut s::AvailabilityRows,
) -> Result<(), &'static str> {
    if !read.profile.extended_columns() {
        availability.const_bodies = exemption_wire::decode(read)?;
        return Ok(());
    }
    let flags = read.reader.u8()?;
    if !read.profile.declaration_policies() && flags > 1 {
        return Err("invalid compiler-operation const-body trailer presence");
    }
    if flags & 1 != 0 {
        availability.const_bodies = Some(
            exemption_wire::decode(read)?
                .ok_or("compiler-operation const-body trailer is missing")?,
        );
    }
    if flags & 2 != 0 {
        availability.deprecations = Some(Box::new(notice_wire::decode(read)?));
    }
    if flags & 4 != 0 {
        let aliases = read.list(crates::MAX_COMPONENTS, 8, |read| {
            Ok(s::TypeAliasHeader {
                declaration: DeclarationRef(read.id()?),
                target: g::TypeRef(read.id()?),
            })
        })?;
        if aliases.is_empty() {
            return Err("type alias trailer must contain an actual alias");
        }
        read.cold(rows)?.type_aliases = aliases;
    }
    if flags & 8 != 0 {
        let statics = global_wire::decode(read)?;
        read.cold(rows)?.statics = Some(statics);
    }
    if flags & 16 != 0 {
        let aliases = read.list(crates::MAX_COMPONENTS, 20, |read| {
            Ok(s::TraitAliasHeader {
                declaration: DeclarationRef(read.id()?),
                self_definition: DeclarationRef(read.id()?),
                self_type: g::TypeRef(read.id()?),
                inputs: read.range()?,
            })
        })?;
        if aliases.is_empty() {
            return Err("trait alias trailer must contain an actual alias");
        }
        read.cold(rows)?.trait_aliases = aliases;
    }
    if flags & 32 != 0 {
        let normalizations = read.list(crates::MAX_COMPONENTS, 8, |read| {
            Ok(s::AliasNormalization {
                declaration: DeclarationRef(read.id()?),
                target: g::TypeRef(read.id()?),
            })
        })?;
        if normalizations.is_empty() {
            return Err("alias normalization trailer must contain an actual root");
        }
        read.cold(rows)?.alias_normalizations = normalizations;
    }
    if flags & 64 != 0 {
        let bounds = read.list(crates::MAX_COMPONENTS, 12, |read| {
            Ok(s::NominalRegionBounds {
                declaration: DeclarationRef(read.id()?),
                bounds: read.range()?,
            })
        })?;
        if bounds.is_empty() {
            return Err("nominal lifetime trailer requires written bounds");
        }
        read.cold(rows)?.nominal_region_bounds = bounds;
    }
    if flags & 128 != 0 {
        let defaults = sparse_defaults_wire::decode(read)?;
        read.cold(rows)?.const_defaults = defaults;
    }
    Ok(())
}
pub(super) fn encode_provided_contracts(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
) -> Result<(), &'static str> {
    let contracts = rows.provided_contracts()?;
    encode_profile(out, rows, true)?;
    header_facts::encode(
        out,
        rows.header_attributes()
            .ok_or("provided advisory columns are missing")?,
    )?;
    provided_extensions::encode(out, contracts.rows())
}
fn encode_profile(
    out: &mut Writer,
    rows: s::ValidatedView<'_>,
    provided: bool,
) -> Result<(), &'static str> {
    if rows.policy.edition != Edition::Rust2024 {
        return Err("source function columns require defining Rust2024 edition");
    }
    let regional = provided || rows.descriptor().profile().conversion_roles();
    if !regional && !rows.body_units.is_empty() {
        return Err("source function columns cannot encode numbering units");
    }
    id(out, rows.policy.provider.0)?;
    range(out, rows.policy.roles)?;
    write(out, &rows.declarations, owner_rows::declaration_write)?;
    write(out, rows.body_owners(), owner_rows::body_write)?;
    write(out, &rows.slots, owner_rows::slot_write)?;
    write(out, &rows.scopes, owner_rows::scope_write)?;
    write(out, &rows.binders, owner_rows::binder_write)?;
    recipe_columns::types_write(
        out,
        &rows.types,
        &recipe_columns::TypeColumns {
            ranges: rows.pattern_ranges(),
            lengths: rows.array_lengths(),
            dynamics: &rows.cold().dynamics,
            opaques: &rows.cold().opaques.applications,
            closures: rows.closures().map_or(&[][..], |c| c.types.as_slice()),
        },
    )?;
    write_copy(out, &rows.roots, recipe_columns::root_write)?;
    write(out, &rows.regions, recipe_columns::region_write)?;
    recipe_columns::arguments_write(out, &rows.arguments, rows.consts())?;
    write(out, &rows.clauses, recipe_columns::clause_write)?;
    write_copy(out, &rows.clause_indices, index_write)?;
    write_copy(out, &rows.region_bounds, recipe_columns::bound_write)?;
    write_copy(out, &rows.region_slots, owner_rows::slot_ref_write)?;
    write(
        out,
        &rows.trait_applications,
        recipe_columns::application_write,
    )?;
    write(
        out,
        &rows.function_applications,
        recipe_columns::function_write,
    )?;
    write(out, &rows.traits, contract_columns::trait_write)?;
    write(
        out,
        &rows.implementations,
        contract_columns::implementation_write,
    )?;
    contract_columns::functions_write(
        out,
        &rows.functions,
        &rows.cold().declared_intrinsics,
        &rows.cold().function_policies,
    )?;
    write(out, &rows.formals, contract_columns::formal_write)?;
    write_copy(out, &rows.members, owner_rows::declaration_ref_write)?;
    write(out, &rows.member_links, contract_columns::member_write)?;
    if rows.descriptor().profile().body_nodes() {
        contract_columns::language_roles_write(out, &rows.roles, rows.diagnostics())?;
    } else if !rows.diagnostics().is_empty() {
        return Err("source diagnostic identity requires a language role table");
    } else if rows.descriptor().profile().conversion_roles() {
        write(out, &rows.roles, contract_columns::conversion_role_write)?;
    } else {
        write(out, &rows.roles, contract_columns::role_write)?;
    }
    if rows.descriptor().profile().declaration_policies() {
        write(out, &rows.originals, operation_columns::body_policy_write)?;
    } else if rows.descriptor().profile().body_nodes() {
        write(out, &rows.originals, operation_columns::body_write)?;
    } else if rows.descriptor().profile().branches() {
        write(out, &rows.originals, operation_columns::branch_write)?;
    } else if rows.descriptor().profile().nominals() {
        write(out, &rows.originals, operation_columns::nominal_write)?;
    } else if rows.descriptor().profile().call_safety() {
        write(out, &rows.originals, operation_columns::safety_write)?;
    } else {
        write(out, &rows.originals, operation_columns::original_write)?;
    }
    write(out, &rows.body_inputs, relation_columns::body_write)?;
    if regional {
        write(out, &rows.body_units, allocation_columns::unit_write)?;
    }
    Ok(())
}

pub(super) fn decode(reader: &mut Reader<'_>, base: usize) -> Result<s::SourceRows, &'static str> {
    decode_profile(reader, base, false).map(|(rows, _)| rows)
}
pub(super) fn decode_provided(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<s::SourceRows, &'static str> {
    decode_profile(reader, base, true).map(|(rows, _)| rows)
}
pub(super) fn decode_provided_contracts(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<
    (
        s::SourceRows,
        s::HeaderAttributeColumns,
        s::ProvidedContractRows,
    ),
    &'static str,
> {
    let (rows, bytes) = decode_profile(reader, base, true)?;
    let mut read = Input {
        reader,
        bytes,
        base,
        profile: s::Profile::Source2024Provided,
    };
    let attributes = header_facts::decode(&mut read)?;
    let contracts = provided_extensions::decode(&mut read)?;
    Ok((rows, attributes, contracts))
}
pub(super) fn decode_contracts(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<(s::SourceRows, s::HeaderAttributeColumns), &'static str> {
    let (rows, bytes) = decode_profile(reader, base, false)?;
    let attributes = header_facts::decode(&mut Input {
        reader,
        bytes,
        base,
        profile: s::Profile::Source2024Functions,
    })?;
    Ok((rows, attributes))
}
pub(super) fn decode_module_contracts(
    reader: &mut Reader<'_>,
    base: usize,
) -> Result<
    (
        s::SourceRows,
        s::HeaderAttributeColumns,
        s::ModuleContractRows,
    ),
    &'static str,
> {
    let (rows, bytes) = decode_profile(reader, base, false)?;
    let mut read = Input {
        reader,
        bytes,
        base,
        profile: s::Profile::Source2024Functions,
    };
    let attributes = header_facts::decode(&mut read)?;
    let modules = namespace_rows::decode(&mut read)?;
    Ok((rows, attributes, modules))
}
fn decode_profile(
    reader: &mut Reader<'_>,
    base: usize,
    provided: bool,
) -> Result<(s::SourceRows, usize), &'static str> {
    let profile = if provided {
        s::Profile::Source2024Provided
    } else {
        s::Profile::Source2024Functions
    };
    decode_profile_with(reader, base, profile)
}
fn decode_profile_with(
    reader: &mut Reader<'_>,
    base: usize,
    profile: s::Profile,
) -> Result<(s::SourceRows, usize), &'static str> {
    let mut read = Input {
        reader,
        bytes: size_of::<s::SourceRows>(),
        base,
        profile,
    };
    let mut tables = recipe_columns::TypeTables::default();
    let mut diagnostics = Vec::new();
    let mut consts = Vec::new();
    let mut declared_intrinsics = Vec::new();
    let mut function_policies = Vec::new();
    let mut rows = s::SourceRows {
        associated_types: Vec::new(),
        cold: None,
        projections: Vec::new(),
        opaque_futures: Vec::new(),
        compiler_operations: Vec::new(),
        nominals: Vec::new(),
        fields: Vec::new(),
        exact_callables: Vec::new(),
        policy: s::Policy {
            edition: Edition::Rust2024,
            provider: s::ArtifactRef(read.id()?),
            roles: read.range()?,
        },
        declarations: read.list(crates::MAX_FUNCTIONS, 13, owner_rows::declaration_read)?,
        body_owners: read.list(crates::MAX_FUNCTIONS, 10, owner_rows::body_read)?,
        slots: read.list(crates::MAX_COMPONENTS, 14, owner_rows::slot_read)?,
        scopes: read.list(crates::MAX_COMPONENTS, 17, owner_rows::scope_read)?,
        binders: read.list(crates::MAX_COMPONENTS, 10, owner_rows::binder_read)?,
        types: read.list(crates::MAX_COMPONENTS, 2, |read| {
            recipe_columns::type_read(read, &mut tables)
        })?,
        roots: read.list(crates::MAX_COMPONENTS, 4, recipe_columns::root_read)?,
        regions: read.list(crates::MAX_COMPONENTS, 1, recipe_columns::region_read)?,
        arguments: read.list(crates::MAX_COMPONENTS, 5, |read| {
            recipe_columns::argument_read(read, &mut consts)
        })?,
        clauses: read.list(crates::MAX_COMPONENTS, 16, recipe_columns::clause_read)?,
        clause_indices: read.list(crates::MAX_COMPONENTS, 4, Input::id)?,
        region_bounds: read.list(crates::MAX_COMPONENTS, 8, recipe_columns::bound_read)?,
        region_slots: read.list(crates::MAX_COMPONENTS, 4, owner_rows::slot_ref_read)?,
        trait_applications: read.list(
            crates::MAX_COMPONENTS,
            20,
            recipe_columns::application_read,
        )?,
        function_applications: read.call_list(
            crates::MAX_COMPONENTS,
            16,
            recipe_columns::function_read,
        )?,
        traits: read.list(crates::MAX_FUNCTIONS, 66, contract_columns::trait_read)?,
        implementations: read.list(
            crates::MAX_FUNCTIONS,
            44,
            contract_columns::implementation_read,
        )?,
        functions: read.list(crates::MAX_FUNCTIONS, 98, |read| {
            contract_columns::function_read(read, &mut declared_intrinsics, &mut function_policies)
        })?,
        formals: read.list(crates::MAX_COMPONENTS, 5, contract_columns::formal_read)?,
        members: read.list(crates::MAX_COMPONENTS, 4, owner_rows::declaration_ref_read)?,
        member_links: read.list(crates::MAX_COMPONENTS, 2, contract_columns::member_read)?,
        roles: if read.profile.body_nodes() {
            contract_columns::language_roles_read(&mut read, &mut diagnostics)?
        } else {
            read.list(crates::MAX_COMPONENTS, 9, contract_columns::role_read)?
        },
        originals: read.list(crates::MAX_FUNCTIONS, 70, operation_columns::original_read)?,
        body_inputs: read.list(crates::MAX_FUNCTIONS, 30, relation_columns::body_read)?,
        body_units: if read.regional() {
            read.list(crates::MAX_FUNCTIONS, 13, allocation_columns::unit_read)?
        } else {
            Vec::new()
        },
    };
    attach_cold_tables(
        &mut read,
        &mut rows,
        tables,
        diagnostics,
        consts,
        declared_intrinsics,
        function_policies,
    )?;
    Ok((rows, read.bytes))
}

fn attach_cold_tables(
    read: &mut Input<'_, '_>,
    rows: &mut s::SourceRows,
    tables: recipe_columns::TypeTables,
    diagnostics: Vec<s::DiagnosticBinding>,
    consts: Vec<s::ConstTerm>,
    declared_intrinsics: Vec<s::DeclaredIntrinsic>,
    function_policies: Vec<s::FunctionPolicy>,
) -> Result<(), &'static str> {
    if tables.ranges.is_empty()
        && tables.lengths.is_empty()
        && tables.dynamics.is_empty()
        && tables.opaques.is_empty()
        && tables.closures.is_empty()
        && diagnostics.is_empty()
        && consts.is_empty()
        && declared_intrinsics.is_empty()
        && function_policies.is_empty()
    {
        return Ok(());
    }
    if !tables.closures.is_empty() {
        read.reserve(size_of::<s::ClosureColumns>())?;
    }
    let cold = read.cold(rows)?;
    cold.pattern_ranges = tables.ranges;
    cold.array_lengths = tables.lengths;
    cold.dynamics = tables.dynamics;
    cold.opaques.applications = tables.opaques;
    if !tables.closures.is_empty() {
        cold.closures = Some(Box::new(s::ClosureColumns {
            types: tables.closures,
            ..s::ClosureColumns::default()
        }));
    }
    cold.diagnostics = diagnostics;
    cold.consts = consts;
    cold.declared_intrinsics = declared_intrinsics;
    cold.function_policies = function_policies;
    Ok(())
}

struct Input<'a, 'b> {
    reader: &'b mut Reader<'a>,
    bytes: usize,
    base: usize,
    profile: s::Profile,
}
impl Input<'_, '_> {
    fn provided(&self) -> bool {
        matches!(
            self.profile,
            s::Profile::Source2024Provided
                | s::Profile::Source2024ProvidedContracts
                | s::Profile::Source2024ProvidedCurrent
        )
    }
    fn regional(&self) -> bool {
        self.provided() || self.profile.conversion_roles()
    }
    fn require_regional(&self, message: &'static str) -> Result<(), &'static str> {
        if self.regional() {
            Ok(())
        } else {
            Err(message)
        }
    }
    fn require_provided(&self, message: &'static str) -> Result<(), &'static str> {
        if self.provided() {
            Ok(())
        } else {
            Err(message)
        }
    }
    fn require_calls(&self, message: &'static str) -> Result<(), &'static str> {
        if self.provided() || self.profile.feature_predicates() {
            Ok(())
        } else {
            Err(message)
        }
    }
    fn call_list<T>(
        &mut self,
        limit: usize,
        minimum: usize,
        row: impl FnMut(&mut Self) -> Result<T, &'static str>,
    ) -> Result<Vec<T>, &'static str> {
        if self.provided() || self.profile.feature_predicates() {
            self.list(limit, minimum, row)
        } else {
            self.empty()?;
            Ok(Vec::new())
        }
    }
    fn regional_list<T>(
        &mut self,
        limit: usize,
        minimum: usize,
        row: impl FnMut(&mut Self) -> Result<T, &'static str>,
    ) -> Result<Vec<T>, &'static str> {
        if self.regional() {
            self.list(limit, minimum, row)
        } else {
            self.empty()?;
            Ok(Vec::new())
        }
    }
    fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.base.saturating_add(self.bytes) > crates::MAX_BYTES {
            return Err("source decoded columns exceed aggregate retained storage budget");
        }
        Ok(())
    }
    /// The cold groups of `rows`, allocated and paid on first write.
    fn cold<'r>(
        &mut self,
        rows: &'r mut s::SourceRows,
    ) -> Result<&'r mut s::ColdColumns, &'static str> {
        if rows.cold.is_none() {
            self.reserve(size_of::<s::ColdColumns>())?;
        }
        Ok(rows.cold_mut())
    }
    fn list<T>(
        &mut self,
        limit: usize,
        minimum: usize,
        mut row: impl FnMut(&mut Self) -> Result<T, &'static str>,
    ) -> Result<Vec<T>, &'static str> {
        let count = self.reader.count(limit, minimum)?;
        self.reader.atoms(count)?;
        self.reserve(count.saturating_mul(size_of::<T>()))?;
        let mut values = Vec::with_capacity(count);
        for _ in 0..count {
            values.push(row(self)?);
        }
        Ok(values)
    }
    fn id(&mut self) -> Result<u32, &'static str> {
        u32::try_from(self.reader.u32()?).map_err(|_| "source index exceeds u32")
    }
    fn boolean(&mut self) -> Result<bool, &'static str> {
        match self.reader.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid source boolean"),
        }
    }
    fn option(&mut self) -> Result<Option<u32>, &'static str> {
        match self.reader.u8()? {
            0 => Ok(None),
            1 => self.id().map(Some),
            _ => Err("invalid source optional index"),
        }
    }
    fn range(&mut self) -> Result<g::Range, &'static str> {
        Ok(g::Range {
            start: self.id()?,
            end: self.id()?,
        })
    }
    fn string(&mut self) -> Result<String, &'static str> {
        let text = self.reader.text(256)?;
        self.reserve(text.len())?;
        Ok(text.to_owned())
    }
    fn empty(&mut self) -> Result<(), &'static str> {
        self.reader.atoms(1)?;
        if self.id()? != 0 {
            return Err("source function profile contains an unsupported nonempty column");
        }
        Ok(())
    }
}
fn write<T>(
    out: &mut Writer,
    rows: &[T],
    row: fn(&mut Writer, &T) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    out.u32(rows.len())?;
    for value in rows {
        row(out, value)?;
    }
    Ok(())
}
fn id(out: &mut Writer, value: u32) -> Result<(), &'static str> {
    out.u32(value as usize)
}
fn write_copy<T: Copy>(
    out: &mut Writer,
    rows: &[T],
    row: fn(&mut Writer, T) -> Result<(), &'static str>,
) -> Result<(), &'static str> {
    out.u32(rows.len())?;
    for &value in rows {
        row(out, value)?;
    }
    Ok(())
}
fn index_write(out: &mut Writer, value: u32) -> Result<(), &'static str> {
    id(out, value)
}
fn option(out: &mut Writer, value: Option<u32>) -> Result<(), &'static str> {
    out.u8(u8::from(value.is_some()))?;
    if let Some(value) = value {
        id(out, value)?;
    }
    Ok(())
}
fn range(out: &mut Writer, value: g::Range) -> Result<(), &'static str> {
    id(out, value.start)?;
    id(out, value.end)
}
fn address_write(out: &mut Writer, value: s::DeclarationAddress) -> Result<(), &'static str> {
    id(out, value.artifact.0)?;
    id(out, value.declaration.0)
}
fn address_read(read: &mut Input<'_, '_>) -> Result<s::DeclarationAddress, &'static str> {
    Ok(s::DeclarationAddress {
        artifact: s::ArtifactRef(read.id()?),
        declaration: DeclarationRef(read.id()?),
    })
}
