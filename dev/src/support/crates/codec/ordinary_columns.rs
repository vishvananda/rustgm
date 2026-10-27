//! Draft ordinary-source columns. Public descriptor admission stays disconnected.
mod allocation_columns;
mod cfg_columns;
mod contract_columns;
#[path = "ordinary_columns/annotation_rows.rs"]
mod header_facts;
mod invoked_rows;
mod namespace_rows;
mod operation_columns;
mod owner_rows;
mod provided_extensions;
mod recipe_columns;
mod relation_columns;
#[cfg(test)]
#[path = "../../../../tests/invariants/supplied_packets.rs"]
mod supplied_tests;
#[cfg(test)]
#[path = "../../../../tests/invariants/packet_roundtrip.rs"]
mod tests;
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
    if !provided && !rows.body_units.is_empty() {
        return Err("source function columns cannot encode numbering units");
    }
    id(out, rows.policy.provider.0)?;
    range(out, rows.policy.roles)?;
    write(out, &rows.declarations, owner_rows::declaration_write)?;
    write(out, rows.body_owners(), owner_rows::body_write)?;
    write(out, &rows.slots, owner_rows::slot_write)?;
    write(out, &rows.scopes, owner_rows::scope_write)?;
    write(out, &rows.binders, owner_rows::binder_write)?;
    write(out, &rows.types, recipe_columns::type_write)?;
    write_copy(out, &rows.roots, recipe_columns::root_write)?;
    write(out, &rows.regions, recipe_columns::region_write)?;
    write_copy(out, &rows.arguments, recipe_columns::argument_write)?;
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
    write(out, &rows.functions, contract_columns::function_write)?;
    write(out, &rows.formals, contract_columns::formal_write)?;
    write_copy(out, &rows.members, owner_rows::declaration_ref_write)?;
    write(out, &rows.member_links, contract_columns::member_write)?;
    write(out, &rows.roles, contract_columns::role_write)?;
    write(out, &rows.originals, operation_columns::original_write)?;
    write(out, &rows.body_inputs, relation_columns::body_write)?;
    if provided {
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
        provided: true,
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
        provided: false,
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
        provided: false,
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
    let mut read = Input {
        reader,
        bytes: size_of::<s::SourceRows>(),
        base,
        provided,
    };
    let rows = s::SourceRows {
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
        types: read.list(crates::MAX_COMPONENTS, 2, recipe_columns::type_read)?,
        roots: read.list(crates::MAX_COMPONENTS, 4, recipe_columns::root_read)?,
        regions: read.list(crates::MAX_COMPONENTS, 1, recipe_columns::region_read)?,
        arguments: read.list(crates::MAX_COMPONENTS, 5, recipe_columns::argument_read)?,
        clauses: read.list(crates::MAX_COMPONENTS, 16, recipe_columns::clause_read)?,
        clause_indices: read.list(crates::MAX_COMPONENTS, 4, Input::id)?,
        region_bounds: read.list(crates::MAX_COMPONENTS, 8, recipe_columns::bound_read)?,
        region_slots: read.list(crates::MAX_COMPONENTS, 4, owner_rows::slot_ref_read)?,
        trait_applications: read.list(
            crates::MAX_COMPONENTS,
            20,
            recipe_columns::application_read,
        )?,
        function_applications: read.provided_list(
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
        functions: read.list(crates::MAX_FUNCTIONS, 98, contract_columns::function_read)?,
        formals: read.list(crates::MAX_COMPONENTS, 5, contract_columns::formal_read)?,
        members: read.list(crates::MAX_COMPONENTS, 4, owner_rows::declaration_ref_read)?,
        member_links: read.list(crates::MAX_COMPONENTS, 2, contract_columns::member_read)?,
        roles: read.list(crates::MAX_COMPONENTS, 9, contract_columns::role_read)?,
        originals: read.list(crates::MAX_FUNCTIONS, 70, operation_columns::original_read)?,
        body_inputs: read.list(crates::MAX_FUNCTIONS, 30, relation_columns::body_read)?,
        body_units: if provided {
            read.list(crates::MAX_FUNCTIONS, 13, allocation_columns::unit_read)?
        } else {
            Vec::new()
        },
    };
    Ok((rows, read.bytes))
}

struct Input<'a, 'b> {
    reader: &'b mut Reader<'a>,
    bytes: usize,
    base: usize,
    provided: bool,
}
impl Input<'_, '_> {
    fn require_provided(&self, message: &'static str) -> Result<(), &'static str> {
        if self.provided { Ok(()) } else { Err(message) }
    }
    fn provided_list<T>(
        &mut self,
        limit: usize,
        minimum: usize,
        row: impl FnMut(&mut Self) -> Result<T, &'static str>,
    ) -> Result<Vec<T>, &'static str> {
        if self.provided {
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
