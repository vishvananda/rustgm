//! Header/value authority is real; it cannot silently prove an unimplemented relation.
use super::*;
use crate::{
    test_source,
    types::{Primitive, TypeId, arguments::Constant},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const CONTEXT: &str = "#![feature(const_trait_impl)]\n";
const ITEMS: &str = r#"
#[lang="bikeshed_guaranteed_no_drop"] #[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait] trait GuaranteedNoDrop{}
#[lang="destruct"] #[rustc_deny_explicit_impl] const trait Destruct:PointeeSized{}
#[lang="manually_drop"] #[repr(transparent)] struct ManuallyDrop<T:?Sized>{value:T}
impl<T> ManuallyDrop<T>{const fn new(value:T)->Self{Self{value}} const fn into_inner(value:Self)->T{value.value}}
#[lang="transmute_trait"] #[rustc_deny_explicit_impl]
#[rustc_coinductive] #[rustc_dyn_incompatible_trait]
unsafe trait Convert<Src,const ASSUME:Assume={Assume::NOTHING}> where Src:?Sized {
    unsafe fn transmute(src:Src)->Self where Src:Sized,Self:Sized {
        #[repr(C)] union Transmute<Src,Dst>{src:ManuallyDrop<Src>,dst:ManuallyDrop<Dst>}
        let transmute=Transmute{src:ManuallyDrop::new(src)};
        let dst=unsafe{transmute.dst};
        ManuallyDrop::into_inner(dst)
    }
}
#[lang="transmute_opts"] struct Assume{alignment:bool,lifetimes:bool,safety:bool,validity:bool}
impl Structural for Assume{} impl Equal for Assume{} impl Key for Assume{}
impl Assume{const NOTHING:Self=Self{alignment:false,lifetimes:false,safety:false,validity:false};}
"#;
fn report(items: &str) -> Result<crate::resolve::expansion::Report> {
    let base = test_source::fixture("pa25/tests/spec/aggregate-parameters/struct-value.rs")?;
    Ok(test_source::source_provider(
        &(CONTEXT.to_owned() + &base + items),
        "assumption_headers",
    )?)
}
fn analyze(items: &str) -> Result<crate::pipeline::Analysis> {
    Ok(crate::pipeline::analyze_source(&report(items)?)?)
}
#[test]
fn actual_forward_header_and_options_keep_ordinary_body_and_source_identity() -> Result {
    let output =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../obj/parallel/transmutability-v1");
    std::fs::create_dir_all(&output)?;
    std::fs::write(
        output.join("cold-layout.json"),
        format!(
            "{{\"reservations\":{},\"catalog\":{},\"body\":{},\"node\":{}}}\n",
            size_of::<crate::resolve::bindings::Reservations>(),
            size_of::<Catalog>(),
            size_of::<crate::body::ir::Body>(),
            size_of::<crate::syntax::ast::Node>()
        ),
    )?;
    let report = report(ITEMS)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(
        program
            .symbols
            .iter()
            .any(|symbol| symbol.name == "observed" && symbol.body.is_some())
    );
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let role = catalog
        .validated_language(LangItem::TransmuteTrait)?
        .ok_or("trait")?;
    let options = catalog
        .validated_language(LangItem::TransmuteOpts)?
        .ok_or("options")?;
    assert_ne!(role.definition, options.definition);
    assert!(facts.associated.owner(role.definition).is_some());
    assert_eq!(options::slots(&facts, options)?, [0, 1, 2, 3]);
    assert_eq!(LangItem::TransmuteTrait.index(), 83);
    assert_eq!(LangItem::TransmuteOpts.index(), 84);
    eprintln!(
        "Reservations={} Catalog={}",
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<Catalog>()
    );
    Ok(())
}
#[test]
fn canonical_options_decode_actual_named_boolean_fields_without_position_authority() -> Result {
    for reordered in [false, true] {
        let items = if reordered {
            ITEMS.replace(
                "alignment:bool,lifetimes:bool,safety:bool,validity:bool",
                "validity:bool,safety:bool,lifetimes:bool,alignment:bool",
            )
        } else {
            ITEMS.to_owned()
        };
        let mut facts = analyze(&items)?.facts;
        let role = facts
            .source_bindings
            .as_deref()
            .ok_or("catalog")?
            .validated_language(LangItem::TransmuteOpts)?
            .ok_or("options")?;
        let slots = options::slots(&facts, role)?;
        let ty = facts
            .types
            .intern(Kind::Adt(ItemId(role.definition.0), GenericArgs::EMPTY))?;
        let bool_ty = TypeId::primitive(Primitive::Bool);
        let no = facts.types.intern_constant(Constant {
            ty: bool_ty,
            bits: 0,
        })?;
        let yes = facts.types.intern_constant(Constant {
            ty: bool_ty,
            bits: 1,
        })?;
        for mask in 0_u8..16 {
            let mut fields = [no; 4];
            let expected = std::array::from_fn(|i| mask & (1 << i) != 0);
            for (i, index) in slots.into_iter().enumerate() {
                fields[index] = if expected[i] { yes } else { no };
            }
            let value = facts
                .types
                .intern_aggregate_constant(ty, None, &fields, |_, _| Ok(()))?;
            assert_eq!(decode(&facts, value)?, expected);
        }
        assert!(decode(&facts, yes).is_err());
        let tuple = facts.types.intern_list(&[bool_ty; 4])?;
        let tuple = facts.types.intern(Kind::Tuple(tuple))?;
        let value = facts
            .types
            .intern_aggregate_constant(tuple, None, &[yes; 4], |_, _| Ok(()))?;
        assert!(decode(&facts, value).is_err());
    }
    Ok(())
}
#[test]
fn malformed_formals_roles_and_option_fields_never_complete() -> Result {
    for (from, to) in [
        ("unsafe trait Convert", "trait Convert"),
        ("where Src:?Sized", "where Src:Sized"),
        ("#[rustc_deny_explicit_impl]", ""),
        ("#[rustc_coinductive]", ""),
        ("#[rustc_dyn_incompatible_trait]", ""),
        ("#[lang=\"transmute_opts\"]", ""),
        ("validity", "other"),
    ] {
        let input = ITEMS.replace(from, to);
        assert_ne!(input, ITEMS, "ineffective opponent: {from}");
        assert!(analyze(&input).is_err(), "accepted {from} -> {to}");
    }
    let integer = ITEMS
        .replace("alignment:bool", "alignment:u8")
        .replace("alignment:false", "alignment:0")
        + "impl Structural for u8{} impl Equal for u8{} impl Key for u8{}";
    let error = analyze(&integer)
        .err()
        .ok_or("nonboolean options accepted")?
        .to_string();
    assert!(error.contains("boolean fields"), "{error}");
    let counterfeit = ITEMS.replace("#[lang=\"transmute_opts\"]", "")
        + "#[lang=\"transmute_opts\"] struct Other{alignment:bool,lifetimes:bool,safety:bool,validity:bool}";
    let error = analyze(&counterfeit)
        .err()
        .ok_or("counterfeit options accepted")?
        .to_string();
    assert!(error.contains("actual transmute_opts type"), "{error}");
    Ok(())
}
#[test]
fn completed_headers_and_values_do_not_manufacture_a_trait_implementation() -> Result {
    let items = format!("{ITEMS}\nfn require<T:Convert<u8>>(){{}} fn check(){{require::<u8>();}}");
    let error = analyze(&items)
        .err()
        .ok_or("unproved transmutability accepted")?
        .to_string();
    assert!(error.contains("all-values layout"), "{error}");
    Ok(())
}
