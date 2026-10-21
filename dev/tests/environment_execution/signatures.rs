//! Captured output lifetimes remain distinct from a callable's quantified inputs.
use super::{checked_sources, native, semantic};
use rustgm::types::{Kind, RegionId, Store, TypeId};
use std::path::PathBuf;

const CASES: &[(&str, &str)] = &[
    ("move-output", ""),
    ("shared-output", ""),
    ("tuple-output", ""),
    ("quantified-input", ""),
    ("mixed-output", ""),
    ("nested-callable", ""),
    (
        "local-output",
        "required lifetime does not outlive its target",
    ),
    (
        "static-escape",
        "required lifetime does not outlive its target",
    ),
    (
        "captured-escape",
        "access conflicts with a live reference loan",
    ),
    (
        "input-escape",
        "required lifetime does not outlive its target",
    ),
    (
        "parent-binder-collision",
        "required lifetime does not outlive its target",
    ),
];
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../pa24/tests/spec/execution-self/{name}.rs"))
}

#[test]
fn captured_output_context_matches_independent_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    let evidence = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa24/execution-self/source/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, reason) in CASES {
        let status = i32::from(!reason.is_empty());
        if status == 0 {
            checked_sources::check(&fixture(name), &evidence, name, (0, 0, reason))?;
        } else {
            checked_sources::check_modes(&fixture(name), &evidence, name, (1, [0, 1, 1], reason))?;
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn captured_outputs_execute_with_nested_callable_binders() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, reason) in CASES {
        if reason.is_empty() {
            let source = std::fs::read_to_string(fixture(name))?;
            let source = source
                .strip_prefix(native::CHECK)
                .ok_or("missing source check")?;
            native::check_cases(
                "parallel/pa24/execution-self/native",
                &[(name, source, false)],
            )?;
        }
    }
    Ok(())
}

fn result(types: &Store, signature: TypeId) -> Result<TypeId, &'static str> {
    let Some(Kind::Function { result, .. }) = types.kind(signature) else {
        return Err("missing callable signature");
    };
    Ok(result)
}
fn pair(types: &Store, ty: TypeId) -> Result<(RegionId, RegionId), &'static str> {
    let Some(Kind::Tuple(list)) = types.kind(ty) else {
        return Err("missing tuple result");
    };
    let Some([a, b]) = types.list(list) else {
        return Err("missing result pair");
    };
    let Some(Kind::Reference { region: a, .. }) = types.kind(*a) else {
        return Err("missing first reference");
    };
    let Some(Kind::Reference { region: b, .. }) = types.kind(*b) else {
        return Err("missing second reference");
    };
    Ok((a, b))
}

#[test]
fn public_input_and_captured_output_use_separate_binders() -> Result<(), Box<dyn std::error::Error>>
{
    let source = std::fs::read_to_string(fixture("mixed-output"))?;
    let (_, facts) = semantic::analyze(&source)?;
    let [body] = facts.closures.all() else {
        return Err("expected one closure".into());
    };
    let execution = body.execution.ok_or("missing captured execution")?;
    let Some(Kind::Closure { signature, .. }) = facts.types.kind(execution.self_ty) else {
        return Err("missing execution Self".into());
    };
    assert!(matches!(
        facts.types.kind(signature),
        Some(Kind::Function {
            bound_regions: 1,
            ..
        })
    ));
    let (input, output) = pair(&facts.types, result(&facts.types, signature)?)?;
    let (_, body_output) = pair(&facts.types, result(&facts.types, body.signature)?)?;
    assert_eq!(input.bound_parts(), Some((0, 0)));
    let (depth, slot) = body_output
        .bound_parts()
        .ok_or("missing body output slot")?;
    assert_eq!(depth, 0);
    assert_eq!(output.bound_parts(), Some((1, slot)));
    let link = facts
        .nested_outputs
        .as_deref()
        .and_then(|links| links.get(&body.source))
        .ok_or("missing conditional output link")?;
    assert!(
        link.bridges
            .iter()
            .any(|bridge| bridge.output && bridge.slot == slot)
    );
    assert!(
        link.bridges
            .iter()
            .all(|bridge| bridge.parent.bound_parts().is_none())
    );
    Ok(())
}

#[test]
fn independent_captured_outputs_keep_additive_interface_work()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut source = "fn infer<'a,F:Fn()->&'a u8>(f:F)->F{f}fn main(){".to_owned();
        for index in 0..width {
            write!(
                source,
                "let n{index}=7u8;let r{index}=&n{index};let f{index}=infer(move||r{index});let _=f{index}();"
            )?;
        }
        source.push('}');
        let (_, facts) = semantic::analyze(&source)?;
        assert_eq!(facts.closures.all().len(), width);
        let links = facts
            .nested_outputs
            .as_deref()
            .ok_or("missing output interfaces")?;
        assert!(links.ports <= 2 * width && links.bridges <= 4 * width);
        assert!(facts.body_work.canonical_steps < 8192 * (width + 1));
        observations.push(facts.body_work.canonical_steps);
    }
    assert!(observations[2] - observations[1] <= 5 * (observations[1] - observations[0]));
    Ok(())
}
