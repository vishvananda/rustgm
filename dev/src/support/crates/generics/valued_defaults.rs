//! Closed constant defaults keep the actual parameter kind and record fields.
use super::{
    ConstFormalType, Declaration, DefinitionKind, HeaderIndex, HeaderRef, NominalField,
    NominalForm, NominalHeader, ParameterKind, SlotRef, const_formal_type, const_parameter_type,
};
use crate::support::crates::{ScalarValue, generics::Work, owner_catalog::DeclarationRef};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConstDefault {
    pub slot: SlotRef,
    pub value: ConstDefaultValue,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConstDefaultValue {
    Scalar(u128),
    Record(Vec<u128>),
}

pub(crate) fn find(values: &[ConstDefault], slot: SlotRef) -> Option<&ConstDefaultValue> {
    let index = values
        .binary_search_by_key(&slot.0, |row| row.slot.0)
        .ok()?;
    Some(&values[index].value)
}

pub(crate) fn fields<'a>(
    header: &NominalHeader,
    declarations: &[Declaration],
    fields: &'a [NominalField],
) -> Result<&'a [NominalField], &'static str> {
    let declaration = declarations
        .get(header.declaration.0 as usize)
        .ok_or("source const default record is unknown")?;
    if declaration.definition.kind != DefinitionKind::Struct
        || header.layout.form == NominalForm::Union
    {
        return Err("source const default requires its actual record fields");
    }
    header.fields.slice(fields)
}

pub(crate) fn validate(headers: &HeaderIndex<'_>, work: &mut Work) -> Result<(), &'static str> {
    let rows = headers.rows();
    let mut prior = None;
    for row in &rows.cold().const_defaults {
        work.charge(1)?;
        if prior
            .replace(row.slot.0)
            .is_some_and(|last| last >= row.slot.0)
        {
            return Err("source const default slot is duplicate or out of order");
        }
        let slot = rows
            .slots
            .get(row.slot.0 as usize)
            .ok_or("source const default slot is unknown")?;
        let ParameterKind::Const { ty } = slot.kind else {
            return Err("source const default names a different kind of formal");
        };
        if !matches!(
            headers.header(slot.owner, work)?,
            Some(HeaderRef::Trait(_) | HeaderRef::Nominal(_))
        ) {
            return Err("source const default has an invalid owner");
        }
        match (const_formal_type(&rows.types, ty)?, &row.value) {
            (ConstFormalType::Scalar(ty), ConstDefaultValue::Scalar(bits)) => {
                ScalarValue::new(ty, *bits)
                    .map_err(|_| "source const default has invalid scalar bits")?;
            }
            (ConstFormalType::Nominal(owner), ConstDefaultValue::Record(values)) => {
                record(headers, owner, values, work)?;
            }
            _ => return Err("source const default changed its actual formal type"),
        }
    }
    Ok(())
}

fn record(
    headers: &HeaderIndex<'_>,
    owner: DeclarationRef,
    values: &[u128],
    work: &mut Work,
) -> Result<(), &'static str> {
    let Some(HeaderRef::Nominal(index)) = headers.header(owner, work)? else {
        return Err("source const default lost its exact record declaration");
    };
    let rows = headers.rows();
    let fields = fields(
        &rows.nominals[index as usize],
        &rows.declarations,
        &rows.fields,
    )?;
    if fields.len() != values.len() {
        return Err("source const default changed its record field count");
    }
    work.charge(values.len())?;
    for (field, bits) in fields.iter().zip(values) {
        let ty = const_parameter_type(&rows.types, field.ty)?;
        ScalarValue::new(ty, *bits).map_err(|_| "source const default field has invalid bits")?;
    }
    Ok(())
}
