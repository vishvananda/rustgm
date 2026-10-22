//! Opaque bounds complete declaration environments without inventing result properties.
use crate::qualify as rustc;
use crate::semantic;

#[test]
fn projected_declaration_bounds_retain_formation_and_sizedness_requirements()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "opaque-function-bound",
            "trait A{type Out;}trait C{}fn f<T:A>()where <T as A>::Out:C{}fn main(){}",
            true,
        ),
        (
            "unused-projected-trait-impl",
            "trait A{type Out;}trait C{}struct W<T>(T);trait B{}impl<T:A>B for W<T>where <T as A>::Out:C{}fn main(){}",
            true,
        ),
        (
            "unused-projected-inherent-impl",
            "trait A{type Out;}trait C{}struct W<T>(T);impl<T:A>W<T>where <T as A>::Out:C{fn get()->u8{7}}fn main(){}",
            true,
        ),
        (
            "unsized-projected-array",
            "trait A{type Out:?Sized;}trait C{}fn f<T:A>()where [<T as A>::Out;1]:C{}fn main(){}",
            false,
        ),
        (
            "explicitly-sized-projected-array",
            "trait A{type Out:?Sized;}trait C{}fn f<T:A>()where <T as A>::Out:Sized,[<T as A>::Out;1]:C{}fn main(){}",
            true,
        ),
        (
            "wrong-projection-trait-argument",
            "trait A<P>{type Out;}trait C{}fn f<T:A<u8>>()where <T as A<u16>>::Out:C{}fn main(){}",
            false,
        ),
        (
            "separate-projection-trait-arguments",
            "trait A<P>{type Out;}trait C{}fn f<T:A<u8>+A<u16>>()where <T as A<u16>>::Out:C{}fn main(){}",
            true,
        ),
    ];
    for (name, source, accepted) in cases {
        rustc::qualify(&format!("projected-declaration-{name}"), source, accepted)?;
        let result = semantic::analyze(source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    Ok(())
}
