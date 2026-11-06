//! Exact checked emitters retain their semantic causes; bookkeeping is not routing.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn source_facts(role: &str) -> Result<crate::sema::model::Declarations> {
    let text = crate::test_source::fixture(
        "pa31/tests/spec/assertion-panics/unused-bounds-alternative.rs",
    )?
    .replace("panic_bounds_check", role);
    let report = crate::test_source::source_provider(&text, "guard_emitter")?;
    Ok(crate::sema::declarations::analyze(&report)?)
}

#[test]
fn checked_wide_multiply_keeps_multiply_cause_for_auxiliary_division_guards() -> Result {
    for (role, blocked) in [
        ("panic_const_mul_overflow", true),
        ("panic_const_div_overflow", false),
    ] {
        let facts = source_facts(role)?;
        let mut builder = Builder::default();
        let entry = builder.block()?;
        builder.enter(entry)?;
        let lhs = builder.integer(Type::I128, 3)?;
        let rhs = builder.integer(Type::I128, 7)?;
        let mut panic = None;
        let result = Arithmetic {
            builder: &mut builder,
            panic: &mut panic,
            constants: Reader::new(&facts),
        }
        .binary(Punctuation::Star, Primitive::I128, lhs, rhs);
        assert_eq!(result.is_err(), blocked, "{role}");
        if blocked {
            assert_eq!(panic, None, "source hook acquired a legacy abort block");
            assert!(
                result
                    .err()
                    .ok_or("boundary")?
                    .contains("checked source panic")
            );
        } else {
            assert!(panic.is_some());
        }
    }
    Ok(())
}

#[test]
fn flag_only_subtraction_and_literal_negation_emit_no_panic_guard() -> Result {
    let facts = source_facts("panic_const_sub_overflow")?;
    let mut builder = Builder::default();
    let entry = builder.block()?;
    builder.enter(entry)?;
    let lhs = builder.integer(Type::I8, 13)?;
    let rhs = builder.integer(Type::I8, 7)?;
    let mut panic = None;
    Arithmetic {
        builder: &mut builder,
        panic: &mut panic,
        constants: Reader::new(&facts),
    }
    .subtract_overflow(false, lhs, rhs)?;
    assert_eq!(panic, None);
    let negation = source_facts("panic_const_neg_overflow")?;
    Arithmetic {
        builder: &mut builder,
        panic: &mut panic,
        constants: Reader::new(&negation),
    }
    .unary(ast::Unary::Negate, Primitive::I8, lhs, true)?;
    assert_eq!(panic, None);
    assert_eq!(size_of::<Arithmetic<'_, '_>>(), 24);
    assert_eq!(size_of::<Reader<'_>>(), 8);
    Ok(())
}
