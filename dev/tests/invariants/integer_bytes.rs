//! Byte encoding cannot collapse an invalid Some payload into the inactive None variant.
use super::*;
#[test]
fn integer_byte_carriers_recheck_initialization_and_reject_forged_some_zero() -> Result<(), String>
{
    let input =
        crate::test_source::fixture("pa31/tests/spec/source-integer-niches/source-runtime.rs")
            .map_err(|e| e.to_string())?;
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(crate::support::diagnostics::LintCap::Allow);
    let report = crate::test_source::source_provider_config(&input, "integer_bytes", &config)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let owner = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == "echo_u8")
        .ok_or("echo")?;
    let Some(Kind::FunctionItem { signature, .. }) = facts
        .types
        .kind(facts.definitions[owner].ok_or("definition")?)
    else {
        return Err("function item".into());
    };
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("signature".into());
    };
    let ty = *facts
        .types
        .list(parameters)
        .and_then(|p| p.first())
        .ok_or("parameter")?;
    let mut machine = Machine::new(
        Evaluator::new(&report, &mut facts),
        std::iter::empty(),
        Limits::default(),
    )
    .map_err(str::to_owned)?;
    let check = (|| -> Result<(), Failure> {
        machine.footprint(ty)?;
        let niche = Reader::new(machine.evaluator.facts()).enum_niche(ty)?;
        let (empty, payload) = (niche.empty, niche.payload);
        let one = Constant::scalar(Scalar::Integer(1));
        let wrapper = machine.evaluator.fields(&[one], None)?;
        let some = machine.evaluator.fields(&[wrapper], Some(payload))?;
        let root = machine.allocate(&mut Vec::new(), Some(some), ty)?;
        let id = machine.convert_byte_root(root.location().ok_or("root")?)?;
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            payload
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] &= !1;
        assert!(
            machine
                .decode_nullable_variant(Source::Backing(id), 0, ty)
                .is_err()
        );
        machine.memory.bytes.roots[id.0 as usize].initialized[0] |= 1;
        machine.fill_byte_root(root.location().ok_or("root")?, 0, 1, 0)?;
        assert_eq!(
            machine.decode_nullable_variant(Source::Backing(id), 0, ty)?,
            empty
        );
        let zero = machine
            .evaluator
            .fields(&[Constant::scalar(Scalar::Integer(0))], None)?;
        let forged = machine.evaluator.fields(&[zero], Some(payload))?;
        let root = machine.allocate(&mut Vec::new(), Some(forged), ty)?;
        let error = machine
            .convert_byte_root(root.location().ok_or("forged root")?)
            .err()
            .ok_or("Some zero was encoded as None")?;
        assert!(
            error.message().contains("scalar pattern"),
            "{}",
            error.message()
        );
        Ok(())
    })();
    check.map_err(|e| e.message().to_owned())
}
