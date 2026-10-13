//! Deterministic type facts are an output view; later phases consume the model.
use super::model::{
    AdjustmentKind, BodyRegion, BodyRegionOrigin, Declarations, InitializerKind, Operation,
    RegionEquality, Scalar,
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
            write!(out, " source={}:{}", node.file.0, node.node.0)?;
        }
        writeln!(out)?;
    }
    for function in &facts.functions {
        write!(
            out,
            "function def={} source={}:{} signature={} body=",
            function.definition.0, function.node.file.0, function.node.node.0, function.signature.0
        )?;
        if let Some(body) = function.body {
            write!(out, "{}:{}", body.file.0, body.node.0)?;
        } else {
            write!(out, "-")?;
        }
        writeln!(out)?;
    }
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
            initializer.node.node.0,
            initializer.value.file.0,
            initializer.value.node.0
        )?;
    }
    body_regions(&facts.body_regions, out)?;
    regions(&facts.region_equalities, out)?;
    nodes(report, facts, out)?;
    write!(out, "eof ")?;
    count(facts, out)
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
            equality.body.file.0, equality.body.node.0
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
pub fn body_regions(regions: &[BodyRegion], out: &mut impl Write) -> io::Result<()> {
    for (ordinal, variable) in regions.iter().enumerate() {
        let origin = match variable.origin {
            BodyRegionOrigin::PatternBorrow => "pattern-borrow",
            BodyRegionOrigin::PatternInput => "pattern-input",
        };
        writeln!(
            out,
            "region-var region:{} body={}:{} source={}:{} origin={origin} pending",
            ordinal + 1,
            variable.body.file.0,
            variable.body.node.0,
            variable.source.file.0,
            variable.source.node.0
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
                write!(out, " control={file}:{}", target.0)?;
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
            if let Some(value) = fact.adjustment {
                let kind = match value.kind {
                    AdjustmentKind::Never => "never",
                    AdjustmentKind::FunctionPointer => "function-pointer",
                    AdjustmentKind::Equality => "equality",
                    AdjustmentKind::ArrayToSlice => "array-to-slice",
                };
                writeln!(
                    out,
                    "adjustment {file}:{node} {kind} target={}",
                    value.target.0
                )?;
            }
        }
    }
    Ok(())
}
/// # Errors
/// Propagates output failures after the same semantic analysis as the full view.
pub fn count(facts: &Declarations, out: &mut impl Write) -> io::Result<()> {
    writeln!(
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
    )
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
fn ty(store: &Store, id: TypeId, out: &mut impl Write) -> io::Result<()> {
    match store
        .kind(id)
        .ok_or_else(|| io::Error::other("invalid type identity"))?
    {
        Kind::Never => write!(out, "never"),
        Kind::Primitive(p) => write!(out, "primitive {}", p.name()),
        Kind::Adt(item) => write!(out, "aggregate def={}", item.0),
        Kind::Tuple(elements) => {
            write!(out, "tuple ")?;
            list(store, elements, out)
        }
        Kind::Array { element, length } => {
            write!(out, "array element={} length={length}", element.0)
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
        } => {
            write!(out, "function binders={bound_regions} parameters=")?;
            list(store, parameters, out)?;
            write!(out, " result={} unsafe={unsafe_}", result.0)
        }
        Kind::FunctionItem { item, signature } => write!(
            out,
            "function-item def={} signature={}",
            item.0, signature.0
        ),
    }
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
        Operation::Field { field } => write!(out, "field {field:?}"),
        Operation::Index { length } => write!(out, "index length={length}"),
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
