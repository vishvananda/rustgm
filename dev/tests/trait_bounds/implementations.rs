//! Implementation headers retain applications without inventing body permissions.
use super::{cases, rustc, semantic};

const CUSTOM_DROP: &str = "struct D<'a>(&'a mut u8);impl<'a> Drop for D<'a>{fn drop(&mut self){*self.0+=1;}}mod shadow{pub trait Drop{}pub struct S<'a>(pub super::D<'a>);impl<'a> Drop for S<'a>{}}fn main(){let mut count=0;{let _s=shadow::S(D(&mut count));}check(count==1);}";

#[test]
fn a_custom_drop_marker_preserves_field_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    cases(&[(
        "impl-marker-custom-drop-cleanup",
        &format!("{}{CUSTOM_DROP}", super::native::CHECK),
        true,
    )])
}

#[test]
#[ignore = "requires the qualified CPPGM LowIR-to-native backend"]
fn custom_drop_marker_executes_field_cleanup() -> Result<(), Box<dyn std::error::Error>> {
    super::native::check_cases("trait-impl-markers", &[("custom-drop", CUSTOM_DROP, false)])
}

#[test]
fn complete_impl_applications_constrain_types_consts_and_lifetimes()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-marker-primitive",
            "trait A{}impl A for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-reference",
            "trait A{}impl<'a,T:?Sized> A for &'a T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-tuple",
            "trait A{}impl<T,U> A for (T,U){}fn main(){}",
            true,
        ),
        (
            "impl-marker-function",
            "trait A{}impl A for fn(u8)->u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-trait-parameter",
            "trait A<U>{}struct S;impl<T> A<T> for S{}fn main(){}",
            true,
        ),
        (
            "impl-marker-const",
            "trait A<const N:usize>{}impl<const N:usize> A<N> for [u8;N]{}fn main(){}",
            true,
        ),
        (
            "impl-marker-lifetime",
            "trait A<'a>{}impl<'a,T> A<'a> for &'a T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-unused-lifetime",
            "trait A{}impl<'a> A for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-unconstrained-type",
            "trait A{}struct S;impl<T> A for S{}fn main(){}",
            false,
        ),
        (
            "impl-marker-unconstrained-const",
            "trait A{}struct S;impl<const N:usize> A for S{}fn main(){}",
            false,
        ),
        (
            "impl-marker-bound-not-constraining",
            "trait A{}trait B{}struct S;impl<T:B> A for S{}fn main(){}",
            false,
        ),
        (
            "impl-marker-wrong-argument-kind",
            "trait A<const N:usize>{}impl A<u8> for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-trait-default",
            "trait A<U:?Sized=Self>{}impl A for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-user-drop",
            "trait Drop{}struct S;impl Drop for S{}fn main(){}",
            true,
        ),
    ])
}

#[test]
fn implementation_contracts_are_proved_without_assuming_the_trait_itself()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-marker-target-implied-equality",
            "trait B<'a>{}trait A{}struct S<'a:'b,'b:'a,T:B<'a>>(&'a (),&'b (),T);impl<'a,'b,T:B<'b>> A for S<'a,'b,T>{}fn main(){}",
            true,
        ),
        (
            "impl-marker-target-one-way",
            "trait B<'a>{}trait A{}struct S<'a:'b,'b,T:B<'a>>(&'a (),&'b (),T);impl<'a,'b,T:B<'b>> A for S<'a,'b,T>{}fn main(){}",
            false,
        ),
        (
            "impl-marker-copy",
            "trait A:Copy{}impl A for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-generic-copy",
            "trait A:Copy{}impl<T:Copy> A for T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-missing-copy",
            "trait A:Copy{}struct S;impl A for S{}fn main(){}",
            false,
        ),
        (
            "impl-marker-missing-generic-copy",
            "trait A:Copy{}impl<T> A for T{}fn main(){}",
            false,
        ),
        (
            "impl-marker-argument-copy",
            "trait A<U:Copy>{}impl<T:Copy> A<T> for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-argument-copy-missing",
            "trait A<U:Copy>{}impl<T> A<T> for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-supertrait-assumption",
            "trait B{}trait A:B{}impl<T:B> A for T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-supertrait-missing",
            "trait B{}trait A:B{}impl<T> A for T{}fn main(){}",
            false,
        ),
        (
            "impl-marker-target-requirements",
            "trait A{}struct S<T:Copy>(T);impl<T:Copy> A for S<T>{}fn main(){}",
            true,
        ),
        (
            "impl-marker-target-requirements-missing",
            "trait A{}struct S<T:Copy>(T);impl<T> A for S<T>{}fn main(){}",
            false,
        ),
        (
            "impl-marker-outlives",
            "trait A<'a>:'a{}impl<'a,T:'a> A<'a> for T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-outlives-missing",
            "trait A<'a>:'a{}impl<'a,T> A<'a> for T{}fn main(){}",
            false,
        ),
        (
            "impl-marker-implied-outlives",
            "trait A<'a>:'a{}impl<'a,T> A<'a> for &'a T{}fn main(){}",
            true,
        ),
        (
            "impl-marker-argument-outlives",
            "trait A<'a,T:'a>{}impl<'a,T:'a> A<'a,T> for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-argument-outlives-missing",
            "trait A<'a,T:'a>{}impl<'a,T> A<'a,T> for u8{}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn implementation_safety_and_complete_header_overlap_are_checked()
-> Result<(), Box<dyn std::error::Error>> {
    cases(&[
        (
            "impl-marker-shared-type-disjoint",
            "trait A<T>{}impl<T> A<T> for (T,T){}impl<U> A<u8> for (U,u16){}fn main(){}",
            true,
        ),
        (
            "impl-marker-shared-const-disjoint",
            "trait A<const N:usize>{}impl<const N:usize> A<N> for [u8;N]{}impl A<2> for [u8;3]{}fn main(){}",
            true,
        ),
        (
            "impl-marker-unsafe",
            "unsafe trait A{}unsafe impl A for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-unsafe-required",
            "unsafe trait A{}impl A for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-unsafe-forbidden",
            "trait A{}unsafe impl A for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-disjoint-subjects",
            "trait A{}impl A for u8{}impl A for u16{}fn main(){}",
            true,
        ),
        (
            "impl-marker-disjoint-arguments",
            "trait A<T>{}impl A<u8> for u8{}impl A<u16> for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-disjoint-constants",
            "trait A<const N:usize>{}impl A<1> for u8{}impl A<2> for u8{}fn main(){}",
            true,
        ),
        (
            "impl-marker-duplicate",
            "trait A{}impl A for u8{}impl A for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-blanket-overlap",
            "trait A{}impl<T> A for T{}impl A for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-reverse-overlap",
            "trait A{}impl A for u8{}impl<T> A for T{}fn main(){}",
            false,
        ),
        (
            "impl-marker-generic-overlap",
            "trait A<T>{}impl<T> A<T> for u8{}impl<U> A<U> for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-const-overlap",
            "trait A<const N:usize>{}impl<const N:usize> A<N> for u8{}impl A<1> for u8{}fn main(){}",
            false,
        ),
        (
            "impl-marker-lifetimes-overlap",
            "trait A<'a>{}impl<'a> A<'a> for u8{}impl A<'static> for u8{}fn main(){}",
            false,
        ),
    ])
}

#[test]
fn validated_records_keep_the_actual_application_and_owner_environment()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::{Kind, arguments::Argument, parameters::predicates::Bound};
    let source =
        "trait A<'a,T,const N:usize>{}impl<'b,U,const M:usize> A<'b,U,M> for [U;M]{}fn main(){}";
    rustc::qualify("impl-marker-retained-application", source, true)?;
    let (_, facts) = semantic::analyze(source)?;
    let records = facts.generics.implementations().collect::<Vec<_>>();
    assert_eq!(records.len(), 1);
    let record = records[0];
    assert!(record.methods.is_empty());
    let Bound::Trait(id) = record.predicate.bound else {
        return Err("missing trait application".into());
    };
    let application = facts
        .types
        .trait_application(id)
        .ok_or("missing application")?;
    let actual = facts
        .types
        .arguments(application.arguments)
        .ok_or("missing arguments")?
        .collect::<Vec<_>>();
    let expected = facts
        .generics
        .parameters
        .get(&record.owner)
        .ok_or("missing owner parameters")?
        .iter()
        .map(|parameter| parameter.argument)
        .collect::<Vec<_>>();
    assert_eq!(actual, expected);
    let Some(Kind::ArrayTerm { element, length }) = facts.types.kind(record.predicate.ty) else {
        return Err("missing array Self type".into());
    };
    assert_eq!(actual[1], Argument::Type(element));
    assert_eq!(actual[2], Argument::Const(length));
    assert_eq!(
        facts
            .associated
            .schema(record.owner)
            .ok_or("missing impl schema")?
            .self_type,
        record.predicate.ty
    );
    super::prove(source)?;
    Ok(())
}
