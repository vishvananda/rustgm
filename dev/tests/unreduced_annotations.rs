//! Declaration constraints retain original projections while executable facts use normal forms.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
    sema::model::Declarations,
    types::{Kind, Primitive, TypeId, arguments::Argument},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

const ALIAS_CASES: &[(&str, &str, bool)] = &[
    (
        "self-parameter",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;fn main(){let _:P<u8>=7u16;}",
        true,
    ),
    (
        "trait-parameter",
        "trait A<T>{type Out;}impl<T>A<T>for u8{type Out=u16;}type P<T>=<u8 as A<T>>::Out;fn main(){let _:P<u32>=7u16;}",
        true,
    ),
    (
        "transitive-alias",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;type Q<T>=P<T>;fn main(){let _:Q<u8>=7u16;}",
        true,
    ),
    (
        "compound-alias",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=[<T as A>::Out;2];fn main(){let _:P<u8>=[7u16,9];}",
        true,
    ),
    (
        "alias-default",
        "trait A{type Out;}impl A for u8{type Out=u16;}type D<T,U=T>=(T,U);type P=D<<u8 as A>::Out>;fn main(){let _:P=(7u16,9);}",
        true,
    ),
    (
        "nominal-default",
        "trait A{type Out;}impl A for u8{type Out=u16;}struct S<T,U=T>(T,U);type P=S<<u8 as A>::Out>;fn main(){let _:P=S(7u16,9);}",
        true,
    ),
    (
        "macro-type-fragment",
        "trait A{type Out;}impl A for u8{type Out=u16;}macro_rules! pair{($t:ty)=>{type P=($t,$t);}}pair!(<u8 as A>::Out);fn main(){let _:P=(7u16,9);}",
        true,
    ),
    (
        "lifetime-alias",
        "trait A{type Out;}impl A for u8{type Out=u16;}type P<'a>=&'a <u8 as A>::Out;type Q<'b>=P<'b>;fn f<'a>(x:&'a u16)->Q<'a>{x}fn main(){let _: &u16=f(&7);}",
        true,
    ),
    (
        "lifetime-alias-cannot-strengthen",
        "trait A{type Out;}impl A for u8{type Out=u16;}type P<'a>=&'a <u8 as A>::Out;fn f<'a>(x:&'a u16)->P<'static>{x}fn main(){}",
        false,
    ),
    (
        "unused-still-invalid",
        "trait A{type Out;}impl A for u8{type Out=u16;}type P<T>=<u8 as A>::Out;fn main(){}",
        false,
    ),
    (
        "wrong-normal-form",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;fn main(){let _:P<u8>=7u8;}",
        false,
    ),
    (
        "explicit-function-argument",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;fn f<T>(x:P<T>)->P<T>{x}fn main(){let _:u16=f::<u8>(7);}",
        true,
    ),
    (
        "projected-result-does-not-infer-argument",
        "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;fn f<T>(x:P<T>)->P<T>{x}fn main(){let _:u16=f(7);}",
        false,
    ),
];

#[test]
fn alias_parameter_occurrences_survive_normalization() -> TestResult {
    for &(name, source, accepted) in ALIAS_CASES {
        rustc::qualify(&format!("unreduced-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(report, mut facts)| {
            let main = report
                .resolution
                .definitions
                .iter()
                .position(|d| {
                    d.kind == DefKind::Function && report.package.names.get(d.name) == "main"
                })
                .ok_or("missing main")?;
            let main = DefId(u32::try_from(main).map_err(|e| e.to_string())?);
            rustgm::lower::instances::build(&report, &mut facts, main)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}

fn alias(
    report: &Report,
    facts: &Declarations,
    name: &str,
) -> Result<(TypeId, TypeId), Box<dyn std::error::Error>> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| d.kind == DefKind::TypeAlias && report.package.names.get(d.name) == name)
        .ok_or("missing alias")?;
    let id = DefId(u32::try_from(index)?);
    Ok((
        facts.definitions[index].ok_or("missing normal form")?,
        facts
            .unreduced
            .definition(id)
            .ok_or("missing unreduced alias")?,
    ))
}

#[test]
fn equal_normal_forms_keep_distinct_projected_origins() -> TestResult {
    let source = "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u32{type Out=u16;}type P=(<u8 as A>::Out,<u32 as A>::Out);type Q=P;fn main(){let _:Q=(7u16,9);}";
    let (report, facts) = semantic::analyze(source)?;
    let (normal, original) = alias(&report, &facts, "P")?;
    assert_eq!(alias(&report, &facts, "Q")?, (normal, original));
    let Kind::Tuple(normal) = facts.types.kind(normal).ok_or("normal tuple")? else {
        return Err("normal kind".into());
    };
    let Kind::Tuple(original) = facts.types.kind(original).ok_or("original tuple")? else {
        return Err("original kind".into());
    };
    assert_eq!(
        facts.types.list(normal).ok_or("normal elements")?,
        &[TypeId::primitive(Primitive::U16); 2]
    );
    let originals = facts.types.list(original).ok_or("original elements")?;
    for (&ty, subject) in originals.iter().zip([Primitive::U8, Primitive::U32]) {
        let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
            return Err("lost projection".into());
        };
        assert_eq!(application.subject, TypeId::primitive(subject));
    }
    Ok(())
}

#[test]
fn defaults_substitute_unreduced_arguments_in_their_own_prefix() -> TestResult {
    let source = "trait A{type Out;}impl A for u8{type Out=u16;}struct S<T,U=T>(T,U);type P=S<<u8 as A>::Out>;fn main(){}";
    let (report, facts) = semantic::analyze(source)?;
    let (normal, original) = alias(&report, &facts, "P")?;
    let Some(Kind::Adt(_, normal_args)) = facts.types.kind(normal) else {
        return Err("normal nominal".into());
    };
    let Some(Kind::Adt(_, original_args)) = facts.types.kind(original) else {
        return Err("original nominal".into());
    };
    assert_eq!(
        facts
            .types
            .arguments(normal_args)
            .ok_or("normal arguments")?
            .collect::<Vec<_>>(),
        vec![Argument::Type(TypeId::primitive(Primitive::U16)); 2]
    );
    let originals = facts
        .types
        .arguments(original_args)
        .ok_or("original arguments")?
        .collect::<Vec<_>>();
    assert_eq!(originals.len(), 2);
    assert_eq!(originals[0], originals[1]);
    let Argument::Type(ty) = originals[0] else {
        return Err("original argument kind".into());
    };
    assert!(matches!(facts.types.kind(ty), Some(Kind::Projection(_))));
    Ok(())
}

#[test]
fn ordinary_programs_need_no_unreduced_storage() -> TestResult {
    let (_, facts) = semantic::analyze("type P<T>=(T,T);fn main(){let _:P<u8>=(7,9);}")?;
    assert!(facts.unreduced.is_empty());
    Ok(())
}

#[test]
fn alias_chains_retain_origins_without_repeating_completed_normalization() -> TestResult {
    use std::fmt::Write;
    let count = 512;
    let mut source = String::from(
        "#![recursion_limit=\"16384\"]\ntrait A{type Out;}impl<T>A for T{type Out=u16;}type P0=<u8 as A>::Out;\n",
    );
    for index in 1..count {
        writeln!(&mut source, "type P{index}=<P{} as A>::Out;", index - 1)?;
    }
    writeln!(&mut source, "fn main(){{let _:P{}=7u16;}}", count - 1)?;
    rustc::qualify("unreduced-alias-chain", &source, true)?;
    let (report, facts) = semantic::analyze(&source)?;
    let (normal, mut original) = alias(&report, &facts, &format!("P{}", count - 1))?;
    assert_eq!(normal, TypeId::primitive(Primitive::U16));
    for _ in 0..count {
        let Some(Kind::Projection(application)) = facts.types.kind(original) else {
            return Err("missing original alias-chain projection".into());
        };
        original = application.subject;
    }
    assert_eq!(original, TypeId::primitive(Primitive::U8));
    Ok(())
}

#[test]
fn compound_wrappers_and_signatures_keep_their_original_children() -> TestResult {
    let source = "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u32{type Out=u16;}type P=(&'static [<u8 as A>::Out;2],*const <u32 as A>::Out,fn(<u8 as A>::Out)-><u32 as A>::Out);type Q=P;fn f(x:<u8 as A>::Out)-><u32 as A>::Out{x}fn main(){let _:fn(u16)->u16=f;}";
    rustc::qualify("unreduced-wrappers", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let (normal, original) = alias(&report, &facts, "P")?;
    assert_eq!(alias(&report, &facts, "Q")?, (normal, original));
    assert_eq!(facts.types.contains_projections(normal), Some(false));
    let Some(Kind::Tuple(elements)) = facts.types.kind(original) else {
        return Err("original tuple".into());
    };
    let children = facts.types.list(elements).ok_or("original children")?;
    let Some(Kind::Reference { referent, .. }) = facts.types.kind(children[0]) else {
        return Err("original reference".into());
    };
    let Some(Kind::Array { element, length: 2 }) = facts.types.kind(referent) else {
        return Err("original array".into());
    };
    assert!(matches!(
        facts.types.kind(element),
        Some(Kind::Projection(_))
    ));
    let Some(Kind::Pointer { pointee, .. }) = facts.types.kind(children[1]) else {
        return Err("original pointer".into());
    };
    assert!(matches!(
        facts.types.kind(pointee),
        Some(Kind::Projection(_))
    ));
    let Some(Kind::Function {
        parameters, result, ..
    }) = facts.types.kind(children[2])
    else {
        return Err("original callable".into());
    };
    assert!(matches!(
        facts.types.kind(result),
        Some(Kind::Projection(_))
    ));
    let input = facts.types.list(parameters).ok_or("original parameter")?[0];
    assert!(matches!(facts.types.kind(input), Some(Kind::Projection(_))));
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
        .ok_or("missing function")?;
    let original = facts
        .unreduced
        .node(function.node)
        .ok_or("original function item")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(original) else {
        return Err("original item kind".into());
    };
    assert_eq!(signature, children[2]);
    assert_eq!(
        facts.types.contains_projections(function.signature),
        Some(false)
    );
    Ok(())
}

#[test]
fn nested_projection_inputs_survive_before_selection_normalizes_them() -> TestResult {
    let source = "trait A{type Out;}impl A for u8{type Out=u16;}impl A for u32{type Out=u16;}trait B<T>{type Out;}impl B<u16>for u16{type Out=u64;}type P=<<u8 as A>::Out as B<<u32 as A>::Out>>::Out;fn main(){let _:P=7u64;}";
    rustc::qualify("unreduced-projection-inputs", source, true)?;
    let (report, facts) = semantic::analyze(source)?;
    let (normal, original) = alias(&report, &facts, "P")?;
    assert_eq!(normal, TypeId::primitive(Primitive::U64));
    let Some(Kind::Projection(application)) = facts.types.kind(original) else {
        return Err("outer projection".into());
    };
    let Some(Kind::Projection(subject)) = facts.types.kind(application.subject) else {
        return Err("subject projection".into());
    };
    assert_eq!(subject.subject, TypeId::primitive(Primitive::U8));
    let arguments = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("original trait application")?
        .arguments;
    let argument = facts
        .types
        .arguments(arguments)
        .ok_or("original trait arguments")?
        .next()
        .ok_or("missing trait argument")?;
    let Argument::Type(ty) = argument else {
        return Err("original argument kind".into());
    };
    let Some(Kind::Projection(argument)) = facts.types.kind(ty) else {
        return Err("argument projection".into());
    };
    assert_eq!(argument.subject, TypeId::primitive(Primitive::U32));
    Ok(())
}

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn executable_aliases_use_normalized_layouts() -> TestResult {
    native::check_cases(
        "unreduced-aliases",
        &[
            (
                "erased-parameter",
                "trait A{type Out;}impl<T>A for T{type Out=u16;}type P<T>=<T as A>::Out;fn f<T>(x:P<T>)->P<T>{x+1}fn main(){check(f::<u8>(299)==300)}",
                false,
            ),
            (
                "default-arguments",
                "trait A{type Out;}impl A for u8{type Out=u16;}struct S<T,U=T>(T,U);type P=S<<u8 as A>::Out>;fn main(){let x:P=S(299,400);check(x.0+x.1==699)}",
                false,
            ),
        ],
    )
}
