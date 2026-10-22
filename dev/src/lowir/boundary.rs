//! Canonical semantic object boundaries retain target classes, never scalar placeholders.
use super::{
    program::{self, MemorySpan, Program, SignatureId},
    vocabulary::Type as Scalar,
};
use std::{collections::HashMap, sync::Arc};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Class {
    Integer,
    Sse,
    None,
}
impl Class {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Integer => "integer",
            Self::Sse => "sse",
            Self::None => "none",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Classes {
    Eightbytes { first: Class, second: Option<Class> },
    Memory,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Object {
    pub span: MemorySpan,
    pub classes: Classes,
}
impl Object {
    /// # Errors
    /// Rejects invalid object spans and incompatible eightbyte/result classes.
    pub fn validate(self, result: bool) -> Result<(), &'static str> {
        if self.span.bytes == 0
            || self.span.bytes > i64::MAX as u64
            || !self.span.align.is_power_of_two()
            || self.span.align > 1 << 29
        {
            return Err("invalid LowIR boundary object span");
        }
        match self.classes {
            Classes::Eightbytes { second, .. }
                if self.span.bytes <= 16 && second.is_some() == (self.span.bytes > 8) =>
            {
                Ok(())
            }
            Classes::Memory if self.span.bytes > 16 || !result => Ok(()),
            _ => Err("LowIR object classes disagree with boundary span or result"),
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    Scalar(Scalar),
    Object(Object),
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Signature {
    pub parameters: Box<[Type]>,
    pub result: Type,
    pub noreturn: bool,
    pub variadic: bool,
}
impl Signature {
    fn validate(&self) -> Result<(), &'static str> {
        let mut object = false;
        if self.parameters.len() > 1_000_000 {
            return Err("LowIR boundary parameter budget exceeded");
        }
        for &parameter in &self.parameters {
            match parameter {
                Type::Scalar(Scalar::Void) => return Err("void LowIR boundary parameter"),
                Type::Scalar(_) => {}
                Type::Object(value) => {
                    value.validate(false)?;
                    object = true;
                }
            }
        }
        if let Type::Object(value) = self.result {
            value.validate(true)?;
            object = true;
        }
        if !object {
            return Err("object signature requires an object boundary");
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignatureIdLocal(pub u32);
struct Entry {
    signature: Arc<Signature>,
    global: SignatureId,
}
#[derive(Default)]
pub(super) struct Signatures {
    entries: Vec<Entry>,
    by_value: HashMap<Arc<Signature>, SignatureIdLocal>,
}
impl Program {
    /// # Errors
    /// Rejects malformed classes, scalar-only boundaries and exhausted identities.
    pub fn object_signature(
        &mut self,
        parameters: &[Type],
        result: Type,
        noreturn: bool,
    ) -> Result<SignatureId, &'static str> {
        self.object_signature_with_arity(parameters, result, noreturn, false)
    }
    /// # Errors
    /// Rejects malformed classes, scalar-only boundaries and exhausted identities.
    pub fn object_signature_with_arity(
        &mut self,
        parameters: &[Type],
        result: Type,
        noreturn: bool,
        variadic: bool,
    ) -> Result<SignatureId, &'static str> {
        if parameters.len() > 1_000_000 {
            return Err("LowIR boundary parameter budget exceeded");
        }
        let signature = Signature {
            parameters: parameters.into(),
            result,
            noreturn,
            variadic,
        };
        signature.validate()?;
        let signatures = self.objects.get_or_insert_with(Default::default);
        if let Some(id) = signatures.by_value.get(&signature) {
            return Ok(signatures.entries[id.0 as usize].global);
        }
        let global = SignatureId(program::index(self.signatures.len())?);
        let local = SignatureIdLocal(program::index(signatures.entries.len())?);
        let signature = Arc::new(signature);
        signatures.entries.push(Entry {
            signature: Arc::clone(&signature),
            global,
        });
        signatures.by_value.insert(signature, local);
        self.signatures.push(program::Signature::Object(local));
        Ok(global)
    }
    /// # Errors
    /// Rejects invalid or scalar signature identities.
    pub fn object_boundary(&self, id: SignatureId) -> Result<&Signature, &'static str> {
        let Some(program::Signature::Object(local)) = self.signatures.get(id.0 as usize) else {
            return Err("LowIR signature is not an object boundary");
        };
        self.objects
            .as_ref()
            .and_then(|objects| objects.entries.get(local.0 as usize))
            .filter(|entry| entry.global == id)
            .map(|entry| entry.signature.as_ref())
            .ok_or("invalid LowIR object signature")
    }
    /// # Errors
    /// Rejects invalid signature identities.
    pub fn variadic(&self, id: SignatureId) -> Result<bool, &'static str> {
        match self.signatures.get(id.0 as usize) {
            Some(program::Signature::Scalar(value)) => Ok(value.variadic),
            Some(program::Signature::Object(_)) => Ok(self.object_boundary(id)?.variadic),
            None => Err("invalid LowIR signature"),
        }
    }
    /// # Errors
    /// Rejects invalid signature identities and parameter ranges.
    pub fn parameter_count(&self, id: SignatureId) -> Result<usize, &'static str> {
        match self.signatures.get(id.0 as usize) {
            Some(program::Signature::Scalar(_)) => Ok(self.parameters(id)?.len()),
            Some(program::Signature::Object(_)) => Ok(self.object_boundary(id)?.parameters.len()),
            None => Err("invalid LowIR signature"),
        }
    }
    /// # Errors
    /// Rejects invalid signature identities or parameter positions.
    pub fn parameter_type(&self, id: SignatureId, index: usize) -> Result<Type, &'static str> {
        match self.signatures.get(id.0 as usize) {
            Some(program::Signature::Scalar(_)) => {
                self.parameters(id)?.get(index).copied().map(Type::Scalar)
            }
            Some(program::Signature::Object(_)) => {
                self.object_boundary(id)?.parameters.get(index).copied()
            }
            None => return Err("invalid LowIR signature"),
        }
        .ok_or("invalid LowIR parameter position")
    }
    /// # Errors
    /// Rejects invalid signature identities.
    pub fn result_type(&self, id: SignatureId) -> Result<Type, &'static str> {
        match self.signatures.get(id.0 as usize) {
            Some(program::Signature::Scalar(value)) => Ok(Type::Scalar(value.result)),
            Some(program::Signature::Object(_)) => Ok(self.object_boundary(id)?.result),
            None => Err("invalid LowIR signature"),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ObjectValue {
    pub operand: program::Operand,
    pub span: MemorySpan,
}
#[derive(Clone, Copy, Debug)]
pub enum Value {
    Scalar(program::Value),
    Object(ObjectValue),
}
#[derive(Clone, Copy, Debug)]
pub enum ResultValue {
    Void,
    Scalar(program::Value),
    Object(ObjectValue),
}

#[cfg(test)]
#[path = "../../tests/invariants/boundary_records.rs"]
mod tests;
