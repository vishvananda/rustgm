//! PA20 solver controls qualify failure reuse without changing caller permissions.
#[path = "harness/native_references.rs"]
mod native;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::resolve::model::{DefId, DefKind};

const CASES: &[(&str, &str, bool)] = &[
    (
        "solver-failed-alternatives",
        "trait A<U>{}trait B{}impl<T:Copy>B for T{}struct N;impl<T:B>A<u8>for T{}impl<T:B>A<u16>for T{}impl<T>A<()>for T{}fn get<T:A<U>,U>(_:T){}fn main(){get::<_,_>(N)}",
        true,
    ),
    (
        "solver-all-alternatives-fail",
        "trait A<U>{}trait B{}impl<T:Copy>B for T{}struct N;impl<T:B>A<u8>for T{}impl<T:B>A<u16>for T{}fn get<T:A<U>,U>(_:T){}fn main(){get::<_,_>(N)}",
        false,
    ),
    (
        "solver-alternatives-become-ambiguous",
        "trait A<U>{}trait B{}impl<T:Copy>B for T{}impl<T:B>A<u8>for T{}impl<T:B>A<u16>for T{}impl<T>A<()>for T{}fn get<T:A<U>,U>(_:T){}fn main(){get::<_,_>(7u8)}",
        false,
    ),
    (
        "solver-other-body-permission",
        "trait A<U>{}trait B{}impl<T:Copy>B for T{}struct N;impl<T:B>A<u8>for T{}impl<T:A<u8>>A<u16>for &T{}impl<T>A<()>for T{}fn get<T:A<U>,U>(_:T){}fn first(){get::<_,_>(N)}fn second<T:Copy>(x:T){get::<_,u8>(x)}fn main(){first();second(7u8)}",
        true,
    ),
    (
        "solver-other-body-missing-permission",
        "trait A<U>{}trait B{}impl<T:Copy>B for T{}struct N;impl<T:B>A<u8>for T{}impl<T>A<()>for T{}fn get<T:A<U>,U>(_:T){}fn first(){get::<_,_>(N)}fn second<T>(x:T){get::<_,u8>(x)}fn main(){}",
        false,
    ),
    (
        "solver-complete-trait-arguments",
        "trait B<U>{}impl B<u16>for u8{}trait A<U>{}impl<T:B<u8>>A<u8>for T{}impl<T:B<u16>>A<u16>for T{}fn get<T:A<U>,U>(_:T){}fn main(){get::<_,_>(7u8)}",
        true,
    ),
    (
        "solver-complete-const-arguments",
        "trait B<const N:usize>{}impl B<2>for u8{}trait A<U>{}impl<T:B<1>>A<u8>for T{}impl<T:B<2>>A<u16>for T{}fn get<T:A<U>,U>(_:T){}fn main(){get::<_,_>(7u8)}",
        true,
    ),
    (
        "solver-cycle-is-not-evidence",
        "trait A{}trait B{}impl<T:B>A for T{}impl<T:A>B for T{}fn get<T:A>(_:T){}fn main(){get(7u8)}",
        false,
    ),
    (
        "solver-cycle-has-caller-base",
        "trait A{}trait B{}impl<T:B>A for T{}impl<T:A>B for T{}fn get<T:A>(_:T){}fn caller<T:B>(x:T){get(x)}fn main(){}",
        true,
    ),
    (
        "solver-productive-recursion",
        "trait A{}impl A for u8{}struct S<T>(T);impl<T:A>A for S<T>{}fn get<T:A>(_:T){}fn main(){get(S(S(S(7u8))))}",
        true,
    ),
    (
        "solver-recursion-missing-base",
        "trait A{}struct S<T>(T);impl<T:A>A for S<T>{}fn get<T:A>(_:T){}fn main(){get(S(S(S(7u8))))}",
        false,
    ),
    (
        "solver-cached-evidence-retains-lifetime-proof",
        "trait A<'a>{}impl<'a,T:'a>A<'a>for T{}fn get<T:A<'static>>(_:T){}fn main(){let x=7u8;get(&x)}",
        false,
    ),
];

#[test]
fn independent_solver_outcomes() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(name, source, accepted)?;
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

#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn inferred_selection_executes_the_successful_candidate() -> Result<(), Box<dyn std::error::Error>>
{
    native::check_cases(
        "solver-selection",
        &[
            (
                "failed-alternatives",
                "trait A<U>{fn value(self)->u8;}trait B{}impl<T:Copy>B for T{}struct N;impl<T:B>A<u8>for T{fn value(self)->u8{1}}impl<T:B>A<u16>for T{fn value(self)->u8{2}}impl<T>A<()>for T{fn value(self)->u8{7}}fn get<T:A<U>,U>(x:T)->u8{x.value()}fn main(){check(get::<_,_>(N)==7);check(get::<_,u8>(9u8)==1);check(get::<_,u16>(9u8)==2)}",
                false,
            ),
            (
                "distinct-const-goals",
                "trait B<const N:usize>{}impl B<2>for u8{}trait A<U>{fn value(self)->u8;}impl<T:B<1>>A<u8>for T{fn value(self)->u8{1}}impl<T:B<2>>A<u16>for T{fn value(self)->u8{7}}fn get<T:A<U>,U>(x:T)->u8{x.value()}fn main(){check(get::<_,_>(9u8)==7)}",
                false,
            ),
        ],
    )
}
