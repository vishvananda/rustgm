//! PA20 declaration controls have independently declared language outcomes.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    resolve::model::{DefId, DefKind},
    types::{ItemId, Kind, Primitive, TypeId, parameters::predicates::Bound},
};

const CASES: &[(&str, &str, bool)] = &[
    (
        "unconstrained-impl-lifetime",
        "trait A{type Out;}impl<'a>A for u8{type Out=&'a u8;}fn main(){}",
        false,
    ),
    (
        "unused-impl-lifetime",
        "trait A{type Out;}impl<'a>A for u8{type Out=u8;}fn main(){}",
        true,
    ),
    (
        "alias-erased-impl-lifetime",
        "type Erase<'a>=u8;trait A{type Out;}impl<'a>A for u8{type Out=Erase<'a>;}fn main(){}",
        true,
    ),
    (
        "unconstrained-secondary-lifetime",
        "trait A<'a>{type Out;}impl<'a,'b>A<'a>for u8{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "outlives-does-not-constrain",
        "trait A<'a>{type Out;}impl<'a:'b,'b:'a>A<'a>for u8{type Out=&'b u8;}fn main(){}",
        false,
    ),
    (
        "nested-binder-is-independent",
        "trait A{type Out;}impl<'a>A for u8{type Out=for<'b>fn(&'b u8)->&'b u8;}fn main(){}",
        true,
    ),
    (
        "nested-outer-capture",
        "trait A{type Out;}impl<'a>A for u8{type Out=fn(&'a u8);}fn main(){}",
        false,
    ),
    (
        "constrained-nested-outer-capture",
        "trait A{type Out;}impl<'a>A for &'a u8{type Out=fn(&'a u8);}fn main(){}",
        true,
    ),
    (
        "second-output-lifetime",
        "trait A<'a>{type X;type Y;}impl<'a,T>A<'a>for T{type X=u8;type Y=&'a T;}fn main(){}",
        false,
    ),
    (
        "other-impl-environment",
        "struct N;trait A{type Out:Copy;}impl A for u8{type Out=u8;}impl A for u16{type Out=N;}fn main(){}",
        false,
    ),
    (
        "nominal-requirement",
        "struct W<T:Copy>(T);trait A{type Out;}impl<T>A for T{type Out=W<T>;}fn main(){}",
        false,
    ),
    (
        "implied-input-lifetime",
        "trait A<'a>{type Out;}impl<'a,T>A<'a>for &'a T{type Out=&'a T;}fn main(){}",
        true,
    ),
    ("required", "trait A{type Out;}fn main(){}", true),
    (
        "concrete",
        "trait A{type Out;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "multiple",
        "trait A{type X;type Y;}impl A for u8{type Y=u16;type X=u32;}fn main(){}",
        true,
    ),
    (
        "generic",
        "trait A{type Out;}impl<T>A for T{type Out=T;}fn main(){}",
        true,
    ),
    (
        "const",
        "trait A{type Out;}impl<T,const N:usize>A for [T;N]{type Out=[T;N];}fn main(){}",
        true,
    ),
    (
        "lifetime",
        "trait A<'a>{type Out;}impl<'a,T:'a>A<'a>for T{type Out=&'a T;}fn main(){}",
        true,
    ),
    (
        "self",
        "trait A{type Out;}impl A for u8{type Out=Self;}fn main(){}",
        true,
    ),
    (
        "relaxed",
        "trait A{type Out:?Sized;}impl A for u8{type Out=[u16];}fn main(){}",
        true,
    ),
    (
        "relaxed-parameter",
        "trait A{type Out:?Sized;}impl<T:?Sized>A for T{type Out=T;}fn main(){}",
        true,
    ),
    (
        "copy-bound",
        "trait A{type Out:Copy;}impl<T:Copy>A for T{type Out=T;}fn main(){}",
        true,
    ),
    (
        "clone-bound",
        "trait A{type Out:Clone;}impl A for u8{type Out=u16;}fn main(){}",
        true,
    ),
    (
        "separate-namespaces",
        "trait A{type X;fn X();}impl A for u8{type X=u16;fn X(){}}fn main(){}",
        true,
    ),
    (
        "missing",
        "trait A{type Out;}impl A for u8{}fn main(){}",
        false,
    ),
    (
        "extra",
        "trait A{}impl A for u8{type Out=u16;}fn main(){}",
        false,
    ),
    (
        "wrong-kind",
        "trait A{type Out;}impl A for u8{fn Out(){}}fn main(){}",
        false,
    ),
    ("default", "trait A{type Out=u8;}fn main(){}", false),
    (
        "inherent",
        "struct S;impl S{type Out=u8;}fn main(){}",
        false,
    ),
    (
        "impl-bound",
        "trait A{type Out;}impl A for u8{type Out:Copy=u16;}fn main(){}",
        false,
    ),
    (
        "trait-visibility",
        "trait A{pub type Out;}fn main(){}",
        false,
    ),
    (
        "impl-visibility",
        "trait A{type Out;}impl A for u8{pub type Out=u16;}fn main(){}",
        false,
    ),
    (
        "implicit-sized",
        "trait A{type Out;}impl A for u8{type Out=[u16];}fn main(){}",
        false,
    ),
    (
        "parameter-sized",
        "trait A{type Out;}impl<T:?Sized>A for T{type Out=T;}fn main(){}",
        false,
    ),
    (
        "missing-copy",
        "struct S;trait A{type Out:Copy;}impl A for u8{type Out=S;}fn main(){}",
        false,
    ),
    (
        "missing-generic-copy",
        "trait A{type Out:Copy;}impl<T>A for T{type Out=T;}fn main(){}",
        false,
    ),
    (
        "rhs-well-formed",
        "trait A{type Out:?Sized;}impl A for u8{type Out=[[u8];1];}fn main(){}",
        false,
    ),
    (
        "missing-outlives",
        "trait A<'a>{type Out;}impl<'a,T>A<'a>for T{type Out=&'a T;}fn main(){}",
        false,
    ),
    (
        "elided-reference",
        "trait A{type Out;}impl<T>A for T{type Out=&T;}fn main(){}",
        false,
    ),
];

#[test]
fn independently_qualified_declarations() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("associated-{name}"), source, accepted)?;
        let result = semantic::analyze(source).map(|_| ());
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

#[test]
fn declared_projection_keeps_trait_self_and_result_contract()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze("trait A<T>{type Out:Copy;}fn main(){}")?;
    let find = |name: &str, kind| {
        report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, d)| d.kind == kind && report.package.names.get(d.name) == name)
            .map(|(i, _)| u32::try_from(i).map(DefId))
            .transpose()
    };
    let trait_item = find("A", DefKind::Trait)?.ok_or("missing trait")?;
    let declaration = find("Out", DefKind::TypeAlias)?.ok_or("missing type")?;
    let contract = facts
        .associated
        .type_contract(declaration)
        .ok_or("missing contract")?;
    assert!(
        contract
            .bounds
            .contains(&rustgm::types::parameters::clauses::Clause::Predicate(
                rustgm::types::parameters::predicates::Predicate {
                    ty: contract.projection,
                    bound: Bound::Copy,
                }
            ))
    );
    let Some(Kind::Projection(application)) = facts.types.kind(contract.projection) else {
        return Err("missing projection".into());
    };
    assert_eq!(application.item, ItemId(declaration.0));
    assert_eq!(
        application.subject,
        facts
            .associated
            .schema(trait_item)
            .ok_or("missing Self")?
            .self_type
    );
    assert_eq!(
        facts
            .types
            .trait_application(application.trait_application)
            .ok_or("missing application")?
            .item,
        ItemId(trait_item.0)
    );
    assert_eq!(facts.types.is_sized(contract.projection), Some(true));
    Ok(())
}

#[test]
fn implementation_retains_each_type_by_declaration_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze(
        "trait A{type X;type Y;}impl A for u8{type Y=u16;type X=u32;}fn main(){}",
    )?;
    let implementation = facts
        .generics
        .implementations()
        .next()
        .ok_or("missing implementation")?;
    assert_eq!(implementation.types.len(), 2);
    let selected = implementation
        .types
        .iter()
        .map(|&(declaration, definition)| {
            let expected = &report.resolution.definitions[declaration.0 as usize];
            let actual = &report.resolution.definitions[definition.0 as usize];
            assert_eq!(expected.name, actual.name);
            assert_ne!(declaration, definition);
            (
                report.package.names.get(expected.name),
                facts.definitions[definition.0 as usize],
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        selected,
        [
            ("X", Some(TypeId::primitive(Primitive::U32))),
            ("Y", Some(TypeId::primitive(Primitive::U16)))
        ]
    );
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn associated_type_contracts_preserve_method_execution() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "associated-declarations",
        &[
            (
                "default",
                "trait A:Sized{type Out;fn value(self)->u8{7}}impl A for u8{type Out=u16;}fn get<T:A>(x:T)->u8{x.value()}fn main(){check(get(9u8)==7)}",
                false,
            ),
            (
                "override",
                "trait A{type Out:Copy;fn value(self)->u8;}impl<T:Copy>A for T{type Out=T;fn value(self)->u8{9}}fn get<T:A>(x:T)->u8{x.value()}fn main(){check(get(7u16)==9)}",
                false,
            ),
        ],
    )
}
