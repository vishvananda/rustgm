#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{sema::model::Operation, types::Kind};

const ACCEPTED: &[(&str, &str)] = &[
    (
        "empty-late",
        "fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{id::<>(x)}",
    ),
    (
        "empty-early",
        "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(x:&u8)->&u8{id::<>(x)}",
    ),
    (
        "empty-nominal",
        "struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->S<'a>{let s:S<>=S::<>(x);s}",
    ),
    (
        "enum-owner",
        "enum E<'a>{A(&'a u8),B}fn f<'a>(x:&'a u8)->E<'a>{E::<'a>::A(x)}",
    ),
    (
        "enum-final",
        "enum E<'a>{A(&'a u8),B}fn f<'a>(x:&'a u8)->E<'a>{E::A::<'a>(x)}",
    ),
    (
        "enum-unit",
        "enum E<'a>{A(&'a u8),B}fn f<'a>()->E<'a>{E::<'a>::B}",
    ),
    (
        "enum-empty-final",
        "enum E<'a>{A(&'a u8)}fn f<'a>(x:&'a u8)->E<'a>{E::<'a>::A::<>(x)}",
    ),
    ("module-empty", "mod m{pub fn f(){}}fn f(){m::<>::f()}"),
    (
        "local-named",
        "fn f<'a>(x:&'a u8)->&'a u8{let y:&'a u8=x;y}",
    ),
    ("local-elided", "fn f(x:&u8)->&u8{let y:&u8=x;y}"),
    ("local-placeholder", "fn f(x:&u8)->&u8{let y:&'_ u8=x;y}"),
    (
        "local-alias",
        "type R<'a>=&'a u8;fn f<'a>(x:&'a u8)->&'a u8{let y:R<'a>=x;y}",
    ),
    (
        "local-alias-elided",
        "type R<'a>=&'a u8;fn f(x:&u8)->&u8{let y:R=x;y}",
    ),
    (
        "local-nominal",
        "struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->&'a u8{let y:S<'a>=S(x);y.0}",
    ),
    (
        "local-captured-pointer",
        "fn f<'a>(p:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{let q:fn(&'a u8)->&'a u8=p;q(x)}",
    ),
    (
        "local-nested-pointer",
        "fn f<'a>(p:for<'b>fn(&'b u8,&'a u8)->&'b u8,x:&'a u8)->&'a u8{let q:for<'b>fn(&'b u8,&'a u8)->&'b u8=p;q(x,x)}",
    ),
    (
        "local-reassignment",
        "fn f<'a,'b>(x:&'a u8,y:&'b u8){let mut z=x;z=y;let _=z;}",
    ),
    (
        "local-join",
        "fn f<'a,'b>(x:&'a u8,y:&'b u8,c:bool){let z=if c{x}else{y};let _=z;}",
    ),
    (
        "call-identity",
        "fn id(x:&u8)->&u8{x}fn f(x:&u8)->&u8{id(x)}",
    ),
    (
        "call-independent",
        "fn id(x:&u8)->&u8{x}fn f<'a,'b>(x:&'a u8,y:&'b u8){let i=id;let _a=i(x);let _b=i(y);}",
    ),
    (
        "call-early",
        "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f(x:&u8)->&u8{id(x)}",
    ),
    (
        "call-explicit-early",
        "fn id<'a:'a>(x:&'a u8)->&'a u8{x}fn f<'b>(x:&'b u8)->&'b u8{id::<'b>(x)}",
    ),
    (
        "call-output",
        "fn f<'a>()->&'a u8{&7}fn g()->&'static u8{f::<'static>()}",
    ),
    (
        "call-higher-rank-pointer",
        "fn f<'a>(g:for<'b>fn(&'b u8)->&'b u8,x:&'a u8)->&'a u8{g(x)}",
    ),
    (
        "call-captured-pointer",
        "fn f<'a>(g:fn(&'a u8)->&'a u8,x:&'a u8)->&'a u8{g(x)}",
    ),
    (
        "call-constructor",
        "struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->S<'a>{S(x)}",
    ),
    (
        "call-constructor-explicit",
        "struct S<'a>(&'a u8);fn f<'a>(x:&'a u8)->S<'a>{S::<'a>(x)}",
    ),
    (
        "call-enum",
        "enum E<'a>{A(&'a u8),B}fn f<'a>(x:&'a u8)->E<'a>{E::A(x)}",
    ),
    (
        "call-pointer-coercion",
        "fn id(x:&u8)->&u8{x}fn f<'a>(x:&'a u8)->&'a u8{let p:for<'b>fn(&'b u8)->&'b u8=id;p(x)}",
    ),
    ("bounded-return", "fn f<'a:'b,'b>(x:&'a u8)->&'b u8{x}"),
    (
        "where-return",
        "fn f<'a,'b>(x:&'a u8)->&'b u8 where 'a:'b{x}",
    ),
    (
        "transitive-return",
        "fn f<'a:'b,'b:'c,'c>(x:&'a u8)->&'c u8{x}",
    ),
    ("output-only-never", "fn f<'a>()->&'a u8{loop{}}"),
    ("output-only-promoted", "fn f<'a>()->&'a u8{&7}"),
    (
        "output-with-erased-input",
        "type U<'a>=u8;fn f<'a>(_:U<'a>)->&'a u8{&7}",
    ),
    (
        "elided-erased-alias",
        "type U<'a>=u8;fn f<'a>(_:U<'a>)->&u8{&7}",
    ),
    (
        "elided-carrying-alias",
        "type R<'a>=&'a u8;fn f(x:R)->&u8{x}",
    ),
    ("nominal", "struct S<'a>(&'a u8);"),
    ("elided", "struct S<'a>(&'a u8);fn f(x:S)->&u8{x.0}"),
    (
        "selected",
        "struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,x:S<'b>)->&'b u8{x.0}",
    ),
    (
        "auto-deref",
        "struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,x:&S<'b>)->&'b u8{x.0}",
    ),
    (
        "record",
        "struct S<'a>{value:&'a u8}fn f<'a,'b>(_:&'a u8,x:S<'b>)->&'b u8{x.value}",
    ),
    (
        "tuple-pattern",
        "struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,S(x):S<'b>)->&'b u8{x}",
    ),
    (
        "record-pattern",
        "struct S<'a>{value:&'a u8}fn f<'a,'b>(_:&'a u8,S{value:x}:S<'b>)->&'b u8{x}",
    ),
    (
        "enum-pattern",
        "enum E<'a>{A(&'a u8),B}fn f<'a>(x:E<'a>,y:&'a u8)->&'a u8{match x{E::A(v)=>v,E::B=>y}}",
    ),
    (
        "nested",
        "struct S<'a>(&'a u8);struct W<'a>(S<'a>);fn f<'a,'b>(_:&'a u8,x:W<'b>)->&'b u8{x.0.0}",
    ),
    ("alias", "type R<'a>=&'a u8;fn f<'a>(x:R<'a>)->R<'a>{x}"),
    ("unused-alias", "type U<'a>=u8;fn f(x:U<'static>)->u8{x}"),
    (
        "forward-alias",
        "type A<'a>=B<'a>;type B<'b>=&'b u8;fn f<'a>(x:A<'a>)->&'a u8{x}",
    ),
    (
        "alias-nominal",
        "struct S<'a>(&'a u8);type R<'a>=S<'a>;fn f<'a>(x:R<'a>)->&'a u8{x.0}",
    ),
    (
        "record-create",
        "struct S<'a>{value:&'a u8}fn f<'a>(x:&'a u8)->S<'a>{S{value:x}}",
    ),
    (
        "explicit-create",
        "struct S<'a>{value:&'a u8}fn f<'a>(x:&'a u8)->S<'a>{S::<'a>{value:x}}",
    ),
    (
        "two-parameters",
        "struct S<'a,'b>{x:&'a u8,y:&'b u8}fn f<'a,'b>(x:&'a u8,y:&'b u8)->S<'a,'b>{S{x,y}}",
    ),
    (
        "module",
        "mod m{pub struct S<'a>(pub &'a u8);}fn f<'a>(x:m::S<'a>)->&'a u8{x.0}",
    ),
    ("recursive", "struct S<'a>{x:&'a u8,next:&'a S<'a>}"),
    (
        "higher-rank-alias",
        "type F<'a>=for<'b>fn(&'a u8,&'b u8)->&'a u8;fn f<'a>(x:F<'a>)->F<'a>{x}",
    ),
];
const REJECTED: &[(&str, &str)] = &[
    (
        "partial-pointer-output-binder",
        "fn id(x:&u8)->&u8{x}fn f(){let p:for<'a>fn(_)->&'a u8=id;}",
    ),
    (
        "enum-both",
        "enum E<'a>{A(&'a u8),B}fn f<'a>(x:&'a u8)->E<'a>{E::<'a>::A::<'a>(x)}",
    ),
    (
        "enum-empty-owner",
        "enum E<'a>{A(&'a u8)}fn f<'a>(x:&'a u8)->E<'a>{E::<>::A::<'a>(x)}",
    ),
    (
        "generic-module",
        "mod m{pub struct S<'a>(pub &'a u8);}fn f<'a>(x:&'a u8)->m::S<'a>{m::<'a>::S(x)}",
    ),
    (
        "late-explicit",
        "fn id<'a>(x:&'a u8)->&'a u8{x}fn f(){id::<'static>(&7);}",
    ),
    (
        "unused-late-explicit",
        "fn f<'a>(){}fn g(){f::<'static>();}",
    ),
    ("extra-function-lifetime", "fn f(){}fn g(){f::<'static>();}"),
    (
        "erased-fnptr-output",
        "type U<'a>=u8;type F=for<'a>fn(U<'a>)->&'a u8;",
    ),
    (
        "two-alias-elision-candidates",
        "type R<'a,'b>=&'a u8;fn f<'a>(x:R<'a,'static>)->&u8{x}",
    ),
    ("unused-nominal", "struct S<'a>(u8);"),
    ("erased-nominal", "type U<'a>=u8;struct S<'a>(U<'a>);"),
    ("recursive-only", "struct S<'a>{next:*const S<'a>}"),
    (
        "mutually-recursive-only",
        "struct A<'a>{next:*const B<'a>}struct B<'b>{next:*const A<'b>}",
    ),
    ("missing-field-lifetime", "struct S<'a>(&u8);"),
    (
        "too-many",
        "struct S<'a>(&'a u8);fn f(_:S<'static,'static>){}",
    ),
    (
        "too-few",
        "struct S<'a,'b>(&'a u8,&'b u8);fn f(_:S<'static>){}",
    ),
    ("nongeneric", "struct S;fn f(_:S<'static>){}"),
    ("wrong-argument", "struct S<'a>(&'a u8);fn f(_:S<u8>){}"),
    ("missing-alias-lifetime", "type S<'a>=&u8;"),
    ("recursive-alias", "type S<'a>=S<'a>;"),
];

#[test]
fn lifetime_declarations_and_projection_types_agree_with_rustc()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/lifetime-declarations-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    for (accepted, cases) in [(true, ACCEPTED), (false, REJECTED)] {
        for (name, source) in cases {
            let source_path = directory.join(format!("{name}.rs"));
            std::fs::write(&source_path, source)?;
            let result = std::process::Command::new("rustc")
                .args([
                    "+1.98.0",
                    "--edition=2024",
                    "--crate-type=lib",
                    "--crate-name=probe",
                    "--emit=metadata",
                    "--cap-lints=allow",
                    "-o",
                ])
                .arg(directory.join("probe.rmeta"))
                .arg(&source_path)
                .output()?;
            assert_eq!(
                result.status.code(),
                Some(i32::from(!accepted)),
                "rustc {name}: {}",
                String::from_utf8_lossy(&result.stderr)
            );
            let result = semantic::analyze(source);
            assert_eq!(
                result.is_ok(),
                accepted,
                "RustGM {name}: {:?}",
                result.as_ref().err()
            );
        }
    }
    Ok(())
}

#[test]
fn selected_instance_fields_use_the_argument_region() -> Result<(), String> {
    let (_, facts) =
        semantic::analyze("struct S<'a>(&'a u8);fn f<'a,'b>(_:&'a u8,x:S<'b>)->&'b u8{x.0}")?;
    let signature = facts.functions.first().ok_or("missing function")?.signature;
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("missing signature".into());
    };
    let fields: Vec<_> = facts
        .nodes
        .iter()
        .flatten()
        .filter(|fact| matches!(fact.operation, Some(Operation::Field { .. })))
        .collect();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].ty, Some(result));
    let Some(Kind::Reference { region, .. }) = facts.types.kind(result) else {
        return Err("missing reference".into());
    };
    assert_eq!(region.bound_parts(), Some((0, 1)));
    Ok(())
}

#[test]
fn nominal_variance_propagates_through_recursive_dependencies() -> Result<(), String> {
    use rustgm::types::{ItemId, variance::Variance};
    let source = "struct Co<'a>(&'a u8);struct Contra<'a>(fn(&'a u8));struct Both<'a>(fn(&'a u8)->&'a u8);struct Mut<'a,'b>(&'a mut &'b u8);struct Reverse<'a>(fn(Contra<'a>));struct A<'a>{next:*const B<'a>}struct B<'b>{next:*const A<'b>,value:&'b u8}struct Cycle<'a>{next:fn(Cycle<'a>),value:&'a u8}";
    let (report, facts) = semantic::analyze(source)?;
    let expected: &[(&str, &[Variance])] = &[
        ("Co", &[Variance::Covariant]),
        ("Contra", &[Variance::Contravariant]),
        ("Both", &[Variance::Invariant]),
        ("Mut", &[Variance::Covariant, Variance::Invariant]),
        ("Reverse", &[Variance::Covariant]),
        ("A", &[Variance::Covariant]),
        ("B", &[Variance::Covariant]),
        ("Cycle", &[Variance::Invariant]),
    ];
    for (name, expected) in expected {
        let (index, _) = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, definition)| report.package.names.get(definition.name) == *name)
            .ok_or_else(|| format!("missing {name}"))?;
        let item = ItemId(u32::try_from(index).map_err(|e| e.to_string())?);
        assert_eq!(
            facts.lifetimes.variances.get(item),
            Some(*expected),
            "{name}"
        );
    }
    assert!(facts.lifetimes.variances.steps < 200);
    Ok(())
}

#[test]
fn function_schemes_separate_binding_time_from_elision_candidates() -> Result<(), String> {
    let cases = [
        ("fn f<'a>(x:&'a u8)->&'a u8{x}", 0, 1, true),
        ("fn f<'a:'a>(x:&'a u8)->&'a u8{x}", 1, 0, false),
        ("fn f<'a>()->&'a u8{&7}", 1, 0, false),
        ("fn f<'a>(){}", 0, 0, true),
        ("type U<'a>=u8;fn f<'a>(_:U<'a>){}", 0, 0, true),
        ("type U<'a>=u8;fn f<'a>(_:U<'a>)->&'a u8{&7}", 1, 0, false),
        ("fn f<'a:'a,'b>(x:&'a u8,y:&'b u8)->&'b u8{y}", 1, 1, true),
    ];
    for (source, early, late, declared_late) in cases {
        let (_, facts) = semantic::analyze(source)?;
        let function = facts.functions.first().ok_or("missing function")?;
        let scheme = facts
            .lifetimes
            .functions
            .get(&function.definition)
            .ok_or("missing lifetime scheme")?;
        assert_eq!(
            (
                scheme.early.len(),
                scheme.late.len(),
                scheme.has_declared_late
            ),
            (early, late, declared_late),
            "{source}"
        );
    }
    Ok(())
}
