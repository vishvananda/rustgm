//! Source marker identity supplies logical T, while physical storage stays empty.
use crate::{
    test_source,
    types::{
        self, GenericArgs, ItemId, Kind, RegionId, Store, TypeId,
        aggregate::{Definitions, Repr, Variant},
        arguments::Argument,
        auto_traits::{AutoTrait, Obligation, Query, Rule, Step},
        parameters::{Parameter, ParameterKind},
        variance::{Variance, Variances},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PROVIDER: &str = "pa31/tests/spec/source-phantom/source-marker.rs";

#[test]
fn source_marker_constructor_clone_and_const_storage_use_real_headers() -> Result {
    let source = test_source::fixture(PROVIDER)?;
    let report = test_source::source_provider(&source, "phantom_source")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let declaration = facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .validated_language(crate::support::bindings::LangItem::PhantomData)?
        .ok_or("actual phantom role")?;
    assert_eq!(
        facts
            .aggregates
            .get(ItemId(declaration.definition.0))
            .ok_or("shape")?
            .kind,
        types::aggregate::Kind::PhantomData
    );
    assert_eq!(
        facts
            .lifetimes
            .variances
            .get(ItemId(declaration.definition.0)),
        Some(&[Variance::Covariant][..])
    );
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in [
        "phantom_size",
        "phantom_align",
        "phantom_drop",
        "phantom_clone",
    ] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-phantom");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn source_marker_rejects_changed_roles_formals_fields_and_lifetime_escape() -> Result {
    let source = test_source::fixture(PROVIDER)?;
    // Header opponents retain their genuine malformed declaration without also
    // referring to a unit constructor that they deliberately removed.
    let header = source
        .split("impl<T: PointeeSized> Copy for PhantomData<T> {}")
        .next()
        .ok_or("marker declaration prefix")?;
    for (name, changed, needle) in [
        (
            "fields",
            header.replace(
                "pub struct PhantomData<T: PointeeSized>;",
                "pub struct PhantomData<T: PointeeSized> {field:*const T}",
            ),
            "PhantomData",
        ),
        (
            "tuple",
            header.replace(
                "pub struct PhantomData<T: PointeeSized>;",
                "pub struct PhantomData<T: PointeeSized>();",
            ),
            "PhantomData",
        ),
        (
            "formals",
            header.replace(
                "pub struct PhantomData<T: PointeeSized>;",
                "pub struct PhantomData<T: PointeeSized,U: PointeeSized>;",
            ),
            "PhantomData",
        ),
        (
            "kind",
            header.replace(
                "pub struct PhantomData<T: PointeeSized>;",
                "pub trait PhantomData<T: PointeeSized> {}",
            ),
            "kind",
        ),
        (
            "escape",
            format!(
                "{source}\nfn escape<'a>(x:PhantomData<&'a u8>)->PhantomData<&'static u8>{{x}}"
            ),
            "lifetime",
        ),
    ] {
        let error = checked_source(&changed, name)
            .err()
            .ok_or_else(|| format!("{name} unexpectedly accepted"))?;
        assert!(error.contains(needle), "{name}: {error}");
    }
    let ordinary = test_source::resolve(
        "struct PhantomData;fn main(){let _marker=PhantomData;}",
        true,
    )?;
    let analysis = crate::pipeline::analyze_source(&ordinary)?;
    assert!(
        ordinary.resolution.source_bindings.is_none() && analysis.facts.source_bindings.is_none()
    );
    assert!(
        analysis
            .facts
            .aggregates
            .iter()
            .all(|definition| definition.kind != types::aggregate::Kind::PhantomData)
    );
    assert_eq!(crate::support::bindings::LangItem::ALL.len(), 61);
    let mut all_roles = 0_u32;
    for (index, role) in crate::support::bindings::LangItem::ALL[..32]
        .iter()
        .copied()
        .enumerate()
    {
        assert_eq!(role.index(), index);
        all_roles |= 1_u32 << index;
    }
    assert_eq!(all_roles, u32::MAX);
    assert_eq!(crate::support::bindings::LangItem::PhantomData.index(), 31);
    assert_eq!(
        1_u32 << crate::support::bindings::LangItem::PhantomData.index(),
        0x8000_0000
    );
    Ok(())
}

// Use the real CFG/ownership/region consumer, which supplies borrow-place
// provenance. Type-only contracts alone cannot replace this proof boundary.
fn checked_source(source: &str, name: &str) -> std::result::Result<(), String> {
    let report = test_source::source_provider(source, name)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report).map_err(|error| error.to_string())?;
    crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
    .map(|_| ())
    .map_err(|error| error.to_string())
}

fn marker(
    types: &mut Store,
    defs: &mut Definitions,
    item: ItemId,
    target: TypeId,
) -> Result<TypeId> {
    let empty = types.intern_list(&[])?;
    if defs.get(item).is_none() {
        defs.insert(
            item,
            types::aggregate::Kind::PhantomData,
            Repr::default(),
            &[Variant {
                fields: empty,
                discriminant: 0,
            }],
        )?;
    }
    let args = types.intern_arguments(&[Argument::Type(target)])?;
    Ok(types.intern(Kind::Adt(item, args))?)
}
#[test]
fn logical_argument_graphs_preserve_physical_drop_warm_work_and_hot_layouts() -> Result {
    let mut store = Store::default();
    let mut defs = Definitions::default();
    let item = ItemId(92);
    let formal = store.intern_parameter(Parameter {
        owner: item,
        index: 0,
        kind: ParameterKind::Type { sized: false },
    })?;
    let formal = store.intern(Kind::Parameter(formal))?;
    let generic = marker(&mut store, &mut defs, item, formal)?;
    let Some(Kind::Adt(_, args)) = store.kind(generic) else {
        return Err("nominal".into());
    };
    let variances = Variances::infer(&store, &defs, &[(item, args)])?;
    assert_eq!(variances.get(item), Some(&[Variance::Covariant][..]));
    let raw = store.intern(Kind::Pointer {
        mutable: false,
        pointee: TypeId::UNIT,
    })?;
    for depth in [1, 8, 64, 128] {
        let mut ty = raw;
        for _ in 0..depth {
            ty = marker(&mut store, &mut defs, item, ty)?;
        }
        let mut query = Query::new(Obligation {
            ty,
            marker: AutoTrait::Send,
        });
        assert_eq!(
            query.advance(&mut store, &mut defs, |_, _| Ok(Rule::Structural))?,
            Step::Ready(false)
        );
        assert!(query.steps <= depth * 8 + 16, "{}", query.steps);
        let before = query.steps;
        assert_eq!(
            query.advance(&mut store, &mut defs, |_, _| Ok(Rule::Structural))?,
            Step::Ready(false)
        );
        assert_eq!(query.steps, before);
    }
    let mut exhausted = Query::new(Obligation {
        ty: generic,
        marker: AutoTrait::Send,
    });
    exhausted.steps = 1_000_000;
    assert!(
        exhausted
            .advance(&mut store, &mut defs, |_, _| Ok(Rule::Structural))
            .is_err()
    );
    assert!(exhausted.steps > 1_000_000);
    owned_argument(&mut store, &mut defs, item)?;
    let empty = store.intern_list(&[])?;
    let ordinary = ItemId(93);
    defs.insert(
        ordinary,
        types::aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields: empty,
            discriminant: 0,
        }],
    )?;
    let ordinary = store.intern(Kind::Adt(ordinary, GenericArgs::EMPTY))?;
    assert_eq!(defs.phantom_argument(&store, ordinary)?, None);
    assert!(
        defs.phantom_argument_from(&store, item, GenericArgs::EMPTY)
            .is_err()
    );
    observe_layouts();
    Ok(())
}
fn owned_argument(store: &mut Store, defs: &mut Definitions, phantom: ItemId) -> Result {
    let owned = ItemId(94);
    let empty = store.intern_list(&[])?;
    defs.insert(
        owned,
        types::aggregate::Kind::Struct,
        Repr::default(),
        &[Variant {
            fields: empty,
            discriminant: 0,
        }],
    )?;
    let args = store.intern_regions(&[RegionId(11)])?;
    let owned = store.intern(Kind::Adt(owned, args))?;
    let marker = marker(store, defs, phantom, owned)?;
    defs.instantiate(store, marker)?;
    let mut drops = types::destruction::NeedsDrop::new([ItemId(94)]);
    assert!(drops.demand(store, defs, owned)?);
    assert!(!drops.demand(store, defs, marker)?);
    let mut regions = types::destruction::regions::Regions::new([ItemId(94)])?;
    assert_eq!(regions.get(store, defs, marker)?, [RegionId(11)]);
    let before = regions.steps;
    assert_eq!(regions.get(store, defs, marker)?, [RegionId(11)]);
    assert_eq!(regions.steps, before);
    Ok(())
}
fn observe_layouts() {
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<crate::syntax::ast::Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    eprintln!(
        "Phantom layouts Kind={} AST={} Node={} Definition={} Definitions={} Store={} Package={} Report={} Body={} Catalog={} Reservations={}",
        size_of::<Kind>(),
        size_of::<crate::syntax::ast::Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<types::aggregate::Definition>(),
        size_of::<Definitions>(),
        size_of::<Store>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>(),
        size_of::<crate::body::ir::Body>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>()
    );
}
