//! A bound reference lifetime cannot erase its referent from a projected input.
use crate::qualify as rustc;
use crate::semantic;
const CASES: &[(&str, &str, bool)] = &[
    (
        "opaque-fn-input",
        "trait A{type Out;}fn need<X:'static>(){}fn f<T>()where fn(&T):A{need::<<fn(&T)as A>::Out>();}fn main(){}",
        false,
    ),
    (
        "static-fn-input",
        "trait A{type Out;}fn need<X:'static>(){}fn f<T:'static>()where fn(&T):A{need::<<fn(&T)as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "static-reference-input",
        "trait A{type Out;}fn need<X:'static>(){}fn f<'a:'static,T>(_:&'a T)where &'a T:A{need::<<&'a T as A>::Out>();}fn main(){}",
        true,
    ),
    (
        "bounded-function-alias",
        "trait A{type Out;}type F<T>=fn(&T);fn need<X:'static>(){}fn f<T>()where F<T>:A,<F<T>as A>::Out:'static{need::<<F<T>as A>::Out>();}fn main(){}",
        true,
    ),
];
#[test]
fn projected_inputs_preserve_referents_and_independent_result_bounds()
-> Result<(), Box<dyn std::error::Error>> {
    for &(name, source, accepted) in CASES {
        rustc::qualify(&format!("projected-input-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry = rustgm::sema::entry::select(&mut report, &facts)
                .map_err(|error| error.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
