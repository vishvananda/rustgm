//! Compact typed identities and function-owned `LowIR` arenas.
use super::vocabulary::{Binary, Convert, Predicate, Type, Unary};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SignatureId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SlotId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TempId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct LiteralId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SpanId(pub u32);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MemorySpan {
    pub bytes: u64,
    pub align: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum Storage {
    Scalar(Type),
    Object(SpanId),
}
#[derive(Clone, Copy, Debug)]
pub enum Operand {
    Temporary(TempId),
    Parameter(u32),
    Slot(SlotId),
    Symbol(SymbolId),
    Global(super::data::GlobalId),
    Literal(LiteralId),
}
#[derive(Clone, Copy, Debug)]
pub struct Value {
    pub ty: Type,
    pub operand: Operand,
}
#[derive(Clone, Copy, Debug)]
pub struct Range {
    pub start: u32,
    pub end: u32,
}
#[derive(Clone, Copy, Debug)]
pub enum Literal {
    Integer(u128),
    F32(u32),
    F64(u64),
    Null,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    Ordinary,
    Volatile,
}
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    AtomicLoad {
        ty: Type,
        address: Operand,
        ordering: super::atomic::LoadOrder,
    },
    AtomicStore {
        value: Value,
        address: Operand,
        ordering: super::atomic::StoreOrder,
    },
    AtomicExchange {
        value: Value,
        address: Operand,
        ordering: super::atomic::RmwOrder,
    },
    AtomicAddFetch {
        value: Value,
        address: Operand,
        ordering: super::atomic::RmwOrder,
    },
    Copy(Value),
    Address(Operand),
    Index {
        element: Type,
        base: Operand,
        offset: Operand,
    },
    CopyObject {
        span: SpanId,
        source: Operand,
        destination: Operand,
    },
    Load {
        access: Access,
        ty: Type,
        address: Operand,
    },
    Store {
        access: Access,
        value: Value,
        address: Operand,
    },
    Unary {
        op: Unary,
        value: Value,
    },
    Binary {
        op: Binary,
        left: Value,
        right: Operand,
    },
    Compare {
        predicate: Predicate,
        left: Value,
        right: Operand,
    },
    Convert {
        op: Convert,
        value: Value,
    },
    Call {
        signature: SignatureId,
        callee: Operand,
        arguments: Range,
    },
}
pub struct Instruction {
    pub result: Option<TempId>,
    pub operation: Operation,
}
#[derive(Clone, Copy, Debug)]
pub enum Terminator {
    Jump(BlockId),
    Branch {
        condition: Operand,
        yes: BlockId,
        no: BlockId,
    },
    Return(Option<Value>),
    Unreachable,
}
pub struct Block {
    pub instructions: Range,
    pub terminator: Terminator,
}
#[derive(Default)]
pub struct Function {
    pub slots: Vec<Storage>,
    pub spans: Vec<MemorySpan>,
    pub temporaries: Vec<Type>,
    pub literals: Vec<Literal>,
    pub arguments: Vec<Value>,
    pub instructions: Vec<Instruction>,
    pub blocks: Vec<Block>,
    pub order: Vec<BlockId>,
}
pub struct Signature {
    pub parameters: Range,
    pub result: Type,
    pub noreturn: bool,
}
pub struct Symbol {
    /// Backend spelling only. Semantic lookup uses `SymbolId` and source definition identities.
    pub name: String,
    pub signature: SignatureId,
    pub entry: bool,
    pub body: Option<Function>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Name {
    Function(SymbolId),
    Global(super::data::GlobalId),
}
#[derive(Default)]
pub struct Program {
    pub globals: Vec<super::data::Global>,
    pub data: Vec<super::data::Item>,
    pub parameters: Vec<Type>,
    pub signatures: Vec<Signature>,
    pub symbols: Vec<Symbol>,
    pub(super) names: std::collections::HashMap<String, Name>,
}
impl Program {
    /// Install exactly one function body for a canonical symbol declaration.
    ///
    /// # Errors
    /// Rejects invalid or noncanonical identities and repeated installation.
    pub fn define(&mut self, symbol: SymbolId, body: Function) -> Result<(), &'static str> {
        let target = self
            .symbols
            .get(symbol.0 as usize)
            .ok_or("invalid LowIR definition symbol")?;
        if self.names.get(&target.name) != Some(&Name::Function(symbol)) {
            return Err("noncanonical LowIR definition symbol");
        }
        if target.body.is_some() {
            return Err("multiple function definitions for one LowIR symbol");
        }
        self.symbols[symbol.0 as usize].body = Some(body);
        Ok(())
    }
    #[must_use]
    pub fn parameters(&self, signature: SignatureId) -> &[Type] {
        let list = self.signatures[signature.0 as usize].parameters;
        &self.parameters[list.start as usize..list.end as usize]
    }
    /// # Errors
    /// Rejects exhausted compact identities and void parameters.
    pub fn signature(
        &mut self,
        parameters: &[Type],
        result: Type,
        noreturn: bool,
    ) -> Result<SignatureId, &'static str> {
        if parameters.contains(&Type::Void) {
            return Err("void LowIR parameter");
        }
        let id = SignatureId(index(self.signatures.len())?);
        let start = index(self.parameters.len())?;
        let end = index(
            self.parameters
                .len()
                .checked_add(parameters.len())
                .ok_or("LowIR parameter space exhausted")?,
        )?;
        self.parameters.extend_from_slice(parameters);
        self.signatures.push(Signature {
            parameters: Range { start, end },
            result,
            noreturn,
        });
        Ok(id)
    }
    /// # Errors
    /// Rejects invalid signatures, incompatible aliases and exhausted identities.
    pub fn declare(
        &mut self,
        name: String,
        signature: SignatureId,
        entry: bool,
    ) -> Result<SymbolId, &'static str> {
        if signature.0 as usize >= self.signatures.len() {
            return Err("invalid LowIR signature");
        }
        if let Some(&named) = self.names.get(&name) {
            let Name::Function(id) = named else {
                return Err("LowIR symbol kind conflicts with prior declaration");
            };
            let previous = &self.symbols[id.0 as usize];
            let a = &self.signatures[previous.signature.0 as usize];
            let b = &self.signatures[signature.0 as usize];
            if previous.entry != entry
                || a.result != b.result
                || a.noreturn != b.noreturn
                || self.parameters(previous.signature) != self.parameters(signature)
            {
                return Err("incompatible LowIR symbol declarations");
            }
            return Ok(id);
        }
        let id = SymbolId(index(self.symbols.len())?);
        self.names.insert(name.clone(), Name::Function(id));
        self.symbols.push(Symbol {
            name,
            signature,
            entry,
            body: None,
        });
        Ok(id)
    }
}
pub(super) fn index(value: usize) -> Result<u32, &'static str> {
    u32::try_from(value).map_err(|_| "LowIR identity space exhausted")
}
