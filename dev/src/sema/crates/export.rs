//! Public scalar exports consume resolved bindings and completed signature facts.
mod named_values;
use super::error;
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, Binding, DefId, DefKind, Origin, ScopeId},
        package::Error,
    },
    sema::{foreign, model::Declarations},
    support::{
        crates::{
            CrateIdentity, ExternalDef, ExternalKind, Function, Interface, Scalar, Signature,
            Symbol,
        },
        linkage::{Linkage as Kind, Naming},
    },
    syntax::{
        ast::Kind as Syntax,
        item::{ItemKind, Safety},
    },
    types::{Kind as Type, Primitive, TypeId, callable::Convention},
};

pub struct Exports {
    pub interface: Interface,
    pub roots: Vec<DefId>,
}
/// Publish the initial scalar profile after ordinary semantic checking.
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
    let mut work = vec![(report.resolution.module_scopes[0], Vec::<String>::new())];
    let mut visits = 0usize;
    while let Some((scope, path)) = work.pop() {
        let members = public_members(report, scope);
        if !path.is_empty() && members.is_empty() {
            return Err(error(
                "empty public modules are outside the scalar metadata profile",
            ));
        }
        for binding in members {
            if binding.origin != Origin::Direct {
                return Err(error(
                    "public reexports are outside the scalar metadata profile",
                ));
            }
            let definition = binding.target;
            visits += 1;
            if visits > 100_000 || path.len() >= 64 {
                return Err(error("metadata export traversal exceeds profile budget"));
            }
            let entry = &report.resolution.definitions[definition.0 as usize];
            let mut child = path.clone();
            child.push(report.package.names.get(entry.name).to_owned());
            match entry.kind {
                DefKind::Module => work.push((
                    entry
                        .members
                        .ok_or_else(|| error("missing public module scope"))?,
                    child,
                )),
                DefKind::Function => {
                    functions.push(function(report, facts, definition, &identity, child)?);
                    roots.push(definition);
                }
                DefKind::Const => {
                    constants.push(named_values::export(facts, definition, &identity, child)?);
                }
                _ => {
                    return Err(error(
                        "exposed declaration is outside the scalar metadata profile",
                    ));
                }
            }
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
    let interface = Interface::with_constants(identity.clone(), dependencies, functions, constants)
        .map_err(error)?;
    let linkage = facts.crates.get_or_insert_default();
    linkage.set_identity(identity).map_err(error)?;
    roots.sort_unstable_by_key(|definition| definition.0);
    Ok(Exports { interface, roots })
}
fn public_members(report: &Report, scope: ScopeId) -> Vec<Binding> {
    let frame = &report.resolution.scopes[scope.0 as usize];
    frame
        .order
        .iter()
        .filter_map(|key| {
            let binding = frame.bindings.get(key)?;
            (binding.access == Access::Public).then_some(*binding)
        })
        .collect()
}
fn function(
    report: &Report,
    facts: &mut Declarations,
    definition: DefId,
    identity: &CrateIdentity,
    path: Vec<String>,
) -> Result<Function, Error> {
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
                linkage: Kind::Export,
            },
        )
        .map_err(error)?;
    let external =
        ExternalDef::new(identity.clone(), path, ExternalKind::Function).map_err(error)?;
    facts
        .crates
        .get_or_insert_default()
        .symbols
        .insert(definition, external.rust_symbol());
    Function::new(external, callable.wire, symbol).map_err(error)
}
struct Callable {
    signature: TypeId,
    wire: Signature,
    safety: Safety,
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
    let ItemKind::Function {
        flags,
        body: Some(_),
        ..
    } = item.kind
    else {
        return Err(error(
            "metadata exports require ordinary defined Rust functions",
        ));
    };
    if flags.constant
        || flags.asynchronous
        || ast
            .list(item.generics)
            .iter()
            .any(|node| report.package.unit(key.file).configured.active[node.0 as usize])
    {
        return Err(error(
            "const, async and generic exports are outside the scalar metadata profile",
        ));
    }
    let ty =
        facts.definitions[definition.0 as usize].ok_or_else(|| error("missing export type"))?;
    let Some(Type::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
        return Err(error("missing export callable"));
    };
    let Some(Type::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_,
        convention: Convention::RUST,
    }) = facts.types.kind(signature)
    else {
        return Err(error(
            "export calling convention is outside the scalar metadata profile",
        ));
    };
    let parameters = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("missing export parameters"))?
        .iter()
        .map(|&ty| scalar(facts, ty))
        .collect::<Result<_, _>>()?;
    let signature_wire =
        Signature::new(parameters, scalar(facts, result)?, unsafe_).map_err(error)?;
    Ok(Callable {
        signature,
        wire: signature_wire,
        safety: flags.safety,
    })
}
fn scalar(facts: &Declarations, ty: TypeId) -> Result<Scalar, Error> {
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
