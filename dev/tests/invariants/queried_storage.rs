//! Actual source SIZE initializers and Clone bodies retain ordinary caller proof.
use super::*;
use crate::{
    pipeline::{self, Analysis},
    types::{
        GenericArgs, ItemId, TypeId,
        aggregate::{Definitions, Kind as Aggregate, Repr, Variant},
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const LAYOUT: &str = r#"
pub mod intrinsics {
    #[rustc_intrinsic]pub const fn size_of<T>()->usize;
    #[rustc_intrinsic]pub const fn align_of<T>()->usize;
}
pub trait SizedTypeProperties:Sized {
    #[lang="mem_size_const"]const SIZE:usize=intrinsics::size_of::<Self>();
    #[lang="mem_align_const"]const ALIGN:usize=intrinsics::align_of::<Self>();
}
impl<T>SizedTypeProperties for T{}
pub mod mem {
    pub const fn size_of<T>()->usize{<T as super::SizedTypeProperties>::SIZE}
    pub const fn align_of<T>()->usize{<T as super::SizedTypeProperties>::ALIGN}
}
#[repr(C)]pub struct Pair<T>{tag:u8,value:T}
pub fn generic_size<T>()->usize{mem::size_of::<T>()}
#[unsafe(no_mangle)]pub extern "C" fn scalar_size()->usize{generic_size::<u16>()}
#[unsafe(no_mangle)]pub extern "C" fn array_size()->usize{generic_size::<[u16;3]>()}
#[unsafe(no_mangle)]pub extern "C" fn pair_size()->usize{generic_size::<Pair<u16>>()}
#[unsafe(no_mangle)]pub extern "C" fn pair_align()->usize{mem::align_of::<Pair<u16>>()}
#[unsafe(no_mangle)]pub extern "C" fn zero_size()->usize{mem::size_of::<[u64;0]>()}
#[unsafe(no_mangle)]pub extern "C" fn zero_align()->usize{mem::align_of::<[u64;0]>()}
"#;
const CLONE: &str = r#"
pub struct D(u8);
impl Clone for D{fn clone(&self)->Self{D(self.0)}}
#[unsafe(no_mangle)]pub extern "C" fn clone_value(x:u8)->u8{let d=D(x);let other=d.clone();other.0}
#[unsafe(no_mangle)]pub extern "C" fn clone_from_value(x:u8)->u8{let mut d=D(1);d.clone_from(&D(x));d.0}
"#;
const GLUE: &str = "#[lang=\"drop_glue\"]pub(crate) const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}";
fn source(tail: &str) -> Result<String> {
    let foundation = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    Ok(format!("#![feature(intrinsics)]{foundation}{GLUE}{tail}"))
}
fn build(tail: &str, tag: &str) -> Result<(String, crate::lowir::program::Program)> {
    let input = source(tail)?;
    let report = crate::test_source::source_provider(&input, tag)?;
    let Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    Ok((input, program))
}
#[test]
fn actual_layout_initializers_and_nominal_clone_bodies_prepare_native_roots() -> Result {
    let (input, program) = build(&(LAYOUT.to_owned() + CLONE), "source_storage")?;
    for expected in [
        "scalar_size",
        "array_size",
        "pair_size",
        "pair_align",
        "zero_size",
        "zero_align",
        "clone_value",
        "clone_from_value",
    ] {
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|s| s.name == expected && s.body.is_some())
                .count(),
            1
        );
    }
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    eprintln!(
        "source storage Layouts={} Catalog={} Resolution={} Package={}",
        size_of::<crate::abi::layout::Layouts>(),
        size_of::<crate::sema::compiler_items::Catalog>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<crate::resolve::package::Package>()
    );
    Ok(())
}
#[test]
fn structural_readiness_preserves_pending_fields_closedness_and_failed_work() -> Result {
    let mut types = crate::types::Store::default();
    let mut definitions = Definitions::default();
    let mut layouts = crate::abi::layout::Layouts::default();
    let item = ItemId(671);
    let ty = types.intern(crate::types::Kind::Adt(item, GenericArgs::EMPTY))?;
    let mut work = 0;
    let pending = layouts.storage_readiness(&types, &definitions, ty, &mut work, 100);
    assert!(pending.is_err());
    assert!(work > 0);
    assert!(layouts.get(ty).is_none());
    let fields = types.intern_list(&[
        TypeId::primitive(crate::types::Primitive::U8),
        TypeId::primitive(crate::types::Primitive::U16),
    ])?;
    definitions.insert(
        item,
        Aggregate::Struct,
        Repr {
            representation: crate::types::aggregate::Representation::C,
            ..Repr::default()
        },
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    definitions.prepare(&mut types, ty)?;
    work = 0;
    layouts.storage_readiness(&types, &definitions, ty, &mut work, 100)?;
    assert_eq!(work, 3);
    let id = layouts.demand_bounded(&types, &definitions, ty, 100)?;
    let actual = layouts.at(id).ok_or("actual prepared nominal layout")?;
    assert_eq!((actual.size, actual.align), (4, 2));
    let cold = layouts.steps;
    work = 0;
    layouts.storage_readiness(&types, &definitions, ty, &mut work, 1)?;
    assert_eq!(work, 1);
    assert_eq!(layouts.steps, cold);
    let prepared = layouts.len();
    work = 0;
    assert!(
        layouts
            .storage_readiness(&types, &definitions, ty, &mut work, 0)
            .is_err()
    );
    assert_eq!(work, 1);
    assert_eq!(layouts.len(), prepared);
    let parameter = types.intern_parameter(crate::types::parameters::Parameter {
        owner: ItemId(672),
        index: 0,
        kind: crate::types::parameters::ParameterKind::Type { sized: true },
    })?;
    work = 0;
    let open = layouts
        .storage_readiness(&types, &definitions, parameter, &mut work, 100)
        .err()
        .ok_or("open type accepted")?;
    assert!(open.contains("closed normalized type"));
    let report = crate::test_source::source_provider(&source("")?, "source_storage_budget")?;
    let mut facts = Declarations::new(&report)?;
    facts.body_work.canonical_steps = crate::support::crates::MAX_COMPONENTS;
    assert!(prepare_layout(&mut facts, TypeId::primitive(crate::types::Primitive::U16)).is_err());
    assert_eq!(
        facts.body_work.canonical_steps,
        crate::support::crates::MAX_COMPONENTS + 1
    );
    assert!(facts.layouts.is_empty());
    eprintln!(
        "source layout cold_steps={cold} warm_readiness=1 failed_attempt=1 actual_size=4 align=2"
    );
    Ok(())
}
