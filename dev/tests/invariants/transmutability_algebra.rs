//! Independent finite-word models, opaque reference obligations and compressed scaling.
use super::*;
use crate::types::{RegionId, TypeId};
use std::collections::HashSet;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn word(language: &mut Language, values: &[u16], budget: &mut Budget<'_>) -> Result<LayoutId> {
    let mut result = language.epsilon(budget)?;
    for &value in values {
        let byte = language.byte(ByteRange::new(value, value + 1)?, budget)?;
        result = language.sequence(result, byte, budget)?;
    }
    Ok(result)
}
fn selected(
    language: &mut Language,
    mask: u8,
    budget: &mut Budget<'_>,
) -> Result<(LayoutId, Vec<Vec<u16>>)> {
    let mut nodes = Vec::new();
    let mut words = Vec::new();
    for i in 0..4 {
        if mask & (1 << i) != 0 {
            let value = vec![i / 2, i % 2];
            nodes.push(word(language, &value, budget)?);
            words.push(value);
        }
    }
    let root = if nodes.is_empty() {
        language.empty(2, budget)?
    } else {
        language.alternatives(nodes, budget)?
    };
    Ok((root, words))
}
fn boolean(proof: &Proof) -> Result<bool> {
    match proof.condition(proof.root) {
        Some(Condition::Yes) => Ok(true),
        Some(Condition::No) => Ok(false),
        _ => Err("unexpected reference obligation in byte language".into()),
    }
}
#[test]
fn correlated_languages_match_independent_exhaustive_word_inclusion_and_intersection() -> Result {
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let mut language = Language::default();
    let mut cases = Vec::new();
    for mask in 0..16 {
        cases.push(selected(&mut language, mask, &mut budget)?);
    }
    for (a, source) in &cases {
        for (b, destination) in &cases {
            for validity in [false, true] {
                let expected = if validity {
                    source.is_empty() || source.iter().any(|s| destination.contains(s))
                } else {
                    source.iter().all(|s| destination.contains(s))
                };
                let mut pay = |_, _| Ok(());
                let mut budget = Budget::new(Limits::default(), &mut pay);
                let proof = decide(
                    &mut language,
                    *a,
                    *b,
                    Assumptions {
                        validity,
                        ..Default::default()
                    },
                    &mut budget,
                )?;
                assert_eq!(
                    boolean(&proof)?,
                    expected,
                    "{source:?} -> {destination:?}, validity={validity}"
                );
            }
        }
    }
    Ok(())
}
#[test]
fn truncation_uninitialized_extension_and_zero_size_nominal_guards_remain_distinct() -> Result {
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let mut language = Language::default();
    let epsilon = language.epsilon(&mut budget)?;
    let bytes = language.byte(ByteRange::INITIALIZED, &mut budget)?;
    let padding = language.byte(ByteRange::UNINITIALIZED, &mut budget)?;
    let two = language.repeat(bytes, 2, &mut budget)?;
    assert!(boolean(&decide(
        &mut language,
        two,
        bytes,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(!boolean(&decide(
        &mut language,
        bytes,
        two,
        Assumptions::default(),
        &mut budget
    )?)?);
    let padded = language.sequence(bytes, padding, &mut budget)?;
    assert!(boolean(&decide(
        &mut language,
        bytes,
        padded,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(!boolean(&decide(
        &mut language,
        padded,
        two,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(boolean(&decide(
        &mut language,
        padded,
        two,
        Assumptions {
            validity: true,
            ..Default::default()
        },
        &mut budget
    )?)?);
    let guarded = language.nominal(epsilon, &mut budget)?;
    let guarded_sequence = language.sequence(guarded, bytes, &mut budget)?;
    assert!(!boolean(&decide(
        &mut language,
        bytes,
        guarded_sequence,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(boolean(&decide(
        &mut language,
        bytes,
        guarded_sequence,
        Assumptions {
            safety: true,
            ..Default::default()
        },
        &mut budget
    )?)?);
    let repeated_guard = language.repeat(guarded, 1_000_000_000, &mut budget)?;
    assert!(!boolean(&decide(
        &mut language,
        epsilon,
        repeated_guard,
        Assumptions::default(),
        &mut budget
    )?)?);
    let zero = language.repeat(guarded, 0, &mut budget)?;
    assert!(boolean(&decide(
        &mut language,
        epsilon,
        zero,
        Assumptions::default(),
        &mut budget
    )?)?);
    let empty = language.empty(8, &mut budget)?;
    assert!(boolean(&decide(
        &mut language,
        empty,
        guarded_sequence,
        Assumptions::default(),
        &mut budget
    )?)?);
    Ok(())
}
fn character(language: &mut Language, budget: &mut Budget<'_>) -> Result<LayoutId> {
    let mut branches = Vec::new();
    for ranges in [
        [(0, 256), (0, 216), (0, 1), (0, 1)],
        [(0, 256), (224, 256), (0, 1), (0, 1)],
        [(0, 256), (0, 256), (1, 17), (0, 1)],
    ] {
        let mut node = language.epsilon(budget)?;
        for (start, end) in ranges {
            let byte = language.byte(ByteRange::new(start, end)?, budget)?;
            node = language.sequence(node, byte, budget)?;
        }
        branches.push(node);
    }
    Ok(language.alternatives(branches, budget)?)
}
#[test]
fn billion_element_character_and_byte_arrays_factor_without_expanding_elements() -> Result {
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let mut language = Language::default();
    let element = character(&mut language, &mut budget)?;
    let byte = language.byte(ByteRange::INITIALIZED, &mut budget)?;
    let characters = language.repeat(element, 1_000_000_000, &mut budget)?;
    let bytes = language.repeat(byte, 4_000_000_000, &mut budget)?;
    assert!(boolean(&decide(
        &mut language,
        characters,
        bytes,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(!boolean(&decide(
        &mut language,
        bytes,
        characters,
        Assumptions::default(),
        &mut budget
    )?)?);
    assert!(boolean(&decide(
        &mut language,
        bytes,
        characters,
        Assumptions {
            validity: true,
            ..Default::default()
        },
        &mut budget
    )?)?);
    assert!(budget.consumed().0 < 30_000, "{:?}", budget.consumed());
    assert!(language.len() < 256, "{}", language.len());
    Ok(())
}
fn obligations(proof: &Proof) -> Result<HashSet<Obligation>> {
    let mut stack = vec![proof.root];
    let mut seen = HashSet::new();
    let mut result = HashSet::new();
    while let Some(id) = stack.pop() {
        if !seen.insert(id) {
            continue;
        }
        match proof.condition(id).ok_or("condition")? {
            Condition::Obligation(value) => {
                result.insert(value);
            }
            Condition::All(a, b) | Condition::Any(a, b) => {
                stack.push(a);
                stack.push(b);
            }
            Condition::Yes | Condition::No => {}
        }
    }
    Ok(result)
}
#[test]
fn references_preserve_recursive_type_lifetime_freeze_and_reverse_mutable_obligations() -> Result {
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(Limits::default(), &mut pay);
    let mut language = Language::default();
    let source = Reference {
        referent: TypeId(7),
        region: RegionId(9),
        mutable: true,
        size: 4,
        align: 4,
    };
    let target = Reference {
        referent: TypeId(8),
        region: RegionId(10),
        mutable: false,
        size: 2,
        align: 2,
    };
    let a = language.reference(source, &mut budget)?;
    let b = language.reference(target, &mut budget)?;
    let proof = decide(&mut language, a, b, Assumptions::default(), &mut budget)?;
    let expected = HashSet::from([
        Obligation::Transmutable {
            source: TypeId(7),
            destination: TypeId(8),
        },
        Obligation::Outlives {
            long: RegionId(9),
            short: RegionId(10),
        },
        Obligation::Freeze(TypeId(8)),
    ]);
    assert_eq!(obligations(&proof)?, expected);
    let mutable = language.reference(
        Reference {
            mutable: true,
            ..target
        },
        &mut budget,
    )?;
    let proof = decide(
        &mut language,
        a,
        mutable,
        Assumptions::default(),
        &mut budget,
    )?;
    let found = obligations(&proof)?;
    assert!(found.contains(&Obligation::Transmutable {
        source: TypeId(8),
        destination: TypeId(7)
    }));
    assert!(found.contains(&Obligation::Outlives {
        long: RegionId(10),
        short: RegionId(9)
    }));
    assert!(!found.contains(&Obligation::Freeze(TypeId(8))));
    let shared = language.reference(
        Reference {
            mutable: false,
            ..source
        },
        &mut budget,
    )?;
    let all = Assumptions {
        alignment: true,
        lifetimes: true,
        safety: true,
        validity: true,
    };
    assert!(!boolean(&decide(
        &mut language,
        shared,
        mutable,
        all,
        &mut budget
    )?)?);
    let proof = decide(&mut language, a, b, all, &mut budget)?;
    assert!(obligations(&proof)?.contains(&Obligation::Freeze(TypeId(8))));
    let many_a = language.repeat(a, 1_000_000_000, &mut budget)?;
    let many_b = language.repeat(b, 1_000_000_000, &mut budget)?;
    let repeated = decide(
        &mut language,
        many_a,
        many_b,
        Assumptions::default(),
        &mut budget,
    )?;
    assert_eq!(obligations(&repeated)?, expected);
    assert!(repeated.len() < 32);
    assert!(budget.consumed().0 < 10_000);
    Ok(())
}
#[test]
fn failed_growth_and_deep_shared_layouts_preserve_bounded_work_and_safe_stacks() -> Result {
    let mut language = Language::default();
    let mut reject = |_, bytes| {
        if bytes == 0 {
            Ok(())
        } else {
            Err("source owner rejected growth")
        }
    };
    let mut denied = Budget::new(Limits::default(), &mut reject);
    assert!(language.byte(ByteRange::BOOL, &mut denied).is_err());
    assert!(language.is_empty());
    let mut pay = |_, _| Ok(());
    let mut budget = Budget::new(
        Limits {
            work: 2_000_000,
            bytes: 64 * 1024 * 1024,
        },
        &mut pay,
    );
    let a = language.byte(ByteRange::BOOL, &mut budget)?;
    let b = language.byte(ByteRange::INITIALIZED, &mut budget)?;
    let mut root = language.nominal(a, &mut budget)?;
    for i in 0..20_000 {
        root = language.sequence(root, if i % 2 == 0 { b } else { a }, &mut budget)?;
    }
    assert!(boolean(&decide(
        &mut language,
        root,
        root,
        Assumptions {
            safety: true,
            ..Default::default()
        },
        &mut budget
    )?)?);
    assert!(budget.consumed().0 < 1_000_000, "{:?}", budget.consumed());
    let mut pay = |_, _| Ok(());
    let mut exhausted = Budget::new(Limits { work: 0, bytes: 0 }, &mut pay);
    assert!(
        decide(
            &mut language,
            root,
            root,
            Assumptions::default(),
            &mut exhausted
        )
        .is_err()
    );
    Ok(())
}
