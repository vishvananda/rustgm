//! Registry mechanics are tested separately from source/handler execution permission.
use super::*;
use std::cell::Cell;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn request(registry: &Registry, count: u32) -> Bindings {
    let imports = (1..=count)
        .map(|id| {
            let definition = DefId(id);
            (
                definition,
                registry.function(definition).unwrap_or(ForeignFunction {
                    signature: TypeId::UNIT,
                    safety: Safety::Default,
                    naming: Naming::Rust,
                    linkage: Linkage::Import,
                }),
            )
        })
        .collect();
    Bindings {
        handler: Some((
            DefId(0),
            ForeignFunction {
                signature: TypeId::UNIT,
                safety: Safety::Default,
                naming: Naming::Rust,
                linkage: Linkage::Export,
            },
        )),
        imports,
    }
}
fn setup(count: u32) -> Result<Registry> {
    let mut registry = Registry::default();
    for (id, function) in request(&registry, count).imports {
        registry.insert_function(id, function)?;
    }
    Ok(registry)
}
#[test]
fn canonical_symbol_batch_preserves_prior_rows_and_rejects_each_failed_payment() -> Result {
    let calls = Cell::new(0usize);
    let mut registry = setup(8)?;
    registry.bind_handler(request(&registry, 8), |_| {
        calls.set(calls.get() + 1);
        Ok(())
    })?;
    for stopped in 0..calls.get() {
        let mut registry = setup(8)?;
        let prior: Vec<_> = (1..=8).map(|id| registry.function(DefId(id))).collect();
        let attempted = Cell::new(0usize);
        let batch = request(&registry, 8);
        assert!(
            registry
                .bind_handler(batch, |_| {
                    let count = attempted.get();
                    attempted.set(count + 1);
                    if count == stopped {
                        Err("stopped handler transaction")
                    } else {
                        Ok(())
                    }
                })
                .is_err()
        );
        assert!(registry.function(DefId(0)).is_none());
        assert!(registry.exports().is_empty());
        assert!(registry.names().is_empty());
        assert_eq!(
            prior,
            (1..=8)
                .map(|id| registry.function(DefId(id)))
                .collect::<Vec<_>>()
        );
        assert_eq!(attempted.get(), stopped + 1);
    }
    let name = registry
        .function(DefId(0))
        .ok_or("actual handler registry row")?
        .naming;
    let Naming::Exact(symbol) = name else {
        return Err("handler was not exact".into());
    };
    assert_eq!(registry.names().get(symbol), Some("rust_begin_unwind"));
    for id in 1..=8 {
        assert_eq!(registry.function(DefId(id)).ok_or("weak row")?.naming, name);
    }
    let warm = request(&registry, 8);
    registry.bind_handler(warm, |_| Ok(()))?;
    assert_eq!(registry.names().len(), 1);
    existing_export_collision_and_prior_callable_changes_deny_before_aliases()
}
fn existing_export_collision_and_prior_callable_changes_deny_before_aliases() -> Result {
    for wrong in [false, true] {
        let mut registry = setup(2)?;
        let mut batch = request(&registry, 2);
        if wrong {
            batch.imports[0].1.safety = Safety::Unsafe;
        } else {
            let name = registry.names_mut().intern("rust_begin_unwind")?;
            registry.insert_function(
                DefId(99),
                ForeignFunction {
                    signature: TypeId::UNIT,
                    safety: Safety::Default,
                    naming: Naming::Exact(name),
                    linkage: Linkage::Export,
                },
            )?;
        }
        let previous = registry.function(DefId(1));
        assert!(registry.bind_handler(batch, |_| Ok(())).is_err());
        assert!(registry.function(DefId(0)).is_none());
        assert_eq!(registry.function(DefId(1)), previous);
    }
    Ok(())
}
