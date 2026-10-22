//! Deterministic type facts are an output view; later phases consume the model.
mod dynamic;
use super::model::{
    AdjustmentKind, BodyRegionOrigin, Declarations, InitializerKind, Operation, RegionEquality,
    Scalar,
};
use crate::{
    literal::value::FloatBits,
    resolve::expansion::Report,
    support::quoted,
    types::{Kind, ListId, RegionId, Store, TypeId},
};
use std::io::{self, Write};

/// # Errors
/// Propagates output errors and inconsistent canonical identities.
pub fn write(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(out, "typed-crate 1")?;
    types(&facts.types, out)?;
    definitions(report, facts, out)?;
    for function in &facts.functions {
        write!(
            out,
            "function def={} source={}:{} signature={} body=",
            function.definition.0,
            function.node.file.0,
            function.node.node.get(),
            function.signature.0
        )?;
        if let Some(body) = function.body {
            write!(out, "{}:{}", body.file.0, body.node.get())?;
        } else {
            write!(out, "-")?;
        }
        writeln!(out)?;
    }
    destructors(facts, out)?;
    for initializer in &facts.initializers {
        let kind = match initializer.kind {
            InitializerKind::Const => "const",
            InitializerKind::Static { mutable: false } => "static",
            InitializerKind::Static { mutable: true } => "static-mut",
        };
        writeln!(
            out,
            "initializer {kind} def={} type={} source={}:{} value={}:{}",
            initializer.definition.0,
            initializer.ty.0,
            initializer.node.file.0,
            initializer.node.node.get(),
            initializer.value.file.0,
            initializer.value.node.get()
        )?;
    }
    body_regions(facts.body_regions.log(), out)?;
    regions(&facts.region_equalities, out)?;
    coercions(&facts.coercions, out)?;
    outlives(&facts.outlives, out)?;
    nodes(report, facts, out)?;
    write!(out, "eof ")?;
    count(facts, out)
}

pub(super) fn definitions(
    report: &Report,
    facts: &Declarations,
    out: &mut impl Write,
) -> io::Result<()> {
    for (ordinal, ty) in facts.definitions.iter().enumerate() {
        let Some(ty) = ty else {
            continue;
        };
        let definition = &report.resolution.definitions[ordinal];
        write!(out, "def {ordinal} type={} name=", ty.0)?;
        quoted::bytes(
            out,
            report.package.names.get(definition.name).as_bytes(),
            false,
        )?;
        if let Some(node) = definition.node {
            write!(out, " source={}:{}", node.file.0, node.node.get())?;
        }
        writeln!(out)?;
    }
    Ok(())
}

fn destructors(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for implementation in facts.destructors.implementations() {
        writeln!(
            out,
            "destructor item={} self={} method={} target={} captures={} source={}:{} cleanup=pending",
            implementation.item.0,
            implementation.self_definition.0,
            implementation.method.0,
            implementation.target.0,
            implementation.parameters,
            implementation.source.file.0,
            implementation.source.node.get()
        )?;
        for &(a, b) in &implementation.bounds {
            write!(
                out,
                "destructor-bound method={} longer=",
                implementation.method.0
            )?;
            region(a, out)?;
            write!(out, " shorter=")?;
            region(b, out)?;
            writeln!(out)?;
        }
    }
    let mut receivers: Vec<_> = facts.receivers.iter().collect();
    receivers.sort_unstable_by_key(|(key, _)| (key.file.0, key.node.get()));
    for (key, receiver) in receivers {
        writeln!(
            out,
            "receiver parameter={}:{} binding={}:{} def={} type={} mutable={}",
            key.file.0,
            key.node.get(),
            receiver.binding.file.0,
            receiver.binding.node.get(),
            receiver.definition.0,
            receiver.ty.0,
            receiver.mutable
        )?;
    }
    Ok(())
}
/// # Errors
/// Propagates output errors; these are obligations, not completed proofs.
pub fn outlives(bounds: &[super::model::Outlives], out: &mut impl Write) -> io::Result<()> {
    for bound in bounds {
        write!(
            out,
            "region-outlives body={}:{} source={}:{} longer=",
            bound.body.file.0,
            bound.body.node.get(),
            bound.source.file.0,
            bound.source.node.get()
        )?;
        region(bound.longer, out)?;
        write!(out, " shorter=")?;
        region(bound.shorter, out)?;
        writeln!(out, " pending")?;
    }
    Ok(())
}
/// # Errors
/// Propagates output errors; preserve every directed conversion for inspection.
pub fn coercions(coercions: &[super::model::Coercion], out: &mut impl Write) -> io::Result<()> {
    for coercion in coercions {
        let kind = match coercion.kind {
            AdjustmentKind::Never => "never",
            AdjustmentKind::FunctionPointer => "function-pointer",
            AdjustmentKind::ArrayToSlice => "array-to-slice",
            AdjustmentKind::Unsize => "unsize",
            AdjustmentKind::Reborrow => "reborrow",
            AdjustmentKind::Equality => "same-shape",
            AdjustmentKind::Pointer => "pointer",
            AdjustmentKind::PointerReborrow => "pointer-reborrow",
        };
        writeln!(
            out,
            "region-coercion body={}:{} source={}:{} actual={} expected={} kind={kind} pending",
            coercion.body.file.0,
            coercion.body.node.get(),
            coercion.source.file.0,
            coercion.source.node.get(),
            coercion.actual.0,
            coercion.expected.0
        )?;
    }
    Ok(())
}
/// Write pending region constraints without claiming they have been proved.
///
/// # Errors
/// Propagates output errors.
pub fn regions(equalities: &[RegionEquality], out: &mut impl Write) -> io::Result<()> {
    for equality in equalities {
        write!(
            out,
            "region-equal body={}:{} left=",
            equality.body.file.0,
            equality.body.node.get()
        )?;
        region(equality.left, out)?;
        write!(out, " right=")?;
        region(equality.right, out)?;
        writeln!(out)?;
    }
    Ok(())
}
/// Write existential region provenance without implying a solved lifetime.
///
/// # Errors
/// Propagates output errors.
pub fn body_regions(
    regions: &[super::body_regions::AllocatedRegion],
    out: &mut impl Write,
) -> io::Result<()> {
    for allocated in regions {
        let variable = &allocated.region;
        let origin = match variable.origin {
            BodyRegionOrigin::QuantifiedProof => "quantified-proof",
            BodyRegionOrigin::Annotation => "annotation",
            BodyRegionOrigin::Inference => "inference",
            BodyRegionOrigin::ItemArgument => "item-argument",
            BodyRegionOrigin::CallArgument => "call-argument",
            BodyRegionOrigin::UniversalAlias => "universal-alias",
            BodyRegionOrigin::ExpressionBorrow => "expression-borrow",
            BodyRegionOrigin::PatternBorrow => "pattern-borrow",
            BodyRegionOrigin::PatternInput => "pattern-input",
        };
        writeln!(
            out,
            "region-var region:{} body={}:{} source={}:{} origin={origin} pending",
            allocated.id.0,
            variable.body.file.0,
            variable.body.node.get(),
            variable.source.file.0,
            variable.source.node.get()
        )?;
    }
    Ok(())
}
fn nodes(report: &Report, facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    for (file, nodes) in facts.nodes.iter().enumerate() {
        for (node, fact) in nodes.iter().enumerate() {
            let Some(ty) = fact.ty else {
                continue;
            };
            let span = report.package.units[file].configured.ast.nodes[node].span;
            write!(
                out,
                "node {file}:{node} type={} span={}:{}",
                ty.0, span.start, span.end
            )?;
            if let Some(binding) = fact.binding {
                write!(out, " binding={}", binding.0)?;
            }
            if let Some(Operation::Transfer { target, .. }) = fact.operation {
                write!(out, " control={file}:{}", target.get())?;
            }
            writeln!(out)?;
            if let Some(value) = fact.scalar {
                write!(out, "scalar {file}:{node} ")?;
                scalar(value, out)?;
                writeln!(out)?;
            }
            if let Some(value) = fact.operation
                && !matches!(value, Operation::Transfer { .. })
            {
                write!(out, "operation {file}:{node} ")?;
                operation(value, out)?;
                writeln!(out)?;
            }
            let key = crate::resolve::unit::NodeKey {
                file: crate::resolve::unit::FileId(u32::try_from(file).map_err(io::Error::other)?),
                node: crate::syntax::ast::NodeId::new(
                    u32::try_from(node).map_err(io::Error::other)?,
                ),
            };
            if let Some(count) = facts.autoderefs.get(&key) {
                writeln!(out, "autoderef {file}:{node} count={count}")?;
            }
            if let Some(value) = fact.adjustment {
                let kind = match value.kind {
                    AdjustmentKind::Never => "never",
                    AdjustmentKind::FunctionPointer => "function-pointer",
                    AdjustmentKind::Equality => "equality",
                    AdjustmentKind::ArrayToSlice => "array-to-slice",
                    AdjustmentKind::Unsize => "unsize",
                    AdjustmentKind::Reborrow => "reborrow",
                    AdjustmentKind::Pointer => "pointer",
                    AdjustmentKind::PointerReborrow => "pointer-reborrow",
                };
                writeln!(
                    out,
                    "adjustment {file}:{node} {kind} target={}",
                    value.target.0
                )?;
                if let Some(reference) = facts.reference_adjustments.get(&key) {
                    writeln!(
                        out,
                        "reference-adjustment {file}:{node} dereferences={} unsize={} two-phase={} borrow={}",
                        reference.conversion.dereferences,
                        reference.conversion.unsize,
                        reference.conversion.two_phase,
                        reference.borrow.0
                    )?;
                }
            }
        }
    }
    Ok(())
}
/// # Errors
/// Propagates output failures after the same semantic analysis as the full view.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    write!(
        out,
        "types={} definitions={} nodes={} functions={} initializers={}",
        facts.types.len(),
        facts.definitions.iter().filter(|t| t.is_some()).count(),
        facts
            .nodes
            .iter()
            .flatten()
            .filter(|n| n.ty.is_some())
            .count(),
        facts.functions.len(),
        facts.initializers.len()
    )?;
    if !facts.destructors.implementations().is_empty() || !facts.receivers.is_empty() {
        write!(
            out,
            " destructors={} receivers={}",
            facts.destructors.implementations().len(),
            facts.receivers.len()
        )?;
    }
    writeln!(out)
}
fn list(store: &Store, id: ListId, out: &mut impl Write) -> io::Result<()> {
    write!(out, "[")?;
    for (ordinal, ty) in store
        .list(id)
        .ok_or_else(|| io::Error::other("invalid type list"))?
        .iter()
        .enumerate()
    {
        if ordinal != 0 {
            write!(out, ",")?;
        }
        write!(out, "{}", ty.0)?;
    }
    write!(out, "]")
}
pub(super) fn region(id: RegionId, out: &mut impl Write) -> io::Result<()> {
    if id == RegionId::STATIC {
        write!(out, "static")
    } else if let Some((depth, index)) = id.bound_parts() {
        write!(out, "bound:{depth}:{index}")
    } else {
        write!(out, "region:{}", id.0)
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one exhaustive type rendering dispatch preserves the stable vocabulary"
)]
fn ty(store: &Store, id: TypeId, out: &mut impl Write) -> io::Result<()> {
    match store
        .kind(id)
        .ok_or_else(|| io::Error::other("invalid type identity"))?
    {
        Kind::Closure {
            identity,
            arguments,
            signature,
            environment,
        } => {
            write!(
                out,
                "closure id={} signature={} environment={}",
                identity.0, signature.0, environment.0
            )?;
            generic_arguments(store, arguments, out)
        }
        Kind::Never => write!(out, "never"),
        Kind::Primitive(p) => write!(out, "primitive {}", p.name()),
        Kind::Parameter(id) => {
            let parameter = store
                .parameter(id)
                .ok_or_else(|| io::Error::other("invalid type parameter"))?;
            write!(
                out,
                "parameter def={} index={}",
                parameter.owner.0, parameter.index
            )
        }
        Kind::Adt(item, args) => {
            write!(out, "aggregate def={}", item.0)?;
            region_arguments(store, args, out)
        }
        Kind::Opaque(id, arguments) => {
            write!(out, "opaque id={}", id.0)?;
            generic_arguments(store, arguments, out)
        }
        kind @ Kind::Dynamic { .. } => dynamic::write(store, kind, out),
        Kind::Projection(application) => {
            let trait_ = store
                .trait_application(application.trait_application)
                .ok_or_else(|| io::Error::other("invalid projection trait"))?;
            write!(
                out,
                "projection def={} self={} trait={}",
                application.item.0, application.subject.0, trait_.item.0
            )?;
            generic_arguments(store, trait_.arguments, out)?;
            write!(out, " family")?;
            generic_arguments(store, application.arguments, out)
        }
        Kind::Tuple(elements) => {
            write!(out, "tuple ")?;
            list(store, elements, out)
        }
        Kind::Array { element, length } => {
            write!(out, "array element={} length={length}", element.0)
        }
        Kind::ArrayTerm { element, length } => {
            let Some(crate::types::arguments::ConstKind::Parameter(id)) = store.const_kind(length)
            else {
                return Err(io::Error::other("invalid symbolic array length"));
            };
            let parameter = store
                .parameter(id)
                .ok_or_else(|| io::Error::other("missing array length parameter"))?;
            write!(
                out,
                "array element={} length=parameter:{}:{}",
                element.0, parameter.owner.0, parameter.index
            )
        }
        Kind::Slice(element) => write!(out, "slice element={}", element.0),
        Kind::Pointer { mutable, pointee } => {
            write!(out, "pointer mutable={mutable} pointee={}", pointee.0)
        }
        Kind::Reference {
            mutable,
            region: id,
            referent,
        } => {
            write!(out, "reference mutable={mutable} region=")?;
            region(id, out)?;
            write!(out, " referent={}", referent.0)
        }
        Kind::Function {
            bound_regions,
            parameters,
            result,
            unsafe_,
            convention,
        } => {
            write!(out, "function binders={bound_regions} parameters=")?;
            list(store, parameters, out)?;
            write!(out, " result={} unsafe={unsafe_}", result.0)?;
            if convention != crate::types::callable::Convention::RUST {
                write!(
                    out,
                    " abi={:?} variadic={}",
                    convention.abi, convention.variadic
                )?;
            }
            Ok(())
        }
        Kind::FunctionItem {
            item,
            signature,
            arguments,
        } => {
            write!(
                out,
                "function-item def={} signature={}",
                item.0, signature.0
            )?;
            region_arguments(store, arguments, out)
        }
    }
}
fn region_arguments(
    store: &Store,
    arguments: crate::types::GenericArgs,
    out: &mut impl Write,
) -> io::Result<()> {
    let Some(args) = store.region_args(arguments) else {
        return generic_arguments(store, arguments, out);
    };
    if !args.is_empty() {
        write!(out, " regions=[")?;
        for (ordinal, id) in args.iter().copied().enumerate() {
            if ordinal != 0 {
                write!(out, ",")?;
            }
            region(id, out)?;
        }
        write!(out, "]")?;
    }
    Ok(())
}
pub(super) fn generic_arguments(
    store: &Store,
    arguments: crate::types::GenericArgs,
    out: &mut impl Write,
) -> io::Result<()> {
    let arguments = store
        .arguments(arguments)
        .ok_or_else(|| io::Error::other("invalid generic arguments"))?;
    write!(out, " arguments=[")?;
    for (index, argument) in arguments.enumerate() {
        if index != 0 {
            write!(out, ",")?;
        }
        self::argument(store, argument, out)?;
    }
    write!(out, "]")
}
pub(super) fn argument(
    store: &Store,
    argument: crate::types::arguments::Argument,
    out: &mut impl Write,
) -> io::Result<()> {
    use crate::types::arguments::{Argument, ConstKind};
    match argument {
        Argument::Lifetime(id) => {
            write!(out, "region:")?;
            region(id, out)?;
        }
        Argument::Type(id) => write!(out, "type:{}", id.0)?,
        Argument::Const(id) => match store
            .const_kind(id)
            .ok_or_else(|| io::Error::other("invalid generic constant"))?
        {
            ConstKind::Value(value) => {
                write!(out, "const:{}:0x{:032x}", value.ty.0, value.bits)?;
            }
            ConstKind::Parameter(id) => {
                let parameter = store
                    .parameter(id)
                    .ok_or_else(|| io::Error::other("missing const parameter"))?;
                write!(
                    out,
                    "const-parameter:{}:{}",
                    parameter.owner.0, parameter.index
                )?;
            }
        },
    }
    Ok(())
}
/// Write the common canonical type table for semantic and body views.
///
/// # Errors
/// Propagates output errors and inconsistent canonical identities.
pub fn types(store: &Store, out: &mut impl Write) -> io::Result<()> {
    for ordinal in 0..store.len() {
        let id = TypeId(
            u32::try_from(ordinal).map_err(|_| io::Error::other("too many canonical types"))?,
        );
        write!(out, "type {ordinal} ")?;
        ty(store, id, out)?;
        writeln!(out)?;
    }
    Ok(())
}
/// Write target scalar bits in the shared fact-view vocabulary.
///
/// # Errors
/// Propagates output errors.
pub fn scalar(value: Scalar, out: &mut impl Write) -> io::Result<()> {
    match value {
        Scalar::Integer(bits) => write!(out, "integer bits=0x{bits:032x}"),
        Scalar::Float(FloatBits::F32(bits)) => write!(out, "f32 bits=0x{bits:08x}"),
        Scalar::Float(FloatBits::F64(bits)) => write!(out, "f64 bits=0x{bits:016x}"),
        Scalar::Boolean(value) => write!(out, "bool {value}"),
        Scalar::Character(value) => write!(out, "char U+{:06X}", u32::from(value)),
    }
}
fn operation(value: Operation, out: &mut impl Write) -> io::Result<()> {
    match value {
        Operation::Construct { variant } => write!(out, "construct variant={}", variant.0),
        Operation::FieldInit { field } => write!(out, "field-init field={}", field.0),
        Operation::Borrow => write!(out, "borrow"),
        Operation::Address => write!(out, "address"),
        Operation::AddressDereference => write!(out, "address-dereference"),
        Operation::PointerAddress { operand } => {
            write!(out, "pointer-address operand={}", operand.get())
        }
        Operation::Dereference => write!(out, "deref"),
        Operation::IndexSlice => write!(out, "index length=dynamic"),
        Operation::Field { field } => write!(out, "field {field:?}"),
        Operation::Index { length } => write!(out, "index length={length}"),
        Operation::IndexParameter { length } => write!(out, "index length=const:{}", length.0),
        Operation::AssignPlace {
            operator,
            left,
            right,
        } => write!(
            out,
            "assign-place operator={operator:?} left={} right={}",
            left.0, right.0
        ),
        Operation::Transfer { .. } => Err(io::Error::other("transfer belongs on node record")),
        Operation::Unary {
            operator,
            operand,
            literal,
        } => write!(
            out,
            "unary {} operand={} literal={literal}",
            operator.name(),
            operand.0
        ),
        Operation::Binary {
            operator,
            left,
            right,
        } => write!(out, "binary {operator:?} left={} right={}", left.0, right.0),
        Operation::Cast {
            kind,
            source,
            target,
        } => write!(out, "cast {kind:?} source={} target={}", source.0, target.0),
        Operation::Call { signature } => write!(out, "call signature={}", signature.0),
        Operation::Assign {
            target,
            operator,
            left,
            right,
        } => {
            write!(out, "assign def={} operator=", target.0)?;
            if let Some(operator) = operator {
                write!(out, "{operator:?}")?;
            } else {
                write!(out, "=")?;
            }
            write!(out, " left={} right={}", left.0, right.0)
        }
    }
}
