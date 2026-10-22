//! Library operator identities select checked signatures and real native callables.
use crate::native_references as native;
use crate::qualify as rustc;
use crate::semantic;

const CASES: &[(&str, bool)] = &[
    ("index", true),
    ("index-argument-loan", false),
    ("moved-operand", false),
    ("index-write", false),
    ("add", true),
    ("neg", true),
    ("generic", true),
    ("alias", true),
    ("shadow", false),
    ("wrong-output", false),
    ("wrong-rhs", false),
];
#[test]
fn operator_contracts_match_independent_language_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let source = std::fs::read_to_string(format!(
            "{}/../pa26/tests/spec/operators/{name}.rs",
            env!("CARGO_MANIFEST_DIR")
        ))?;
        rustc::qualify(&format!("language-operator-{name}"), &source, accepted)?;
        let result = checked(&source);
        if result.is_ok() != accepted {
            failures.push(format!("{name}: {result:?}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
fn checked(source: &str) -> Result<(), String> {
    let (mut report, mut facts) = semantic::analyze(source)?;
    let entry = rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
    rustgm::lower::instances::build(&report, &mut facts, entry)
        .map(|_| ())
        .map_err(|e| e.to_string())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn operators_execute_selected_library_methods() -> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "language-operators",
        &[
            (
                "evaluation-order",
                "struct C(u8);impl core::ops::Add for C{type Output=u8;fn add(self,rhs:C)->u8{self.0*10+rhs.0}}fn mark(x:&mut u8)->C{*x+=1;C(*x)}fn main(){let mut x=0;check(mark(&mut x)+mark(&mut x)==12);check(x==2);}",
                false,
            ),
            (
                "add",
                "struct C(u8);impl core::ops::Add<u16> for C{type Output=u16;fn add(self,rhs:u16)->u16{self.0 as u16+rhs}}fn main(){check(C(3)+9u16==12);}",
                false,
            ),
            (
                "index",
                "struct P(u16,u16);impl core::ops::Index<bool> for P{type Output=u16;fn index(&self,x:bool)->&u16{if x{&self.1}else{&self.0}}}fn main(){let p=P(3,9);check(p[true]==9&&p[false]==3);}",
                false,
            ),
            (
                "neg",
                "struct C(i32);impl core::ops::Neg for C{type Output=i32;fn neg(self)->i32{-self.0}}fn main(){check(-C(7)==-7);}",
                false,
            ),
            (
                "generic",
                "fn add<T:core::ops::Add<u8,Output=u16>>(x:T)->u16{x+4u8}struct C(u16);impl core::ops::Add<u8> for C{type Output=u16;fn add(self,rhs:u8)->u16{self.0+rhs as u16}}fn main(){check(add(C(3))==7);}",
                false,
            ),
        ],
    )
}

#[test]
fn primitive_invalid_operators_preserve_existing_diagnostics()
-> Result<(), Box<dyn std::error::Error>> {
    for (path, diagnostic) in [
        (
            "pa12/tests/spec/reject/index-tuple.rs",
            "indexing requires a known array type",
        ),
        (
            "pa18/tests/spec/functions/reject-arithmetic.rs",
            "incompatible types",
        ),
        (
            "pa22/tests/spec/pointers/pointer-coercions/cases/pointer-add.rs",
            "binary operator is not defined for these scalar types",
        ),
    ] {
        let source = std::fs::read_to_string(format!("{}/../{path}", env!("CARGO_MANIFEST_DIR")))?;
        rustc::qualify(
            path.rsplit('/').next().ok_or("missing fixture name")?,
            &source,
            false,
        )?;
        let error = checked(&source)
            .err()
            .ok_or("invalid primitive operation was accepted")?;
        assert!(error.contains(diagnostic), "{path}: {error}");
    }
    Ok(())
}

#[path = "language_operators/mutation.rs"]
mod mutation;

#[path = "language_operators/assignment.rs"]
mod assignment;

#[path = "language_operators/primitives.rs"]
mod primitives;

#[path = "language_operators/methods.rs"]
mod methods;

#[path = "language_operators/addition.rs"]
mod addition;

#[path = "language_operators/binary.rs"]
mod binary;

#[path = "language_operators/comparison.rs"]
mod comparison;

#[path = "language_operators/ordering.rs"]
mod ordering;

#[path = "language_operators/extrema.rs"]
mod extrema;

#[path = "language_operators/ranks.rs"]
mod ranks;

#[path = "language_operators/referents.rs"]
mod referents;

#[path = "language_operators/lengths.rs"]
mod lengths;

#[path = "language_operators/collections.rs"]
mod collections;

#[path = "language_operators/lexicographic.rs"]
mod lexicographic;

#[path = "language_operators/limits.rs"]
mod limits;

#[path = "language_operators/iterators.rs"]
mod iterators;
