//! Library exports consume resolved bindings and completed signature/member facts.
mod named_values;
mod public_names;
mod records;
mod static_exports;
use super::error;
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::{foreign, model::Declarations},
    support::{
        crates::{CrateIdentity, ExternalDef, Interface, Scalar, Symbol},
        linkage::{Linkage as Kind, Naming},
    },
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Safety},
    },
    types::{Kind as Type, Primitive, TypeId, arguments::Argument},
};

pub struct Exports {
    pub interface: Interface,
    pub roots: Vec<DefId>,
}
/// Publish a supported metadata profile after ordinary semantic checking.
/// The caller must complete body proofs/lowering before writing the artifact.
/// # Errors
/// Diagnoses unsupported exposed declarations instead of silently omitting them.
pub fn export(
    report: &Report,
    facts: &mut Declarations,
    identity: CrateIdentity,
) -> Result<Exports, Error> {
    if facts
        .crates
        .as_ref()
        .and_then(|linkage| linkage.identity())
        .is_some_and(|old| old != &identity)
    {
        return Err(error("conflicting local crate linkage identity"));
    }
    let mut functions = Vec::new();
    let mut constants = Vec::new();
    let mut roots = Vec::new();
    let mut records = Vec::new();
    let mut statics = Vec::new();
    let plan = public_names::collect(report, &identity)?;
    for (definition, external) in plan.owned {
        match report.resolution.definitions[definition.0 as usize].kind {
            DefKind::Function => {
                functions.push(function(report, facts, definition, external)?);
                roots.push(definition);
            }
            DefKind::Const => constants.push(named_values::export(facts, definition, external)?),
            DefKind::Struct => records.push(definition),
            DefKind::Static => statics.push(definition),
            _ => return Err(error("unsupported owned metadata declaration")),
        }
    }
    let dependencies = report
        .resolution
        .crates
        .as_ref()
        .map_or_else(Vec::new, |imported| {
            let mut identities: Vec<_> = imported
                .dependencies()
                .aliases()
                .iter()
                .map(|alias| alias.identity().clone())
                .collect();
            identities.sort_unstable();
            identities.dedup();
            identities
        });
    functions.sort_unstable_by(|left, right| left.definition.cmp(&right.definition));
    let storage = static_exports::prepare(report, facts, &identity, &statics)?;
    let interface = records::finish(
        report,
        facts,
        identity.clone(),
        dependencies,
        Prepared {
            functions,
            constants,
            records,
            storage,
        },
    )?;
    let interface = if let Some(names) = plan.names {
        interface.with_public_names(names).map_err(error)?
    } else {
        interface
    };
    let linkage = facts.crates.get_or_insert_default();
    linkage.set_identity(identity).map_err(error)?;
    roots.sort_unstable_by_key(|definition| definition.0);
    Ok(Exports { interface, roots })
}
struct PreparedFunction {
    definition: ExternalDef,
    signature: TypeId,
    symbol: Symbol,
}
struct Prepared {
    functions: Vec<PreparedFunction>,
    constants: Vec<crate::support::crates::Constant>,
    records: Vec<DefId>,
    storage: static_exports::Prepared,
}
fn function(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    external: ExternalDef,
) -> Result<PreparedFunction, Error> {
    let callable = callable(report, facts, definition)?;
    let signature = callable.signature;
    let registry = facts.foreign.get_or_insert_default();
    let naming = registry
        .function(definition)
        .map_or(Naming::Rust, |record| record.naming);
    let symbol = match naming {
        Naming::Rust => Symbol::Rust,
        Naming::Exact(id) => Symbol::Exact(
            registry
                .names()
                .get(id)
                .ok_or_else(|| error("missing export symbol spelling"))?
                .into(),
        ),
    };
    registry
        .insert_function(
            definition,
            foreign::Function {
                signature,
                safety: callable.safety,
                naming,
                linkage: callable.linkage,
            },
        )
        .map_err(error)?;
    facts
        .crates
        .get_or_insert_default()
        .symbols
        .insert(definition, external.rust_symbol());
    Ok(PreparedFunction {
        definition: external,
        signature,
        symbol,
    })
}
struct Callable {
    signature: TypeId,
    safety: Safety,
    linkage: Kind,
}
fn callable(report: &Report, facts: &Declarations, definition: DefId) -> Result<Callable, Error> {
    if facts.target_feature_functions.contains(&definition) {
        return Err(error(
            "target-feature exports are outside the scalar metadata profile",
        ));
    }
    let entry = &report.resolution.definitions[definition.0 as usize];
    let key = entry
        .node
        .ok_or_else(|| error("reexported functions are outside the scalar metadata profile"))?;
    let ast = &report.package.unit(key.file).configured.ast;
    let Syntax::Item(index) = ast.node(key.node).kind else {
        return Err(error("missing export declaration"));
    };
    let item = ast.items[index as usize];
    let ItemKind::Function { flags, body, .. } = item.kind else {
        return Err(error(
            "metadata exports require ordinary defined Rust functions",
        ));
    };
    if flags.constant
        || flags.asynchronous
        || facts
            .generics
            .parameters
            .get(&definition)
            .is_some_and(|parameters| {
                parameters
                    .iter()
                    .any(|parameter| !matches!(parameter.argument, Argument::Lifetime(_)))
            })
    {
        return Err(error(
            "const, async and generic exports are outside the scalar metadata profile",
        ));
    }
    if facts
        .lifetimes
        .functions
        .get(&definition)
        .is_some_and(|scheme| {
            scheme.captures != 0 || !scheme.early.is_empty() || !scheme.bounds.is_empty()
        })
        || facts
            .generics
            .predicates
            .get(&definition)
            .is_some_and(|predicates| !predicates.is_empty())
    {
        return Err(error(
            "early lifetime parameters and explicit bounds are outside the metadata profile",
        ));
    }
    let ty =
        facts.definitions[definition.0 as usize].ok_or_else(|| error("missing export type"))?;
    let Some(Type::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err(error("missing export callable"));
    };
    if body.is_none() {
        let imported = facts
            .foreign
            .as_deref()
            .and_then(|registry| registry.function(definition))
            .filter(|record| record.signature == signature && record.linkage == Kind::Import)
            .ok_or_else(|| {
                error("bodyless metadata functions require a checked foreign declaration")
            })?;
        return Ok(Callable {
            signature,
            safety: imported.safety,
            linkage: Kind::Import,
        });
    }
    Ok(Callable {
        signature,
        safety: flags.safety,
        linkage: Kind::Export,
    })
}
pub(super) fn scalar(facts: &Declarations, ty: TypeId) -> Result<Scalar, Error> {
    if ty == TypeId::UNIT {
        return Ok(Scalar::Unit);
    }
    Ok(match facts.types.kind(ty) {
        Some(Type::Primitive(primitive)) => match primitive {
            Primitive::Bool => Scalar::Bool,
            Primitive::Char => Scalar::Char,
            Primitive::U8 => Scalar::U8,
            Primitive::U16 => Scalar::U16,
            Primitive::U32 => Scalar::U32,
            Primitive::U64 => Scalar::U64,
            Primitive::U128 => Scalar::U128,
            Primitive::Usize => Scalar::Usize,
            Primitive::I8 => Scalar::I8,
            Primitive::I16 => Scalar::I16,
            Primitive::I32 => Scalar::I32,
            Primitive::I64 => Scalar::I64,
            Primitive::I128 => Scalar::I128,
            Primitive::Isize => Scalar::Isize,
            Primitive::F32 => Scalar::F32,
            Primitive::F64 => Scalar::F64,
            Primitive::Str => return Err(error("str is outside the scalar metadata profile")),
        },
        _ => return Err(error("exposed type is outside the scalar metadata profile")),
    })
}
