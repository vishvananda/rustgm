//! Declaration-phase constants relate lifetime nominals by their declared
//! variance and outlives contracts before the crate-wide inference runs.
type TestResult = Result<(), Box<dyn std::error::Error>>;

const CONTROLS: &str = "pa31/tests/spec/declaration-lifetime-contracts";

/// Independently checked rustc outcomes: `None` accepts, otherwise the first
/// error this compiler reports for the control rustc also rejects.
const CASES: [(&str, Option<&str>); 11] = [
    ("field-length", None),
    ("signature-length", None),
    ("later-nominal-length", None),
    ("earlier-nominal-length", None),
    ("covariant-shrink-length", None),
    ("covariant-chain-length", None),
    ("implied-outlives-length", None),
    (
        "covariant-widen-length",
        Some("required lifetime does not outlive its target"),
    ),
    (
        "invariant-shrink-length",
        Some("required lifetime does not outlive its target"),
    ),
    (
        "outlives-escape-length",
        Some("required lifetime does not outlive its target"),
    ),
    (
        "invariant-chain-length",
        Some("required lifetime does not outlive its target"),
    ),
];

#[test]
fn declaration_constants_demand_reached_lifetime_contracts() -> TestResult {
    for (name, expected) in CASES {
        let path = format!("{CONTROLS}/{name}.rs");
        let source = crate::test_source::fixture(&path)?;
        let report = crate::test_source::source_provider(&source, "lifetime_contract")?;
        match (expected, crate::pipeline::analyze_source(&report)) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message, "{path}"),
            (None, Err(error)) => return Err(format!("{path}: rejected: {error}").into()),
            (Some(message), Ok(_)) => return Err(format!("{path}: accepted, not {message}").into()),
        }
    }
    Ok(())
}

/// The lang-item prefix of the declaration-time lifetime controls.
fn prefix() -> Result<String, Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(&format!("{CONTROLS}/field-length.rs"))?;
    let end = source
        .find("pub struct Note")
        .ok_or("lifetime control prefix")?;
    Ok(source[..end].to_owned())
}

/// A declaration-time constant whose callee relates `count` lifetime nominals,
/// each declared after the constant's owner. `wide` passes them as `count`
/// parameters `M<&L{index}<'_>>`; otherwise the parameter is `L0<'_>` and each
/// nominal names the next behind a reference, which storage does not expand.
fn waiting_nominals(count: usize, wide: bool) -> Result<String, Box<dyn std::error::Error>> {
    use std::fmt::Write as _;
    let mut source = prefix()?;
    source.push_str("pub enum M<T> { No, Yes(T) }\n");
    if wide {
        source.push_str("const fn mark(");
        for index in 0..count {
            write!(source, "a{index}: M<&L{index}<'_>>, ")?;
        }
        source.push_str(") -> usize { 3 }\nconst fn later() -> usize { mark(");
        for _ in 0..count {
            source.push_str("M::No, ");
        }
        source.push_str(") }\n");
    } else {
        source.push_str(
            "const fn mark(l: L0<'_>) -> usize { match l { L0 { next: _ } => 3 } }\n\
             const fn later() -> usize { mark(L0 { next: M::No }) }\n",
        );
    }
    source.push_str("pub struct Holder([u8; later()]);\n");
    for index in 0..count {
        if wide || index + 1 == count {
            writeln!(source, "pub struct L{index}<'a> {{ text: &'a str }}")?;
        } else {
            writeln!(
                source,
                "pub struct L{index}<'a> {{ next: M<&'a L{}<'a>> }}",
                index + 1
            )?;
        }
    }
    source.push_str("fn take(h: Holder) -> [u8; 3] { match h { Holder(a) => a } }\n");
    Ok(source)
}

/// Each wait for a nominal's field template resumes the body's contract
/// demand at that nominal: its unreached nominals are collected once and a
/// nominal's closure walk continues at the owner it waited for. A waiting
/// closure's graph pays the capacity it retains, so a chain of 512 nominals
/// that all wait in one body stays inside the input-graph bound, where the
/// conservative estimates exceeded it from about 256.
#[test]
fn waited_lifetime_contracts_resume_at_the_waited_nominal() -> TestResult {
    for wide in [true, false] {
        let mut costs = Vec::new();
        for count in [64, 256, 512] {
            let source = waiting_nominals(count, wide)?;
            let report = crate::test_source::source_provider(&source, "lifetime_contract")?;
            let analysis = crate::pipeline::analyze_source(&report)
                .map_err(|error| format!("wide={wide} count={count}: {error}"))?;
            let (work, bytes) =
                crate::sema::header_attributes::source_contract_costs(&analysis.facts);
            let bound = crate::sema::header_attributes::source_contract_bound(&analysis.facts);
            eprintln!(
                "lifetime contracts wide={wide} count={count} work={work} bytes={bytes} bound={bound:?}"
            );
            costs.push((work, bytes));
        }
        let [small, middle, large] = costs[..] else {
            return Err("three nominal counts".into());
        };
        // Near-linear increments: 192 nominals from 64 to 256 and 256 from 256
        // to 512. Geometric scratch growth pays each relocation once.
        for (name, [small, middle, large]) in [
            ("work", [small.0, middle.0, large.0]),
            ("bytes", [small.1, middle.1, large.1]),
        ] {
            assert!(
                (large - middle) * 192 < (middle - small) * 256 * 3 / 2,
                "wide={wide} contract {name} {small} {middle} {large} is not linear"
            );
        }
    }
    Ok(())
}
