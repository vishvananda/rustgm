//! Both entry points use one canonical table; failed cold payments cannot poison it.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn capacities(names: &Names) -> (usize, usize, usize, usize) {
    (
        names.bytes.capacity(),
        names.entries.capacity(),
        names.slots.capacity(),
        names.scratch.capacity(),
    )
}
#[test]
fn paid_and_ordinary_names_preserve_ids_capacities_and_atomic_failure() -> TestResult {
    let mut ordinary = Names::default();
    let mut paid = Names::default();
    let mut work = 0;
    let mut bytes = 0;
    for spelling in ["", "r#exact", "'exact", "é", "e\u{301}", "nul\0name"] {
        assert_eq!(
            ordinary.intern_exact(spelling)?,
            paid.intern_exact_paid(spelling, &mut |g| {
                work += g.work;
                bytes += g.bytes;
                Ok(())
            })?
        );
    }
    for index in 0..128 {
        let name = format!("prefix_{index}");
        assert_eq!(
            ordinary.intern_exact(&name)?,
            paid.intern_exact_paid(&name, &mut |g| {
                work += g.work;
                bytes += g.bytes;
                Ok(())
            })?
        );
    }
    assert_eq!(capacities(&ordinary), capacities(&paid));
    let before = capacities(&ordinary);
    for _ in 0..128 {
        ordinary.intern_exact("prefix_0")?;
    }
    assert_eq!(capacities(&ordinary), before);
    for limit in [0, 1, 16, 64, 128] {
        let mut names = Names::default();
        for index in 0..8 {
            names.intern_exact(&format!("seed_{index}"))?;
        }
        let before = (names.len(), names.byte_len(), capacities(&names));
        let mut attempted = 0;
        let result = names.intern_exact_paid("extra", &mut |growth| {
            attempted += growth.work;
            if attempted > limit {
                Err("no payment")
            } else {
                Ok(())
            }
        });
        if result.is_err() {
            assert_eq!((names.len(), names.byte_len(), capacities(&names)), before);
            for index in 0..8 {
                assert_eq!(names.intern_exact(&format!("seed_{index}"))?, NameId(index));
            }
        }
    }
    eprintln!(
        "paid Names={} names={} work={work} allocated_charge={bytes} capacities={:?}",
        size_of::<Names>(),
        paid.len(),
        capacities(&paid)
    );
    Ok(())
}
