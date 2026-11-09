//! Original-owned operation/region rows contain no serialized proof or source handles.
use super::{FunctionApplicationRef, RegionRef, TraitApplicationRef};
use crate::support::{
    constness::Constness,
    crates::{
        generics::{self as g, BinderRef, Range, TypeRef},
        owner_catalog::{BodyRef, DeclarationRef},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Local(u32),
    Projection(u32),
    /// Version-nine read or reference projection base: exact evaluated SSA snapshot.
    Temporary(u32),
}
/// Exactly a retained reference Dereference projection. Raw dereferences, fields,
/// captures and other projections are not encodable by this first source form.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Projection {
    pub base: Place,
    pub ty: TypeRef,
    pub kind: ProjectionKind,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProjectionKind {
    Dereference { region: RegionRef, mutable: bool },
    Field { owner: DeclarationRef, ordinal: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OwnedRegion {
    pub site: u32,
    pub allocation: RegionAllocation,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegionAllocation {
    Inference {
        needs_provenance: bool,
    },
    /// Ordinal of the exact owning raw normalization input in this body.
    Normalization(u32),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueKind {
    Aggregate {
        fields: Range,
    },
    Unit,
    /// Closed primitive bits in target little-endian order. The actual `Value.ty`
    /// supplies their type; no provider type ID or proof travels with these bits.
    Scalar([u8; 16]),
    Read(Place),
    /// Immediate only. The producer must reject `TwoPhase` and `UniqueImmutable`;
    /// absence of a kind field must never erase their distinct semantics.
    Borrow(Place),
    /// Version thirteen: a call operator's two-phase receiver borrow. Its
    /// reference type decides mutability; it is never an immediate borrow.
    TwoPhaseBorrow(Place),
    Constrain(u32),
    /// A closed primitive `as` conversion. The operand and result types determine
    /// its operation; no provider cast decision or semantic proof is serialized.
    ScalarCast(u32),
    Function(FunctionApplicationRef),
    Call {
        signature: TypeRef,
        callee: u32,
        arguments: Range,
    },
    /// Version thirteen: a call through a preceding closed Rust fn-pointer
    /// value. It selects no target and owns no request row: its site is its
    /// one evaluation, and `safety` is its lexical unsafe context. Every
    /// consumer re-proves the signature against the callee's pointer type.
    PointerCall {
        signature: TypeRef,
        callee: u32,
        arguments: Range,
        safety: CallSafety,
    },
    /// Version thirteen: a closed primitive comparison or bitwise operation. The
    /// operand and result types decide its typed operation; no provider operator
    /// selection, overflow decision or proof is serialized.
    Binary {
        operator: BinaryOperator,
        operands: OperandForm,
        left: u32,
        right: u32,
    },
    /// Version thirteen: a call operator. `arguments` starts with the owned or
    /// borrowed receiver, then the untupled inputs. The signature is the
    /// caller-side `fn(inputs) -> Output`; no selected impl, closure entry or
    /// trait proof is serialized, and every consumer selects the call again.
    Invoke {
        kind: crate::support::call_traits::CallTrait,
        signature: TypeRef,
        arguments: Range,
    },
    /// Version thirteen: an enum variant constructed from its positional
    /// payload. The variant declaration names its enum; the value type decides
    /// the instance. No tag, niche or layout decision is serialized.
    VariantAggregate {
        variant: DeclarationRef,
        fields: Range,
    },
    /// Version thirteen: the non-moving constructor test of an enum place.
    /// Every consumer re-proves the facts any downcast field use relies on.
    IsVariant {
        place: Place,
        variant: DeclarationRef,
    },
    /// Version thirteen: a fieldless enum's `as` conversion to its integer
    /// result; the discriminants come from the consumer's validated header.
    EnumCast(u32),
    /// Version thirteen: a `&'static str` literal, the original's text with
    /// this ordinal. Ordinals follow value order densely.
    Text(u32),
    /// Version thirteen: a preceding function-item value reified to the fn
    /// pointer `Value.ty`. No provider coercion decision is serialized: every
    /// consumer re-proves the item's applied signature against the pointer.
    Reify(u32),
    /// Version thirteen: one inline assembly invocation, the original's
    /// assembly instruction with this ordinal. Ordinals follow value order
    /// densely; `arguments` lists its inputs in operand order.
    Assembly {
        instruction: u32,
        arguments: Range,
    },
    /// Version thirteen: a read of a constant declaration of this artifact.
    /// Its header row holds the only copy of the value.
    Constant(DeclarationRef),
}
/// The reconstructable primitive operators. Add, Sub and Mul are checked: each
/// consumer instance lowers its own overflow guard and abort exit. Div, Rem and
/// the shifts own stable wire codes but no row form until their proof exists.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOperator {
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    BitAnd,
    BitOr,
    BitXor,
    Add,
    Sub,
    Mul,
}
impl BinaryOperator {
    #[must_use]
    pub const fn comparison(self) -> bool {
        !matches!(
            self,
            Self::BitAnd | Self::BitOr | Self::BitXor | Self::Add | Self::Sub | Self::Mul
        )
    }
    #[must_use]
    pub const fn arithmetic(self) -> bool {
        matches!(self, Self::Add | Self::Sub | Self::Mul)
    }
}
/// Retained lexical operand form; data for promotion, never execution permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperandForm {
    NotRequired,
    Neither,
    Left,
    Right,
    Both,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Value {
    pub ty: TypeRef,
    pub kind: ValueKind,
}
impl Value {
    pub(crate) fn scalar_cast_types(
        self,
        original: &Original,
        types: &[super::Type],
    ) -> Result<
        (
            crate::support::crates::Scalar,
            crate::support::crates::Scalar,
        ),
        &'static str,
    > {
        use crate::support::crates::Scalar;
        let ValueKind::ScalarCast(operand) = self.kind else {
            return Err("source cast request belongs to another value operation");
        };
        let operand = original
            .values
            .get(operand as usize)
            .ok_or("source scalar cast lost its actual operand")?;
        let (Some(&super::Type::Scalar(from)), Some(&super::Type::Scalar(to))) = (
            types.get(operand.ty.0 as usize),
            types.get(self.ty.0 as usize),
        ) else {
            return Err("source scalar cast requires actual closed primitive types");
        };
        let number = |ty| !matches!(ty, Scalar::Unit | Scalar::Bool | Scalar::Char);
        let integer = |ty| number(ty) && !matches!(ty, Scalar::F32 | Scalar::F64);
        if !(number(from) && number(to)
            || matches!(from, Scalar::Bool | Scalar::Char) && integer(to)
            || from == Scalar::U8 && to == Scalar::Char)
        {
            return Err("source scalar cast has an invalid primitive conversion");
        }
        Ok((from, to))
    }

    /// Both operands share one closed non-unit primitive; comparisons yield bool
    /// and bitwise operations keep their bool or integer operand type.
    pub(crate) fn binary_types(
        self,
        original: &Original,
        types: &[super::Type],
    ) -> Result<crate::support::crates::Scalar, &'static str> {
        use crate::support::crates::Scalar;
        let ValueKind::Binary {
            operator,
            left,
            right,
            ..
        } = self.kind
        else {
            return Err("source binary request belongs to another value operation");
        };
        let scalar = |value: u32| {
            let operand = original
                .values
                .get(value as usize)
                .ok_or("source binary operation lost its actual operand")?;
            match types.get(operand.ty.0 as usize) {
                Some(&super::Type::Scalar(ty)) if ty != Scalar::Unit => Ok(ty),
                _ => Err("source binary operation requires closed primitive operands"),
            }
        };
        let operand = scalar(left)?;
        if scalar(right)? != operand {
            return Err("source binary operation changed its shared operand type");
        }
        let result = if operator.comparison() {
            Scalar::Bool
        } else if operator.arithmetic() {
            if matches!(operand, Scalar::Bool | Scalar::Char) {
                return Err("source checked arithmetic requires integer or float operands");
            }
            operand
        } else if matches!(operand, Scalar::Char | Scalar::F32 | Scalar::F64) {
            return Err("source bitwise operation requires bool or integer operands");
        } else {
            operand
        };
        if types.get(self.ty.0 as usize) != Some(&super::Type::Scalar(result)) {
            return Err("source binary operation changed its primitive result type");
        }
        Ok(operand)
    }

    pub(crate) fn scalar_value(
        self,
        types: &[super::Type],
    ) -> Result<crate::support::crates::ScalarValue, &'static str> {
        use crate::support::crates::{Scalar, ScalarValue};
        let ValueKind::Scalar(bits) = self.kind else {
            return Err("source scalar request belongs to another value operation");
        };
        let Some(&super::Type::Scalar(ty)) = types.get(self.ty.0 as usize) else {
            return Err("source scalar value requires its actual closed primitive type");
        };
        if ty == Scalar::Unit {
            return Err("source unit value requires its distinct Unit operation");
        }
        ScalarValue::new(ty, u128::from_le_bytes(bits))
    }
    /// A text value is exactly a shared `'static` reference to `str`.
    pub(crate) fn text_type(
        self,
        types: &[super::Type],
        regions: &[super::Region],
    ) -> Result<(), &'static str> {
        let Some(&super::Type::Reference {
            mutable: false,
            region,
            referent,
        }) = types.get(self.ty.0 as usize)
        else {
            return Err("source text value is not a shared reference");
        };
        if regions.get(region.0 as usize) != Some(&super::Region::Static)
            || types.get(referent.0 as usize) != Some(&super::Type::Str)
        {
            return Err("source text value is not a static str reference");
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    StorageLive(u32),
    StorageDead(u32),
    StorageEnd {
        first: u32,
        last: u32,
    },
    Project(u32),
    Evaluate(u32),
    Store {
        place: Place,
        value: u32,
    },
    CheckInitialized(Place),
    Mention(Place),
    /// Version thirteen: start the attempt of the guard with this ordinal.
    GuardStart(u32),
    /// Version thirteen: keep that guard's protections live to its end.
    GuardUse(u32),
    /// Version thirteen: select a guard view's protected address.
    BindView {
        view: u32,
        place: Place,
    },
    /// Version thirteen: re-form a pattern place relative to its scrutinee.
    Select {
        projection: u32,
        anchor: Place,
    },
}
impl Action {
    /// Guard and pattern-selection actions exist only in version thirteen.
    #[must_use]
    pub const fn is_guard_form(self) -> bool {
        matches!(
            self,
            Self::GuardStart(_) | Self::GuardUse(_) | Self::BindView { .. } | Self::Select { .. }
        )
    }
}
/// A guard protects its tested places shallowly and its views deeply.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    /// Range of the original's tested guard places.
    pub tested: Range,
    /// Range of the original's locals that are this guard's views.
    pub views: Range,
}
/// Version thirteen's cold guard columns, present exactly when an original
/// starts a guard. Tested places are proof-only and never evaluated.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Guards {
    pub rows: Vec<Guard>,
    pub places: Vec<Place>,
}
/// Version thirteen's cold literal column, present exactly when an original
/// has a text value: each literal's UTF-8 contents, in value order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Texts {
    pub rows: Vec<Box<str>>,
}
/// Version thirteen's cold assembly columns, present exactly when an original
/// has an assembly value. No provider class, feature or lowering decision
/// travels: every consumer re-proves the initial assembly profile.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Assembly {
    pub instructions: Vec<AssemblyInstruction>,
    pub templates: Vec<AssemblyTemplate>,
    pub pieces: Vec<AssemblyPiece>,
    pub operands: Vec<AssemblyOperand>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssemblyInstruction {
    pub site: u32,
    pub templates: Range,
    pub operands: Range,
    pub options: u16,
    /// The lexical unsafe context; `Safe` never decodes.
    pub safety: CallSafety,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssemblyTemplate {
    pub text: Box<str>,
    pub pieces: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssemblyPiece {
    pub start: u32,
    pub end: u32,
    pub operand: u32,
    /// Only zero decodes; no modifier encoding is defined yet.
    pub modifier: u8,
}
/// The register class tags. Only `General` (`reg`) and `Vector` (`xmm_reg`)
/// have a wire layout; the others are reserved tags every reader refuses by
/// their initial-profile texts, so a writer can only forge them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssemblyClass {
    General,
    Abcd,
    Byte,
    Vector,
    Explicit,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AssemblyOperand {
    pub class: AssemblyClass,
    pub input: Option<u32>,
    /// Only `false` decodes; an output place has no encoding yet.
    pub output: bool,
    pub ty: TypeRef,
    pub late: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statement {
    pub site: u32,
    pub action: Action,
}
/// Exact late opening; the full resulting signature must match, not its arity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Opening {
    pub binder: BinderRef,
    pub regions: Range,
}
/// Actual defining call context, independent of constness or a paid call proof.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CallSafety {
    #[default]
    Safe,
    ExplicitUnsafeBlock,
    UnsafeFunction,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CallRequest {
    pub value: u32,
    pub site: u32,
    pub target: FunctionApplicationRef,
    pub method: Option<TraitApplicationRef>,
    pub signature: TypeRef,
    pub opening: Option<Opening>,
    /// Defining request data. Reached instantiation still derives and proves its
    /// actual runtime/const obligations; this is never a paid call witness.
    pub effect: Constness,
    pub safety: CallSafety,
}
/// Exact owned-rvalue storage inputs; ordinary destruction proof remains required.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Release {
    pub value: u32,
    pub location: g::Location,
    pub exits_before: u32,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OwnedValues {
    pub created: Vec<g::Temporary>,
    pub returned: Vec<Release>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallContext {
    Runtime,
    ConstDefinition,
    ClosedConst,
}
impl CallContext {
    #[must_use]
    pub const fn request(self) -> Constness {
        match self {
            Self::Runtime => Constness::Ordinary,
            Self::ConstDefinition => Constness::Conditional,
            Self::ClosedConst => Constness::Const,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Original {
    pub body: BodyRef,
    pub declaration: DeclarationRef,
    pub result: TypeRef,
    pub sites: u32,
    pub context: CallContext,
    pub variables: Vec<OwnedRegion>,
    pub locals: Vec<g::Local>,
    pub parameter_bindings: Vec<g::ParameterBinding>,
    pub projections: Vec<Projection>,
    pub values: Vec<Value>,
    pub statements: Vec<Statement>,
    pub blocks: Vec<g::Block>,
    pub scopes: Vec<g::Scope>,
    pub exits: Vec<g::Exit>,
    pub temporaries: Vec<g::Temporary>,
    pub owned: Option<OwnedValues>,
    pub arguments: Vec<u32>,
    pub openings: Vec<RegionRef>,
    pub calls: Vec<CallRequest>,
    /// Version thirteen: out of line, absent unless the body starts a guard.
    pub guards: Option<Box<Guards>>,
    /// Version thirteen: out of line, absent unless the body has a text value.
    pub texts: Option<Box<Texts>>,
    /// Version thirteen: out of line, absent unless the body has an assembly
    /// value.
    pub assembly: Option<Box<Assembly>>,
}
