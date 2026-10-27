//! Compact immutable attribute witnesses retain complete structural coverage.
use super::*;
use crate::{
    resolve::{expansion, package::Error},
    syntax::token_tree::TreeRange,
};
use std::fmt::Write as _;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ATTRIBUTES: &str = "/// retained\n#[doc=\"kept\"]#[inline]#[inline]#[allow(unsafe_op_in_unsafe_fn,unused_must_use)]";
fn owned(source: &str) -> TestResult<(expansion::Report, Producer)> {
    let mut report = super::tests::report(source)?;
    let producer = report
        .attribute_effects
        .as_deref_mut()
        .and_then(|effects| effects.policy.take())
        .ok_or("actual source producer")?;
    Ok((report, producer))
}
#[test]
fn exact_owner_arrays_keep_empty_witnesses_and_linear_payloads() -> TestResult {
    let mut unit_payload = None;
    for width in [1, 8, 32, 128] {
        let mut source = String::new();
        for index in 0..width {
            writeln!(source, "{ATTRIBUTES}fn f{index}(){{}}")?;
        }
        let (report, mut producer) = owned(&source)?;
        assert_eq!(producer.owners.len(), width);
        assert_eq!(
            producer.validate(&report.package).map_err(Error::from)?,
            (width, 2 * width, true)
        );
        let (mut headers, mut trees, mut bytes) = (0, 0, 0);
        for owner in &producer.owners {
            let parsed = &owner.parsed;
            assert_eq!(parsed.witnesses.len(), 5);
            assert_eq!(parsed.witnesses[0].tree_count, 0);
            assert_eq!(
                parsed
                    .witnesses
                    .iter()
                    .map(|row| row.tree_count as usize)
                    .sum::<usize>(),
                parsed.trees.len()
            );
            headers += parsed.witnesses.len();
            trees += parsed.trees.len();
            bytes += size_of_val(parsed.witnesses.as_ref()) + size_of_val(parsed.trees.as_ref());
        }
        assert_eq!(bytes, width * *unit_payload.get_or_insert(bytes));
        let key = producer.owners[0].key;
        let tail = producer.index[&key];
        let policy = producer
            .policy(&report.package, Some(tail), key)
            .map_err(Error::from)?;
        assert!(matches!(policy.operation, super::super::Level::Allow));
        assert!(matches!(policy.must_use, super::super::Level::Allow));
        let before = producer.work().compositions;
        producer
            .policy(&report.package, Some(tail), key)
            .map_err(Error::from)?;
        assert_eq!(producer.work().compositions, before);
        println!(
            "compact witnesses width={width} headers={headers} trees={trees} payload={bytes} owner={} witness={} work={} charged={}",
            size_of::<Owner>(),
            size_of::<Witness>(),
            producer.work().steps,
            producer.work().bytes
        );
    }
    Ok(())
}
#[test]
fn altered_counts_order_and_flat_tree_extent_fail_warm_validation() -> TestResult {
    for mutation in 0..9 {
        let (report, mut producer) = owned(&format!("{ATTRIBUTES}fn f(){{}}"))?;
        producer.validate(&report.package).map_err(Error::from)?;
        let key = producer.owners[0].key;
        let tail = producer.index[&key];
        producer
            .policy(&report.package, Some(tail), key)
            .map_err(Error::from)?;
        let parsed = &mut producer.owners[0].parsed;
        match mutation {
            0 => parsed.witnesses[2].tree_count = 0,
            1 => parsed.witnesses[2].tree_count = u32::MAX,
            2 => {
                parsed.witnesses[2].tree_count = 0;
                parsed.witnesses[3].tree_count = 2;
            }
            3 => parsed.witnesses.swap(2, 3),
            4 => parsed.witnesses[2].effect = u32::MAX,
            5 => parsed.witnesses[2].capture = u32::MAX,
            6 => {
                let mut values = std::mem::take(&mut parsed.trees).into_vec();
                values.pop();
                parsed.trees = values.into_boxed_slice();
            }
            7 => {
                let mut values = std::mem::take(&mut parsed.trees).into_vec();
                values.push(values[0]);
                parsed.trees = values.into_boxed_slice();
            }
            _ => parsed.trees.swap(0, 1),
        }
        let before = producer.work().steps;
        assert!(
            producer.validate(&report.package).is_err(),
            "mutation {mutation}"
        );
        assert!(
            producer.policy(&report.package, Some(tail), key).is_err(),
            "warm mutation {mutation}"
        );
        assert!(producer.work().steps > before);
    }
    Ok(())
}
#[test]
fn malformed_source_ranges_and_missing_witness_rows_are_diagnosed() -> TestResult {
    for mutation in 0..4 {
        let (mut report, mut producer) = owned(&format!("{ATTRIBUTES}fn f(){{}}"))?;
        let owner = &mut producer.owners[0];
        let effect = owner.parsed.witnesses[2].effect as usize;
        if mutation < 2 {
            let range = if mutation == 0 {
                TreeRange {
                    start: u32::MAX,
                    end: u32::MAX,
                }
            } else {
                TreeRange {
                    start: 0,
                    end: u32::MAX,
                }
            };
            report.package.units[owner.key.file.0 as usize]
                .configured
                .effects[effect]
                .attribute
                .attribute
                .path = range;
            owner.parsed.witnesses[2].attribute.path = range;
        } else {
            let mut rows = std::mem::take(&mut owner.parsed.witnesses).into_vec();
            if mutation == 2 {
                rows.remove(0);
            } else {
                rows.pop();
            }
            owner.parsed.witnesses = rows.into_boxed_slice();
        }
        assert!(
            producer.validate(&report.package).is_err(),
            "mutation {mutation}"
        );
    }
    Ok(())
}
#[test]
fn failed_exact_payload_payment_retains_prior_owner_and_sticky_work() -> TestResult {
    let (report, observed) = owned("#[doc=\"first\"]fn f(){} #[inline]fn g(){}")?;
    let first = observed.owners[0].key;
    let second = observed.owners[1].key;
    for storage in [true, false] {
        let mut producer = Producer::new(0, 0);
        producer
            .owner(&report.package, first, None)
            .map_err(Error::from)?;
        let first_bytes = size_of_val(producer.owners[0].parsed.witnesses.as_ref());
        if storage {
            let overhead =
                4 * size_of::<Owner>() + crate::support::retained::new_map::<NodeKey, Tail>(1);
            let remaining = (64 * 1024 * 1024_usize)
                .checked_sub(producer.work().bytes + overhead)
                .ok_or("payment budget")?;
            producer
                .reserve_diagnostic(second, remaining)
                .map_err(Error::from)?;
        } else {
            producer.work.steps = 10_000_000 - 2;
        }
        let before = producer.work().steps;
        let error = producer
            .owner(&report.package, second, None)
            .err()
            .ok_or("unpaid owner published")?;
        assert_eq!(
            error.message,
            if storage {
                "source lint policy storage budget exceeded"
            } else {
                "source lint policy work budget exceeded"
            }
        );
        assert_eq!(producer.owners.len(), 1);
        assert_eq!(producer.index.len(), 1);
        assert_eq!(
            size_of_val(producer.owners[0].parsed.witnesses.as_ref()),
            first_bytes
        );
        assert!(producer.work().steps > before);
        let failed = producer.work().steps;
        assert!(producer.owner(&report.package, second, None).is_err());
        assert!(producer.work().steps > failed);
        assert_eq!(producer.work().owners, 1);
    }
    Ok(())
}
#[test]
fn compact_changes_preserve_forbid_order_and_source_offset() -> TestResult {
    let source = "#[forbid(unused_must_use)]#[doc=\"keep\"]#[allow(unused_must_use)]fn f(){}";
    let (report, mut producer) = owned(source)?;
    let key = producer.owners[0].key;
    let tail = producer.index[&key];
    let changes = &producer.owners[0].parsed.changes;
    assert_eq!(changes.len(), 2);
    let expected = u32::try_from(source.find("#[allow").ok_or("allow source offset")?)?;
    assert_eq!(changes[1].offset, expected);
    for _ in 0..2 {
        let error = producer
            .policy(&report.package, Some(tail), key)
            .err()
            .ok_or("forbid was weakened")?;
        assert_eq!(error.offset, expected);
        assert_eq!(
            error.message,
            "forbidden unused_must_use lint cannot be weakened"
        );
    }
    Ok(())
}
