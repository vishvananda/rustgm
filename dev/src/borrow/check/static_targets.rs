//! Solved caller equalities select callable identities without strengthening a callee proof.
use crate::support::hash::{HashMap, HashSet};
use crate::{
    body::ir::Body,
    borrow::regions,
    types::{
        ItemId, Kind, RegionId, Store, TypeId, arguments::Argument, instance::Instance,
        relations::RegionEqualities, substitute::Bound,
    },
};

pub(super) fn prepare(proof: &regions::Body) -> Result<Option<Box<Bound>>, &'static str> {
    prepare_targets(proof, None)
}

/// Preserve proven universal identities when a constructor opens its inputs.
/// Other runtime targets use their individually selected identities.
pub(super) fn prepare_constructors(
    types: &Store,
    body: &Body,
    is_constructor: &dyn Fn(ItemId) -> bool,
    proof: &regions::Body,
) -> Result<Option<Box<Bound>>, &'static str> {
    if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance {
        return Err("constructor proof belongs to another caller body or instance");
    }
    if proof.identities.universals().len() < 2
        || (proof.solution.obligations == 0 && proof.solution.assumption_count() == 0)
        || !body.values.iter().any(|value| {
            matches!(types.kind(value.ty), Some(Kind::FunctionItem { item, .. })
                if is_constructor(item))
        })
    {
        return Ok(None);
    }
    prepare_equivalences::<false>(proof)
}

/// A captured callback can use the same lifetime in its explicit input and
/// result even where inference gave them different names. Preserve that identity
/// when the native callee opens the call anew.
pub(super) fn prepare_captured_calls(
    types: &Store,
    body: &Body,
    proof: &regions::Body,
) -> Result<Option<Box<Bound>>, &'static str> {
    if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance {
        return Err("captured call proof belongs to another caller body or instance");
    }
    if proof.solution.obligations != 0 || proof.solution.assumption_count() != 0 {
        for value in &body.values {
            if captured_call(types, value.ty) {
                return prepare_equivalences::<true>(proof);
            }
        }
    }
    Ok(None)
}

fn captured_call(types: &Store, ty: TypeId) -> bool {
    let Some(Kind::FunctionItem { arguments, .. }) = types.kind(ty) else {
        return false;
    };
    if types.contains_regions(ty) != Some(true) {
        return false;
    }
    let Some(arguments) = types.arguments(arguments) else {
        return false;
    };
    for argument in arguments {
        if let Argument::Type(ty) = argument
            && let Some(Kind::Closure {
                environment,
                signature,
                ..
            }) = types.kind(ty)
            && environment != TypeId::UNIT
            && matches!(types.kind(signature), Some(Kind::Function { parameters, .. })
                if types.list(parameters).is_some_and(|inputs| !inputs.is_empty()))
        {
            return true;
        }
    }
    false
}

fn prepare_equivalences<const INFERRED: bool>(
    proof: &regions::Body,
) -> Result<Option<Box<Bound>>, &'static str> {
    let equalities = RegionEqualities::proven::<INFERRED>(&proof.solution, &proof.identities)
        .map_err(|_| "callable lifetime equivalence proof exhausted")?;
    if equalities.is_empty() {
        return Ok(None);
    }
    prepare_targets(proof, Some(&equalities))
}

fn prepare_targets(
    proof: &regions::Body,
    equalities: Option<&RegionEqualities>,
) -> Result<Option<Box<Bound>>, &'static str> {
    let required: HashSet<_> = proof
        .solution
        .required_static()
        .map_err(|_| "static callable proof query exhausted")?
        .into_iter()
        .collect();
    if required.is_empty() && equalities.is_none_or(RegionEqualities::is_empty) {
        return Ok(None);
    }
    fold(proof, |original, region| {
        if required.contains(&region) {
            Ok(RegionId::STATIC)
        } else if let Some(equalities) = equalities {
            equalities
                .canonical(original, 0)
                .map_err(|_| "invalid callable lifetime representative")
        } else {
            Ok(original)
        }
    })
}

fn fold(
    proof: &regions::Body,
    replace: impl Fn(RegionId, regions::graph::Region) -> Result<RegionId, &'static str>,
) -> Result<Option<Box<Bound>>, &'static str> {
    let mut replacements = HashMap::default();
    let mut changed = false;
    let mut insert = |original, region| {
        let replacement = replace(original, region)?;
        changed |= replacement != original;
        replacements.insert(original, replacement);
        Ok::<_, &'static str>(())
    };
    for (original, region) in proof.identities.variables() {
        insert(original, region)?;
    }
    for (index, region) in proof.identities.universals().enumerate() {
        let original = RegionId::bound(
            0,
            u16::try_from(index).map_err(|_| "too many callable proof universals")?,
        )?;
        insert(original, region)?;
    }
    Ok(changed.then(|| Box::new(Bound::restore_regions(replacements))))
}

pub(super) type ConstructorFold<'a> = (&'a dyn Fn(ItemId) -> bool, &'a mut Bound);

pub(super) struct Folds<'a> {
    pub(super) statics: Option<&'a mut Bound>,
    pub(super) constructors: Option<ConstructorFold<'a>>,
    pub(super) captured_calls: Option<&'a mut Bound>,
}

impl Folds<'_> {
    fn for_item(&mut self, types: &Store, ty: TypeId, item: ItemId) -> Option<&mut Bound> {
        if let Some((is_constructor, fold)) = &mut self.constructors
            && is_constructor(item)
        {
            Some(fold)
        } else if let Some(fold) = &mut self.captured_calls
            && captured_call(types, ty)
        {
            Some(fold)
        } else {
            self.statics.as_deref_mut()
        }
    }
}

pub(super) fn resolve(
    types: &mut Store,
    body: &mut Body,
    folds: Folds<'_>,
) -> Result<(), &'static str> {
    if body.has_resolved_callables() {
        return Err("callable targets already belong to a completed proof");
    }
    let mut work = 0;
    let targets = select(types, body, folds, usize::MAX, &mut work)?;
    if !targets.is_empty() {
        body.install_callable_targets(targets)?;
    }
    Ok(())
}

/// The caller supplies the exact solved regions only after completing its loans.
/// No solution means no replacement; structural item arguments grant no proof.
pub(crate) struct Solved<'body> {
    pub body: &'body Body,
    pub targets: HashMap<Instance, Instance>,
    pub storage: Option<Box<Bound>>,
}
pub(crate) fn solved<'body>(
    types: &mut Store,
    body: &'body Body,
    proof: &regions::Body,
    bytes: usize,
    work: &mut usize,
) -> Result<Solved<'body>, &'static str> {
    if proof.key != body.key() || proof.source != body.source || proof.instance != body.instance {
        return Err("solved callable regions belong to another actual body or instance");
    }
    let Some(mut fold) = prepare(proof)? else {
        return Ok(Solved {
            body,
            targets: HashMap::default(),
            storage: None,
        });
    };
    if body.has_resolved_callables() {
        return Err("callable targets already belong to a completed proof");
    }
    let folds = Folds {
        statics: Some(&mut fold),
        constructors: None,
        captured_calls: None,
    };
    let targets = select(types, body, folds, bytes, work)?;
    Ok(Solved {
        body,
        targets,
        storage: Some(fold),
    })
}

fn select(
    types: &mut Store,
    body: &Body,
    mut folds: Folds<'_>,
    bytes: usize,
    work: &mut usize,
) -> Result<HashMap<Instance, Instance>, &'static str> {
    let mut targets = HashMap::default();
    for value in &body.values {
        *work = work.saturating_add(1);
        let Some(Kind::FunctionItem { item, .. }) = types.kind(value.ty) else {
            continue;
        };
        let Some(fold) = folds.for_item(types, value.ty, item) else {
            continue;
        };
        let before = fold.steps;
        let selected = fold.apply(types, value.ty);
        *work = work.saturating_add(fold.steps.saturating_sub(before));
        let selected = selected?;
        if selected == value.ty {
            continue;
        }
        let original = Instance::new(types, value.ty)?;
        if !targets.contains_key(&original) {
            if crate::support::retained::new_map::<Instance, Instance>(targets.len() + 1) > bytes {
                return Err("solved callable targets exceed snapshot storage budget");
            }
            targets
                .try_reserve(1)
                .map_err(|_| "solved callable target allocation failed")?;
            if crate::support::retained::map(&targets) > bytes {
                return Err("solved callable target payload exceeds snapshot storage budget");
            }
        }
        targets.insert(original, Instance::new(types, selected)?);
    }
    Ok(targets)
}
