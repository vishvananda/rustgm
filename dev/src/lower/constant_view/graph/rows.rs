//! Row vocabulary uses display IDs and exact scalar bits, never arena encodings.
use super::{Backing, Graph};
use crate::{
    literal::value::FloatBits,
    sema::{
        model::Scalar,
        values::{Constant, Metadata, Origin, View, WritePermission},
    },
    types::{Kind, TypeId},
};
use std::fmt::Write;
impl Graph<'_> {
    pub(super) fn value_row(
        &mut self,
        id: usize,
        ty: TypeId,
        value: Constant,
    ) -> Result<String, &'static str> {
        let mut row = format!("value v{id} type={} ", ty.0);
        match self.reader.view(value)? {
            View::Scalar(scalar) => row.push_str(&scalar_row(scalar)),
            View::Image { size } => self.image_row(&mut row, ty, value, size)?,
            View::Function(instance) => row.push_str(&callable_row(self.facts, instance)?),
            View::Reference(location) => {
                let allocation = self.location(location)?;
                let write = match value
                    .write_permission()
                    .ok_or("pointer value lacks write permission")?
                {
                    WritePermission::Writable => "writable",
                    WritePermission::SharedDerived => "shared-derived",
                };
                write!(row, "pointer allocation=a{allocation} write={write}")
                    .map_err(|_| "constant view formatting failed")?;
            }
            View::Address { bits } => write!(row, "address bits={bits:016x}")
                .map_err(|_| "constant view formatting failed")?,
            View::Text(_) | View::Bytes(_) | View::BytePointer { .. } => {
                self.literal_row(&mut row, ty, value)?;
            }
            View::ByteRange { length, .. } => self.byte_range_row(&mut row, ty, value, length)?,
            View::Repeat {
                value: element,
                length,
            } => self.repeat_row(&mut row, ty, value, element, length)?,
            View::Product { length, variant } => {
                self.product_row(&mut row, ty, value, length, variant)?;
            }
        }
        match value.metadata() {
            None => row.push_str(" metadata=none"),
            Some(Metadata::Length(length)) => write!(row, " metadata=length:{length}")
                .map_err(|_| "constant view formatting failed")?,
            Some(Metadata::Object { concrete, target }) => {
                write!(row, " metadata=object:{}:{}", concrete.0, target.0)
                    .map_err(|_| "constant view formatting failed")?;
            }
        }
        Ok(row)
    }
    fn byte_range_row(
        &mut self,
        row: &mut String,
        ty: TypeId,
        value: Constant,
        length: u64,
    ) -> Result<(), &'static str> {
        let bytes = self.reader.stored_bytes(ty, value)?;
        if bytes.len() as u64 != length {
            return Err("constant byte range differs from its recipe length");
        }
        self.charge(bytes.len())?;
        self.storage(bytes.len().saturating_mul(2).saturating_add(64))?;
        write!(row, "byte-range length={length} data=")
            .map_err(|_| "constant view formatting failed")?;
        for byte in bytes {
            write!(row, "{byte:02x}").map_err(|_| "constant view formatting failed")?;
        }
        Ok(())
    }
    fn literal_row(
        &mut self,
        row: &mut String,
        ty: TypeId,
        value: Constant,
    ) -> Result<(), &'static str> {
        let (text, kind) = match self.reader.view(value)? {
            View::Text(text) => (text, "text"),
            View::Bytes(text) => (text, "bytes"),
            View::BytePointer { text } => (text, "byte-pointer"),
            _ => return Err("constant literal row lacks bytes"),
        };
        let stored = match self.facts.types.kind(ty) {
            Some(Kind::Array { element, length }) => {
                if kind != "bytes"
                    || element != TypeId::primitive(crate::types::Primitive::U8)
                    || usize::try_from(length).ok() != Some(self.reader.bytes(text)?.len())
                {
                    return Err("constant byte image differs from its stored type");
                }
                true
            }
            Some(Kind::Primitive(crate::types::Primitive::Str)) if kind == "text" => true,
            Some(Kind::Reference { .. } | Kind::Pointer { .. }) => false,
            _ => return Err("constant literal lacks a stored byte or address type"),
        };
        if stored {
            write!(row, "{kind}-inline data=").map_err(|_| "constant view formatting failed")?;
            self.hex(row, text)?;
        } else {
            let allocation = self.allocation(
                Backing::Literal(text),
                TypeId::primitive(crate::types::Primitive::U8),
                None,
            )?;
            write!(
                row,
                "{kind} allocation=a{allocation} length={}",
                self.reader.bytes(text)?.len()
            )
            .map_err(|_| "constant view formatting failed")?;
        }
        Ok(())
    }
    fn hex(&mut self, row: &mut String, text: crate::sema::text::Id) -> Result<(), &'static str> {
        let bytes = self.reader.bytes(text)?;
        self.storage(
            bytes
                .len()
                .checked_mul(2)
                .ok_or("constant literal exceeds view size")?,
        )?;
        for byte in bytes {
            write!(row, "{byte:02x}").map_err(|_| "constant view formatting failed")?;
        }
        Ok(())
    }
    fn repeat_row(
        &mut self,
        row: &mut String,
        ty: TypeId,
        value: Constant,
        element: Constant,
        length: u64,
    ) -> Result<(), &'static str> {
        let fields = self.reader.field_types(ty, value)?;
        if fields.len() != length {
            return Err("constant repeat differs from its stored type");
        }
        write!(row, "repeat length={length} element=")
            .map_err(|_| "constant view formatting failed")?;
        if length == 0 {
            row.push_str("none");
        } else {
            let element = self.value(
                fields
                    .get(0)
                    .ok_or("constant repeat lacks an element type")?,
                element,
            )?;
            write!(row, "v{element}").map_err(|_| "constant view formatting failed")?;
        }
        Ok(())
    }
    fn product_row(
        &mut self,
        row: &mut String,
        ty: TypeId,
        value: Constant,
        length: u64,
        variant: Option<crate::sema::members::VariantId>,
    ) -> Result<(), &'static str> {
        let fields = self.reader.field_types(ty, value)?;
        if fields.len() != length {
            return Err("constant product differs from its stored type");
        }
        self.charge(usize::try_from(length).map_err(|_| "constant product exceeds host size")?)?;
        write!(
            row,
            "product variant={} fields=[",
            variant.map_or_else(|| "none".into(), |id| id.0.to_string())
        )
        .map_err(|_| "constant view formatting failed")?;
        for index in 0..length {
            self.storage(24)?;
            let field = self.value(
                fields.get(index).ok_or("constant field lacks a type")?,
                self.reader.field(value, index)?,
            )?;
            if index != 0 {
                row.push(',');
            }
            write!(row, "v{field}").map_err(|_| "constant view formatting failed")?;
        }
        row.push(']');
        Ok(())
    }
    pub(super) fn allocation_row(
        &mut self,
        id: usize,
        backing: Backing,
        ty: TypeId,
        value: Option<Constant>,
    ) -> Result<String, &'static str> {
        let mut row = format!("allocation a{id} type={} ", ty.0);
        match backing {
            Backing::Literal(text) => {
                row.push_str("origin=literal bytes=");
                self.hex(&mut row, text)?;
                return Ok(row);
            }
            Backing::Static(definition) => {
                write!(row, "origin=static:{}", definition.0)
                    .map_err(|_| "constant view formatting failed")?;
                if self
                    .facts
                    .foreign
                    .as_deref()
                    .is_some_and(|r| r.imports_global(definition))
                {
                    row.push_str(" contents=external");
                    return Ok(row);
                }
            }
            Backing::Location(location) => match self.reader.location(location)?.origin {
                Origin::ByteOffset { base, offset } => {
                    let base = self.location(base)?;
                    write!(
                        row,
                        "origin=byte-projection:a{base}:{offset} contents=alias"
                    )
                    .map_err(|_| "constant view formatting failed")?;
                    return Ok(row);
                }
                Origin::Projection { base, index } => {
                    let base = self.location(base)?;
                    write!(row, "origin=projection:a{base}:{index} contents=alias")
                        .map_err(|_| "constant view formatting failed")?;
                    return Ok(row);
                }
                Origin::Subsequence { base, from, length } => {
                    let base = self.location(base)?;
                    write!(
                        row,
                        "origin=subsequence:a{base}:{from}:{length} contents=alias"
                    )
                    .map_err(|_| "constant view formatting failed")?;
                    return Ok(row);
                }
                Origin::Allocation => row.push_str("origin=allocation"),
                Origin::Static(_) => {
                    return Err("static location lacks its canonical display identity");
                }
            },
        }
        let value = self.value(
            ty,
            value.ok_or("constant view allocation lacks completed contents")?,
        )?;
        write!(row, " contents=v{value}").map_err(|_| "constant view formatting failed")?;
        Ok(row)
    }
}
fn scalar_row(scalar: Scalar) -> String {
    match scalar {
        Scalar::Integer(bits) => format!("integer bits={bits:032x}"),
        Scalar::Float(FloatBits::F32(bits)) => format!("f32 bits={bits:08x}"),
        Scalar::Float(FloatBits::F64(bits)) => format!("f64 bits={bits:016x}"),
        Scalar::Boolean(value) => format!("bool bits={}", u8::from(value)),
        Scalar::Character(value) => format!("char bits={:08x}", u32::from(value)),
    }
}

fn callable_row(
    facts: &crate::sema::model::Declarations,
    instance: crate::types::instance::Instance,
) -> Result<String, &'static str> {
    Ok(match facts.types.kind(instance.ty()) {
        Some(Kind::FunctionItem {
            item, signature, ..
        }) => format!(
            "function def={} instance={} signature={} const={}",
            item.0,
            instance.ty().0,
            signature.0,
            facts.is_const_function(crate::resolve::model::DefId(item.0))
        ),
        Some(Kind::Closure {
            identity,
            signature,
            ..
        }) => format!(
            "function closure={} instance={} signature={} const=false",
            identity.0,
            instance.ty().0,
            signature.0
        ),
        _ => return Err("constant function value lacks a callable identity"),
    })
}
#[cfg(test)]
#[path = "../../../../tests/invariants/inspected_callables.rs"]
mod tests;
