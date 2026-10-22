use crate::qualify as rustc;
use crate::semantic;
use rustgm::{body, borrow::ownership::initialization};

#[test]
fn custom_destructors_preserve_whole_objects_but_allow_copy_fields_and_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let common = "struct Leaf;impl Drop for Leaf{fn drop(&mut self){}}struct Outer(Leaf,u8);impl Drop for Outer{fn drop(&mut self){}}";
    let cases = [
        ("whole", true, "fn f(x:Outer){let y=x;}"),
        ("copy-field", true, "fn f(x:Outer){let y=x.1;}"),
        ("borrow-field", true, "fn f(x:Outer){let Outer(ref y,_)=x;}"),
        ("replace-field", true, "fn f(mut x:Outer){x.0=Leaf;}"),
        ("non-drop-parent", true, "fn f(x:(Outer,u8)){let y=x.0;}"),
        (
            "array-whole-element",
            true,
            "fn f(x:[Outer;2]){let [y,..]=x;}",
        ),
        ("field", false, "fn f(x:Outer){let y=x.0;}"),
        ("pattern", false, "fn f(x:Outer){let Outer(y,_)=x;}"),
        ("nested", false, "fn f(x:(Outer,u8)){let y=x.0.0;}"),
        ("temporary", false, "fn f(){let y=Outer(Leaf,0).0;}"),
        (
            "array-pattern",
            false,
            "fn f(x:[Outer;2]){let [Outer(y,_),..]=x;}",
        ),
        (
            "enum-payload",
            false,
            "enum E{A(Leaf),B}impl Drop for E{fn drop(&mut self){}}fn f(x:E){match x{E::A(y)=>(),E::B=>()}}",
        ),
        (
            "enum-borrow",
            true,
            "enum E{A(Leaf),B}impl Drop for E{fn drop(&mut self){}}fn f(x:E){match x{E::A(ref y)=>(),E::B=>()}}",
        ),
        (
            "mutable-reference",
            false,
            "struct R<'a>(&'a mut u8);impl Drop for R<'_>{fn drop(&mut self){}}fn f(x:R<'_>){let y=x.0;}",
        ),
        (
            "empty-array",
            false,
            "struct Z([Leaf;0]);impl Drop for Z{fn drop(&mut self){}}fn f(x:Z){let y=x.0;}",
        ),
    ];
    for (name, expected, case) in cases {
        let source = format!("{common}{case}");
        rustc::qualify(name, &source, expected)?;
        let (report, mut facts) = semantic::analyze(&source)?;
        let program = body::build(&report, &facts)?;
        let custom = facts
            .destructors
            .implementations()
            .iter()
            .map(|i| i.item)
            .collect::<std::collections::HashSet<_>>();
        let mut rejected = false;
        for cfg in &program.bodies {
            let init = initialization::analyze(
                &facts.types,
                &facts.aggregates,
                &facts.derives,
                &mut facts.copies,
                cfg,
            )
            .map_err(|e| format!("{name}: {}", e.message))?;
            if let Err(error) = init.check_destructor_moves(&facts.types, &custom) {
                assert_eq!(
                    error.message,
                    "cannot move a non-Copy field out of a type that implements Drop",
                    "{name}"
                );
                rejected = true;
            }
        }
        assert_eq!(!rejected, expected, "{name}");
    }
    Ok(())
}
