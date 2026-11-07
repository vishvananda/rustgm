//! Actual callable members retain their tuple ABI and source-owned Output identity.
use super::{
    CallTrait, Catalog, Declaration, Declarations, DefId, DefKind, Error, ItemKind, Kind, LangItem,
    Report, TypeId, function, item,
};
use crate::{
    sema::compiler_items::{
        Role,
        headers::{Flags, Trait},
    },
    support::constness::Constness,
    types::{arguments::Argument, callable::Convention, parameters::clauses::Clause},
};
#[cfg(test)]
#[path = "../../../../tests/invariants/invocation_roles.rs"]
mod tests;

/// Identity data only; no trait application or callable permission is proved here.
pub(crate) fn definition(facts: &Declarations, kind: CallTrait) -> Result<DefId, &'static str> {
    let role = match kind {
        CallTrait::FnOnce => LangItem::FnOnce,
        CallTrait::FnMut => LangItem::FnMut,
        CallTrait::Fn => LangItem::Fn,
    };
    // The neutral identity also answers for a consumer's imported family.
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source invocation requires actual source roles")?
        .validated_role(role)?
        .ok_or("source invocation requires its actual callable role")?;
    if facts.associated.callable_definition(kind) != Some(declaration.definition) {
        return Err("source invocation changed its actual callable declaration");
    }
    Ok(declaration.definition)
}
/// The validated `FnOnce::Output` declaration of the completed family.
pub(crate) fn output(facts: &Declarations) -> Result<DefId, &'static str> {
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source invocation requires actual source roles")?
        .validated_role(LangItem::FnOnceOutput)?
        .ok_or("source invocation requires its actual callable Output")?;
    if facts.associated.callable_output() != Some(declaration.definition) {
        return Err("source invocation changed its actual callable Output");
    }
    Ok(declaration.definition)
}

pub(super) fn validate(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    role: LangItem,
    self_type: TypeId,
) -> Result<(CallTrait, DefId), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let (kind, name) = match role {
        LangItem::FnOnce => (CallTrait::FnOnce, "call_once"),
        LangItem::FnMut => (CallTrait::FnMut, "call_mut"),
        LangItem::Fn => (CallTrait::Fn, "call"),
        _ => return Err(error("invalid callable source role")),
    };
    let tuple = catalog
        .language(LangItem::Tuple)
        .ok_or_else(|| error("source callable requires actual Tuple"))?;
    let output = catalog
        .language(LangItem::FnOnceOutput)
        .ok_or_else(|| error("source callable requires actual FnOnce Output"))?;
    let once = catalog
        .language(LangItem::FnOnce)
        .ok_or_else(|| error("source callable requires actual FnOnce"))?;
    if output.parent != Some(once.definition)
        || report.resolution.definitions[output.definition.0 as usize].kind != DefKind::TypeAlias
    {
        return Err(error(
            "source callable Output has a different owner or member kind",
        ));
    }
    let [parameter] = facts
        .generics
        .parameters
        .get(&declaration.definition)
        .map_or(&[][..], Vec::as_slice)
    else {
        return Err(error("source callable requires its one Args parameter"));
    };
    let Argument::Type(args) = parameter.argument else {
        return Err(error("source callable Args must be a type"));
    };
    let predicates = || {
        facts
            .generics
            .predicates
            .get(&declaration.definition)
            .into_iter()
            .flatten()
            .map(|p| p.value)
    };
    if !has_premise(
        facts,
        predicates(),
        args,
        tuple.definition,
        Constness::Ordinary,
        None,
    ) {
        return Err(error(
            "source callable Args requires its actual Tuple obligation",
        ));
    }
    validate_parent(report, facts, catalog, declaration, kind, self_type, args)?;
    let owner = facts
        .associated
        .owner(declaration.definition)
        .ok_or_else(|| error("missing callable owner"))?;
    let mut methods = facts
        .associated
        .owner_members(owner)
        .ok_or_else(|| error("missing callable members"))?
        .iter()
        .filter_map(|m| {
            let d = &report.resolution.definitions[m.0 as usize];
            (d.kind == DefKind::Function && report.package.names.get(d.name) == name).then_some(*m)
        });
    let method = methods
        .next()
        .ok_or_else(|| error("source callable lacks its pinned method"))?;
    if methods.next().is_some() {
        return Err(error("source callable has duplicate methods"));
    }
    validate_method(
        report,
        facts,
        declaration,
        method,
        kind,
        (self_type, args),
        (once.definition, output.definition),
    )?;
    Ok((kind, method))
}

fn validate_parent(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    declaration: Declaration,
    kind: CallTrait,
    self_type: TypeId,
    args: TypeId,
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let ItemKind::Trait { bounds, .. } = item(report, declaration)?.kind else {
        return Err(error("missing source callable syntax"));
    };
    let expected = match kind {
        CallTrait::FnOnce => None,
        CallTrait::FnMut => Some((LangItem::FnOnce, Constness::Ordinary)),
        CallTrait::Fn => Some((LangItem::FnMut, Constness::Conditional)),
    };
    if report
        .package
        .unit(declaration.origin.file)
        .configured
        .ast
        .list(bounds)
        .len()
        != usize::from(expected.is_some())
    {
        return Err(error("source callable requires its exact pinned hierarchy"));
    }
    let Some((role, effect)) = expected else {
        return Ok(());
    };
    let parent = catalog
        .language(role)
        .ok_or_else(|| error("source callable parent is missing"))?;
    let predicates = facts
        .generics
        .predicates
        .get(&declaration.definition)
        .into_iter()
        .flatten()
        .map(|p| p.value);
    if !has_premise(
        facts,
        predicates,
        self_type,
        parent.definition,
        effect,
        Some(args),
    ) {
        return Err(error(
            "source callable hierarchy lost its Args or const qualification",
        ));
    }
    Ok(())
}

fn validate_method(
    report: &Report,
    facts: &Declarations,
    declaration: Declaration,
    method: DefId,
    kind: CallTrait,
    inputs: (TypeId, TypeId),
    identities: (DefId, DefId),
) -> Result<(), Error> {
    let error = |message| Declarations::error(report, declaration.origin, message);
    let f =
        function(facts, method).ok_or_else(|| error("source callable method is not prepared"))?;
    method_contract(facts, f.signature, kind, inputs, identities).map_err(error)?;
    if facts
        .generics
        .parameters
        .get(&method)
        .is_some_and(|p| !p.is_empty())
    {
        return Err(error(
            "source callable method differs from its exact receiver/Args/Output contract",
        ));
    }
    Ok(())
}

/// One trait premise `subject: Item[<argument>]` with exactly this effect.
fn has_premise(
    facts: &Declarations,
    clauses: impl IntoIterator<Item = Clause>,
    subject: TypeId,
    item: DefId,
    effect: Constness,
    argument: Option<TypeId>,
) -> bool {
    clauses.into_iter().any(|clause| {
        let Clause::Predicate(p) = clause else {
            return false;
        };
        p.ty == subject
            && p.bound.trait_parts().is_some_and(|(id, actual)| {
                actual == effect
                    && facts.types.trait_application(id).is_some_and(|a| {
                        a.item.0 == item.0
                            && facts.types.arguments(a.arguments).is_some_and(|mut a| {
                                a.len() == usize::from(argument.is_some())
                                    && argument
                                        .is_none_or(|ty| a.next() == Some(Argument::Type(ty)))
                            })
                    })
            })
    })
}

/// The call method's exact safe rust-call receiver, tuple `Args` and
/// `<Self as FnOnce<Args>>::Output` result, shared by source and imports.
fn method_contract(
    facts: &Declarations,
    signature: TypeId,
    kind: CallTrait,
    (self_type, args): (TypeId, TypeId),
    (once, output): (DefId, DefId),
) -> Result<(), &'static str> {
    let Some(Kind::Function {
        bound_regions,
        parameters,
        result,
        unsafe_: false,
        convention: Convention::RUST_CALL,
    }) = facts.types.kind(signature)
    else {
        return Err("source callable requires its safe rust-call method signature");
    };
    let [receiver, actual_args] = facts
        .types
        .list(parameters)
        .ok_or("missing callable inputs")?
    else {
        return Err("source callable requires receiver and tuple arguments");
    };
    let valid_receiver = if kind == CallTrait::FnOnce {
        *receiver == self_type
    } else {
        matches!(facts.types.kind(*receiver), Some(Kind::Reference { mutable, referent, region })
            if referent == self_type && mutable == (kind == CallTrait::FnMut) && region.bound_parts() == Some((0, 0)))
    };
    let valid_result = matches!(facts.types.kind(result), Some(Kind::Projection(p))
        if p.item.0 == output.0 && p.subject == self_type
            && facts.types.arguments(p.arguments).is_some_and(|a| a.len() == 0)
            && facts.types.trait_application(p.trait_application).is_some_and(|a|
                a.item.0 == once.0 && facts.types.arguments(a.arguments)
                    .is_some_and(|mut a| a.len() == 1 && a.next() == Some(Argument::Type(args)))));
    if bound_regions != u32::from(kind != CallTrait::FnOnce)
        || !valid_receiver
        || *actual_args != args
        || !valid_result
    {
        return Err("source callable method differs from its exact receiver/Args/Output contract");
    }
    Ok(())
}

/// Version thirteen: an imported Tuple marker and callable family pass the
/// same declaration contract as the provider's source roles, read from the
/// imported headers. It returns each admitted role, call method and Output.
pub(super) fn imported(
    report: &Report,
    facts: &Declarations,
    catalog: &Catalog,
    steps: &mut usize,
) -> Result<Imported, &'static str> {
    let mut admitted = Imported::default();
    let Some(once) = catalog.role(LangItem::FnOnce)? else {
        if [LangItem::FnMut, LangItem::Fn, LangItem::FnOnceOutput]
            .into_iter()
            .map(|role| catalog.role(role))
            .any(|role| role.is_ok_and(|role| role.is_some()))
        {
            return Err("imported callable family requires its actual FnOnce");
        }
        return Ok(admitted);
    };
    let tuple = catalog
        .role(LangItem::Tuple)?
        .ok_or("imported callable requires its actual Tuple")?;
    let output = catalog
        .role(LangItem::FnOnceOutput)?
        .ok_or("imported callable requires its actual FnOnce Output")?;
    if output.parent != Some(once.definition) {
        return Err("imported callable Output belongs to another role parent");
    }
    if facts.associated.trait_owner(output.definition) != Some(once.definition)
        || report
            .resolution
            .definitions
            .get(output.definition.0 as usize)
            .is_none_or(|d| d.kind != DefKind::TypeAlias)
    {
        return Err("imported callable Output has a different owner or member kind");
    }
    for (role, kind, name) in [
        (LangItem::FnOnce, CallTrait::FnOnce, "call_once"),
        (LangItem::FnMut, CallTrait::FnMut, "call_mut"),
        (LangItem::Fn, CallTrait::Fn, "call"),
    ] {
        let Some(declaration) = catalog.role(role)? else {
            continue;
        };
        let family = Family {
            tuple: tuple.definition,
            once: once.definition,
            output: output.definition,
        };
        let method = imported_role(
            report,
            facts,
            (declaration, kind, name),
            family,
            &admitted,
            steps,
        )?;
        admitted.roles[kind as usize] = Some(declaration.definition);
        admitted.methods[kind as usize] = Some(method);
    }
    admitted.output = Some(output.definition);
    Ok(admitted)
}

/// The family's own identities shared by every role's admission.
#[derive(Clone, Copy)]
struct Family {
    tuple: DefId,
    once: DefId,
    output: DefId,
}

/// One imported role: its header profile, Args and Tuple obligation, written
/// hierarchy and call method contract. Returns the admitted call method.
fn imported_role(
    report: &Report,
    facts: &Declarations,
    (declaration, kind, name): (Role, CallTrait, &str),
    family: Family,
    admitted: &Imported,
    steps: &mut usize,
) -> Result<DefId, &'static str> {
    facts.associated.require_imported_role(declaration)?;
    let header = Trait::current(report, facts, declaration, steps)?;
    header.profile(
        &facts.types,
        Flags {
            unsafe_: false,
            constant: true,
            auto_: false,
        },
        1,
    )?;
    *steps = steps
        .saturating_add(header.inputs.len())
        .saturating_add(header.members.len());
    let [parameter] = header.parameters else {
        return Err("imported callable requires its one Args parameter");
    };
    let Argument::Type(args) = parameter.argument else {
        return Err("imported callable Args must be a type");
    };
    let parent = match kind {
        CallTrait::FnOnce => None,
        CallTrait::FnMut => Some((
            admitted.roles[CallTrait::FnOnce as usize],
            Constness::Ordinary,
        )),
        CallTrait::Fn => Some((
            admitted.roles[CallTrait::FnMut as usize],
            Constness::Conditional,
        )),
    };
    let inputs = || header.inputs.iter().map(|input| input.value);
    if header.written.parameters != 1
        || header.members.len() != if kind == CallTrait::FnOnce { 2 } else { 1 }
        || header.written.supertraits != usize::from(parent.is_some())
    {
        return Err("imported callable changed its written parameters, members or hierarchy");
    }
    if !has_premise(
        facts,
        inputs(),
        args,
        family.tuple,
        Constness::Ordinary,
        None,
    ) {
        return Err("imported callable Args lost its actual Tuple obligation");
    }
    if parent.is_some_and(|(parent, effect)| {
        parent.is_none_or(|parent| {
            !has_premise(
                facts,
                inputs(),
                header.self_type,
                parent,
                effect,
                Some(args),
            )
        })
    }) {
        return Err("imported callable hierarchy lost its Args or const qualification");
    }
    let method = call_member(report, facts, &header, name, family.output)?;
    let function = facts
        .declaration_header(method)?
        .ok_or("imported callable has no completed method header")?;
    if function.body.is_some()
        || facts
            .generics
            .parameters
            .get(&method)
            .is_some_and(|p| !p.is_empty())
    {
        return Err("imported callable method requires its required nongeneric profile");
    }
    method_contract(
        facts,
        function.signature,
        kind,
        (header.self_type, args),
        (family.once, family.output),
    )?;
    crate::sema::compiler_items::operator_contract::imported_item(
        facts,
        method,
        function.signature,
    )?;
    Ok(method)
}

/// The admitted imported family: role definitions and call methods indexed
/// by `CallTrait`, and `FnOnce::Output`.
#[derive(Default)]
pub(super) struct Imported {
    pub roles: [Option<DefId>; 3],
    pub methods: [Option<DefId>; 3],
    pub output: Option<DefId>,
}

/// Exactly the role's call method, and `FnOnce::Output`, owned by the header.
fn call_member(
    report: &Report,
    facts: &Declarations,
    header: &Trait<'_>,
    name: &str,
    output: DefId,
) -> Result<DefId, &'static str> {
    let mut method = None;
    for &member in header.members {
        let definition = report
            .resolution
            .definitions
            .get(member.0 as usize)
            .ok_or("imported callable member has no actual definition")?;
        if facts.associated.trait_owner(member) != Some(header.definition) {
            return Err("imported callable member belongs to another actual owner");
        }
        match definition.kind {
            DefKind::TypeAlias if member == output => {}
            DefKind::Function
                if report.package.names.get(definition.name) == name && method.is_none() =>
            {
                method = Some(member);
            }
            _ => return Err("imported callable changed its actual member profile"),
        }
    }
    method.ok_or("imported callable lacks its pinned method")
}
