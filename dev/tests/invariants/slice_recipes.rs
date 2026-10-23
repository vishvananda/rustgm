//! Slice recipes share one selected element, independently of runtime length.
use super::*;
use crate::{
    body, sema, test_source,
    types::{Kind, TypeId},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn facts() -> Result<(crate::resolve::expansion::Report, sema::model::Declarations)> {
    let report = test_source::resolve(
        "struct D;impl Drop for D{fn drop(&mut self){}}fn f(value:D){}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    Ok((report, facts))
}
#[test]
fn source_slice_recipes_share_width_and_reuse_work_with_explicit_dense_baseline() -> Result {
    let mut baseline = None;
    for noise in [0, 8192] {
        let (report, mut facts) = facts()?;
        let program = body::build(&report, &facts)?;
        let methods = &program.drops.as_ref().ok_or("methods")?.methods;
        let leaf = methods[0].target;
        for length in 0..noise {
            facts.types.intern(Kind::Array {
                element: TypeId::UNIT,
                length,
            })?;
        }
        let mut roots = Vec::new();
        for width in [8, 32, 128] {
            let fields = facts.types.intern_list(&vec![leaf; width])?;
            let element = facts.types.intern(Kind::Tuple(fields))?;
            roots.push((width, facts.types.intern(Kind::Slice(element))?));
        }
        let mut catalog =
            Cache::new(methods)?.attach(&facts.types, &facts.aggregates, &facts.members);
        let mut measured = Vec::new();
        for (width, root) in roots {
            let before = catalog.steps();
            let id = catalog.expose(root)?.ok_or("slice root")?;
            let recipe = catalog.get(id).ok_or("slice recipe")?;
            assert_eq!(recipe.ty, root);
            let super::super::Shape::Slice { element } = recipe.kind else {
                return Err("slice shape".into());
            };
            assert_ne!(element, id);
            assert_eq!(catalog.cache.storage.slice(&facts.types, id)?.1, element);
            let work = catalog.steps() - before;
            assert!(work <= 4 * width + 16);
            let steps = catalog.steps();
            let count = catalog.recipes().len();
            for _ in 0..128 {
                assert_eq!(catalog.expose(root)?, Some(id));
            }
            assert_eq!(catalog.steps(), steps);
            assert_eq!(catalog.recipes().len(), count);
            measured.push((work, count));
        }
        let cache = catalog.into_cache();
        println!(
            "noise{noise}: work/count={measured:?}; recipe-charge={} query-bytes={} entries-cap={} fields-cap={} pending-cap={}",
            cache.retained,
            cache.needs.retained_bytes(),
            cache.storage.entries.capacity(),
            cache.storage.fields.capacity(),
            cache.work.capacity()
        );
        if let Some(expected) = &baseline {
            assert_eq!(&measured, expected);
        } else {
            baseline = Some(measured);
        }
        assert_eq!(cache.storage.exposed.len(), 3);
    }
    println!(
        "Shape={} Recipe={} Cache={} Pending={} Body={}",
        size_of::<super::super::Shape>(),
        size_of::<super::super::Recipe>(),
        size_of::<Cache>(),
        size_of::<(TypeId, bool)>(),
        size_of::<body::ir::Body>()
    );
    Ok(())
}
#[test]
fn source_slice_recipes_retain_failed_budget_and_dropless_answers() -> Result {
    let (report, mut facts) = facts()?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let root = facts.types.intern(Kind::Slice(methods[0].target))?;
    let empty = facts.types.intern(Kind::Slice(TypeId::UNIT))?;
    let mut cache = Cache::new(methods)?;
    cache.steps = 999_999;
    let mut catalog = cache.attach(&facts.types, &facts.aggregates, &facts.members);
    assert!(catalog.expose(root).is_err());
    let steps = catalog.steps();
    for _ in 0..128 {
        assert!(catalog.expose(root).is_err());
    }
    assert_eq!(catalog.steps(), steps);
    let mut catalog = Cache::new(methods)?.attach(&facts.types, &facts.aggregates, &facts.members);
    assert_eq!(catalog.expose(empty)?, None);
    assert_eq!(catalog.cache.storage.exposed.get(&empty), Some(&None));
    Ok(())
}

#[test]
fn source_slice_recipe_rejects_wrong_element_owner_and_invented_method() -> Result {
    let (report, mut facts) = facts()?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let root = facts.types.intern(Kind::Slice(methods[0].target))?;
    let fields = facts.types.intern_list(&[methods[0].target])?;
    let other = facts.types.intern(Kind::Tuple(fields))?;
    let mut catalog = Cache::new(methods)?.attach(&facts.types, &facts.aggregates, &facts.members);
    let id = catalog.expose(root)?.ok_or("slice")?;
    let wrong_child = catalog.demand(other)?.ok_or("same-layout tuple")?;
    let original = *catalog.get(id).ok_or("slice recipe")?;
    let recipes = &mut catalog.cache.storage;
    recipes.entries[id.0 as usize].kind = super::super::Shape::Slice {
        element: wrong_child,
    };
    assert!(recipes.slice(&facts.types, id).is_err());
    recipes.entries[id.0 as usize].kind = super::super::Shape::Slice { element: id };
    assert!(recipes.slice(&facts.types, id).is_err());
    recipes.entries[id.0 as usize] = original;
    recipes.entries[id.0 as usize].method = Some(super::super::Method::Definition(DefId(u32::MAX)));
    assert!(recipes.slice(&facts.types, id).is_err());
    recipes.entries[id.0 as usize] = original;
    recipes.entries[id.0 as usize].ty = methods[0].target;
    assert!(recipes.slice(&facts.types, id).is_err());
    recipes.entries[id.0 as usize] = original;
    recipes.slice(&facts.types, id)?;
    Ok(())
}

#[test]
fn source_slice_roots_keep_full_identity_while_nominal_children_share_regions() -> Result {
    let report = test_source::resolve(
        "struct D<'a>(&'a u8);impl Drop for D<'_>{fn drop(&mut self){}}fn f<'a,'b>(first:&'a[D<'a>],second:&'b[D<'b>]){}",
        true,
    )?;
    let facts = sema::declarations::analyze(&report)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "f"
        })
        .ok_or("function")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(function.signature) else {
        return Err("signature".into());
    };
    let mut catalog = Cache::new(methods)?.attach(&facts.types, &facts.aggregates, &facts.members);
    let mut roots = Vec::new();
    let mut children = Vec::new();
    for &parameter in facts.types.list(parameters).ok_or("parameters")? {
        let Some(Kind::Reference { referent, .. }) = facts.types.kind(parameter) else {
            return Err("reference".into());
        };
        let id = catalog.expose(referent)?.ok_or("slice root")?;
        assert_eq!(catalog.get(id).ok_or("recipe")?.ty, referent);
        let (element, child) = catalog.cache.storage.slice(&facts.types, id)?;
        assert_ne!(referent, element);
        roots.push((referent, id));
        children.push(child);
    }
    assert_eq!(roots.len(), 2);
    assert_ne!(roots[0].0, roots[1].0);
    assert_ne!(roots[0].1, roots[1].1);
    assert_eq!(children[0], children[1]);
    Ok(())
}
