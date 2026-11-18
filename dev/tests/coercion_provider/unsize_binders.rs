//! Quantified Unsize children retain their lifetime proof through executable selection.
use super::TestResult;

#[test]
fn quantified_object_goals_keep_their_lifetime_obligations() -> TestResult {
    let path = "pa31/tests/spec/unsizing-roles/object-goal.rs";
    let input = crate::resolve::fixture(path)?;
    let (prefix, _) = input
        .split_once("fn require<T:")
        .ok_or("Unsize provider prefix")?;
    let cases = [
        (
            "principal",
            "trait Lend<'a>{} struct Subject; impl<'a>Lend<'a> for Subject{}",
            "require::<Subject,dyn for<'a>Lend<'a>>();",
            true,
        ),
        (
            "associated",
            "trait Lend<'a>{type Out;} struct Subject; impl<'a>Lend<'a> for Subject{type Out=&'a u8;}",
            "require::<Subject,dyn for<'a>Lend<'a,Out=&'a u8>>();",
            true,
        ),
        (
            "caller",
            "trait Lend<'a>{} struct Subject; impl<'a>Lend<'a> for Subject{}
         fn use_bound<T:'static+for<'a>Lend<'a>>(){require::<T,dyn for<'a>Lend<'a>>();}",
            "use_bound::<Subject>();",
            true,
        ),
        (
            "static-only",
            "trait Lend<'a>{} struct Subject; impl Lend<'static> for Subject{}",
            "require::<Subject,dyn for<'a>Lend<'a>>();",
            false,
        ),
        (
            "correlation",
            "trait Lend<'a,'b>{} struct Subject; impl<'a>Lend<'a,'a> for Subject{}",
            "require::<Subject,dyn for<'a,'b>Lend<'a,'b>>();",
            false,
        ),
        (
            "wrong-associated",
            "trait Lend<'a>{type Out;} struct Subject; impl<'a>Lend<'a> for Subject{type Out=&'static u8;}",
            "require::<Subject,dyn for<'a>Lend<'a,Out=&'a u8>>();",
            false,
        ),
    ];
    let mut failures = Vec::new();
    for (name, declarations, body, accepted) in cases {
        let source = format!(
            "{prefix}
             fn require<T:PointeeSized+Unsize<U>,U:PointeeSized>(){{}}
             {declarations}
             #[unsafe(no_mangle)] pub extern \"C\" fn observe()->u8{{{body}7}}"
        );
        match crate::ffi_program::exported(&source, "quantified_object_goals") {
            Ok(program) if accepted => {
                assert!(
                    program
                        .symbols
                        .iter()
                        .any(|s| s.name == "observe" && s.body.is_some()),
                    "{name}"
                );
            }
            Err(error) if !accepted && error.to_string().contains("lifetime") => {}
            result => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".to_owned()),
            )),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
