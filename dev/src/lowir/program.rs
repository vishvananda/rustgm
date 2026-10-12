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
#[derive(Clone, Copy, Debug)]
pub enum Operand {
    Temporary(TempId),
    Parameter(u32),
    Slot(SlotId),
    Symbol(SymbolId),
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
#[derive(Clone, Copy, Debug)]
pub enum Operation {
    Copy(Value),
    Address(Operand),
    Load {
        ty: Type,
        address: Operand,
    },
    Store {
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
    pub slots: Vec<Type>,
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
#[derive(Default)]
pub struct Program {
    pub parameters: Vec<Type>,
    pub signatures: Vec<Signature>,
    pub symbols: Vec<Symbol>,
}
impl Program {
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
    /// Rejects an invalid signature or exhausted symbol identities.
    pub fn declare(
        &mut self,
        name: String,
        signature: SignatureId,
        entry: bool,
    ) -> Result<SymbolId, &'static str> {
        if signature.0 as usize >= self.signatures.len() {
            return Err("invalid LowIR signature");
        }
        let id = SymbolId(index(self.symbols.len())?);
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
