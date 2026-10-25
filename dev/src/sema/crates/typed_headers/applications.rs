//! Derive one selected signature/goal template per exact complete application.
use super::{Declarations, Error, Headers, error, roots::Mapped};
use crate::{
    resolve::{crates::Imported, model::DefId},
    support::crates::{CrateIdentity, generics as g},
    types::{
        self, ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{
            ParameterKind,
            predicates::{Bound as Goal, Predicate},
        },
        substitute::Bound,
    },
};
pub(crate) struct Application {
    pub target: DefId,
    pub item: TypeId,
    pub signature: TypeId,
    pub arguments: Vec<TypeId>,
    pub predicates: Vec<Predicate>,
}
pub(super) fn import(
    imported: &Imported,
    facts: &mut Declarations,
    owner: &CrateIdentity,
    payload: &g::Payload,
    mapped: &mut Mapped,
    headers: &mut Headers,
) -> Result<Vec<Application>, Error> {
    let rows = payload.legacy_rows().map_err(error)?;
    let Some(calls) = &rows.calls else {
        return Ok(Vec::new());
    };
    headers.reserve(
        calls
            .applications
            .len()
            .saturating_mul(size_of::<Application>()),
    )?;
    let mut result = Vec::with_capacity(calls.applications.len());
    for (index, raw) in calls.applications.iter().enumerate() {
        headers.charge(1)?;
        payload
            .application_owner(g::ApplicationRef(
                u32::try_from(index).map_err(|_| error("application index overflow"))?,
            ))
            .map_err(error)?;
        let target = imported
            .generic_definition(owner, raw.target)
            .map_err(error)?;
        let header = headers
            .entries
            .get(&target)
            .ok_or_else(|| error("application target has no completed canonical header"))?;
        let callable = header.callable;
        let source = &rows.functions[raw.target.0 as usize];
        let args = raw.arguments.slice(&calls.arguments).map_err(error)?;
        let roots = args.iter().map(|r| mapped.nodes[r.0 as usize]);
        let application = derive(facts, target, callable, source, rows, roots, headers)?;
        mapped.nodes.push(application.item);
        mapped.nodes.push(application.signature);
        facts.crates.get_or_insert_default().import_work.type_nodes += 2;
        headers.observed.vector(&application.arguments);
        headers.observed.vector(&application.predicates);
        result.push(application);
    }
    headers.observed.vector(&result);
    Ok(result)
}
fn derive(
    facts: &mut Declarations,
    target: DefId,
    callable: TypeId,
    source: &g::Function,
    rows: &g::Rows,
    arguments: impl Iterator<Item = TypeId>,
    headers: &mut Headers,
) -> Result<Application, Error> {
    let Some(Kind::Function {
        bound_regions: 0,
        parameters,
        result,
        unsafe_: false,
        convention,
    }) = facts.types.kind(callable)
    else {
        return Err(error(
            "application target has no exact safe zero-region callable",
        ));
    };
    let count = (source.parameters.end - source.parameters.start) as usize;
    let clauses = source.clauses.slice(&rows.clauses).map_err(error)?;
    let roots = facts
        .types
        .list(parameters)
        .ok_or_else(|| error("application callable inputs disappeared"))?
        .len()
        + 1
        + clauses.len();
    let growth = Bound::parameter_leaf_allocation_bound(count, roots)
        .saturating_add(
            4 * count.saturating_mul(
                size_of::<TypeId>() + size_of::<(types::parameters::ParameterId, Argument)>(),
            ),
        )
        .saturating_add(4 * (count + clauses.len()).saturating_mul(size_of::<Predicate>()))
        .saturating_add(4 * roots.saturating_mul(size_of::<TypeId>()));
    headers.reserve(growth)?;
    let arguments: Vec<_> = arguments.collect();
    let (bindings, mut predicates) =
        bindings(facts, target, &arguments, count, clauses.len(), headers)?;
    let mut fold = Bound::parameters(&facts.types, &bindings).map_err(error)?;
    let outcome = (|| {
        let inputs = facts
            .types
            .list(parameters)
            .ok_or_else(|| error("application input list disappeared"))?
            .to_vec();
        let mut mapped = Vec::with_capacity(inputs.len());
        for ty in inputs {
            headers.charge(1)?;
            mapped.push(fold.apply(&mut facts.types, ty).map_err(error)?);
        }
        let result = fold.apply(&mut facts.types, result).map_err(error)?;
        for clause in clauses {
            headers.charge(1)?;
            predicates.push(clause_goal(rows, clause, &arguments)?);
        }
        let parameters = facts.types.intern_list(&mapped).map_err(error)?;
        let signature = facts
            .types
            .intern(Kind::Function {
                bound_regions: 0,
                parameters,
                result,
                unsafe_: false,
                convention,
            })
            .map_err(error)?;
        let args: Vec<_> = arguments.iter().copied().map(Argument::Type).collect();
        let args = facts.types.intern_arguments(&args).map_err(error)?;
        let item = facts
            .types
            .intern(Kind::FunctionItem {
                item: ItemId(target.0),
                arguments: args,
                signature,
            })
            .map_err(error)?;
        Ok(Application {
            target,
            item,
            signature,
            arguments,
            predicates,
        })
    })();
    let (bytes, visits) = fold.retained_allocation();
    headers.charge(fold.steps.saturating_add(visits))?;
    if bytes > growth {
        return Err(error("application fold exceeded its allocation preflight"));
    }
    outcome
}

type Bindings = Vec<(types::parameters::ParameterId, Argument)>;
fn bindings(
    facts: &Declarations,
    target: DefId,
    arguments: &[TypeId],
    count: usize,
    clauses: usize,
    headers: &mut Headers,
) -> Result<(Bindings, Vec<Predicate>), Error> {
    let mut bindings = Vec::with_capacity(count);
    let mut predicates = Vec::with_capacity(count + clauses);
    for (index, formal) in facts.generics.parameters_in_scope(target).enumerate() {
        headers.charge(1)?;
        let Argument::Type(ty) = formal.argument else {
            return Err(error("application formal is not a type"));
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err(error("application formal is not rigid"));
        };
        let parameter = facts
            .types
            .parameter(id)
            .ok_or_else(|| error("application formal disappeared"))?;
        let ParameterKind::Type { sized } = parameter.kind else {
            return Err(error("application formal changed kind"));
        };
        let actual = *arguments
            .get(index)
            .ok_or_else(|| error("application omitted a complete formal"))?;
        if parameter.owner != ItemId(target.0)
            || parameter.index as usize != index
            || facts.types.escaping_binders(actual) != Some(0)
        {
            return Err(error(
                "application binding belongs to a foreign owner or binder",
            ));
        }
        if sized {
            predicates.push(Predicate {
                ty: actual,
                bound: Goal::Sized,
            });
        }
        bindings.push((id, Argument::Type(actual)));
    }
    if bindings.len() != count || arguments.len() != count {
        return Err(error("application has incomplete actual arguments"));
    }
    Ok((bindings, predicates))
}

fn clause_goal(
    rows: &g::Rows,
    clause: &g::Clause,
    arguments: &[TypeId],
) -> Result<Predicate, Error> {
    let ty = match rows.types[clause.subject.0 as usize] {
        g::Type::Scalar(value) => super::super::scalar(value),
        g::Type::Parameter(reference) => {
            let g::Declaration::TypeParameter { ordinal, .. } =
                rows.declarations[reference.0 as usize]
            else {
                return Err(error("application clause parameter changed kind"));
            };
            arguments[ordinal as usize]
        }
        _ => return Err(error("application clause is outside the leaf profile")),
    };
    Ok(Predicate {
        ty,
        bound: match clause.bound {
            g::Bound::Copy(g::CopyIdentity::LegacyBuiltin) => Goal::Copy,
            g::Bound::Sized => Goal::Sized,
            g::Bound::Copy(_) => {
                return Err(error("application requires source trait transport"));
            }
        },
    })
}
