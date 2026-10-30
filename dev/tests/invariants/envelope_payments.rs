//! Combined original/header envelopes reject real interning before mutation.
use super::*;
use crate::{
    body::transported_originals::decoded_bodies::Allowance,
    types::{Primitive as ScalarPrimitive, Store},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn pointer() -> Kind {
    Kind::Pointer {
        mutable: false,
        pointee: TypeId::primitive(ScalarPrimitive::U8),
    }
}
#[test]
fn combined_proof_envelopes_reconcile_failure_and_warm_canonical_growth() -> Result {
    assert_eq!(size_of::<Primitive>(), 16);
    assert_eq!(allocation(128, false), 128 * 4 * 16);
    assert_eq!(
        allocation(128, true),
        128 * 4 * size_of::<Option<Applied>>()
    );
    for axis in 0..3 {
        let mut store = Store::default();
        let before = store.len();
        let mut work = ImportedOriginals::default();
        let mut allowance = Allowance {
            header_work: 0,
            header_bytes: 0,
            templates: crates::MAX_BYTES,
        };
        match axis {
            0 => allowance.header_work = crates::MAX_COMPONENTS,
            1 => allowance.header_bytes = crates::MAX_BYTES.saturating_sub(work.retained_bytes()),
            _ => allowance.templates = work.retained_bytes(),
        }
        for _ in 0..2 {
            let previous = (work.steps, work.payload);
            assert!(
                paying(&mut work, &allowance, |pay| {
                    store
                        .intern_observed::<true>(pointer(), &|s, b| pay(s, b))
                        .map_err(str::to_owned)
                })
                .is_err()
            );
            assert_eq!(
                store.len(),
                before,
                "no mutation before exact envelope admission"
            );
            assert!(work.steps > previous.0);
            assert!(work.payload >= previous.1);
        }
    }
    let mut store = Store::default();
    let mut work = ImportedOriginals::default();
    let allowance = Allowance {
        header_work: 0,
        header_bytes: 0,
        templates: crates::MAX_BYTES,
    };
    let result: std::result::Result<(), String> = paying(&mut work, &allowance, |pay| {
        store.intern_observed::<true>(pointer(), &|s, b| pay(s, b))?;
        Err("after genuine canonical allocation".into())
    });
    assert_eq!(result, Err("after genuine canonical allocation".into()));
    assert!(work.steps > 0 && work.payload > 0);
    let retained = work.payload;
    for _ in 0..8 {
        paying(&mut work, &allowance, |pay| {
            store
                .intern_observed::<true>(pointer(), &|s, b| pay(s, b))
                .map_err(str::to_owned)
        })?;
        assert_eq!(
            work.payload, retained,
            "warm canonical interning allocates zero bytes"
        );
    }
    eprintln!(
        "selected Pending={} legacy_row={} applied_slot={} Member={} Index={}",
        size_of::<Pending>(),
        size_of::<Primitive>(),
        size_of::<Option<Applied>>(),
        size_of::<Member>(),
        size_of::<Index<'_>>()
    );
    Ok(())
}
