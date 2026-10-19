//! Partial constructors can expose later goals, without treating unfinished candidates as proof.
use std::fmt::Write;
#[test]
fn qualified_partial_call_controls_preserve_results_and_rejections()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa19/tests/spec/partial-inference");
    for (name, accepted) in [
        ("partial-method-result", true),
        ("reversed-bound-order", true),
        ("nested-partial-condition", true),
        ("partial-const-method-result", true),
        ("failed-selected-condition", false),
        ("ambiguous-distinct-headers", false),
        ("unknown-self-not-guessed", false),
    ] {
        let source = std::fs::read_to_string(directory.join(format!("{name}.rs")))?;
        super::cases(&[(name, &source, accepted)])?;
    }
    Ok(())
}

#[test]
fn reverse_body_dependencies_use_notifications_instead_of_whole_body_rounds()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [8, 32, 128, 512] {
        let mut source = String::from("trait Link<T>{}impl Link<u8>for u8{}fn need<");
        for i in 0..=count {
            if i != 0 {
                source.push(',');
            }
            write!(source, "T{i}")?;
        }
        write!(source, ">(_:T{count})where ")?;
        for i in 0..count {
            if i != 0 {
                source.push(',');
            }
            write!(source, "T{}:Link<T{i}>", i + 1)?;
        }
        source.push_str("{}fn main(){need(0u8);}");
        super::rustc::qualify(&format!("reverse-body-traits-{count}"), &source, true)?;
        let (_, facts) = super::semantic::analyze(&source)?;
        let selections = facts.body_work.trait_selections;
        eprintln!("reverse body chain count={count} selections={selections}");
        assert!(selections <= 3 * count + 16, "{selections} at {count}");
    }
    Ok(())
}
