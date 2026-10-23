//! Ordering is bounded data processing, never declaration/proof authority.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(width: u32) -> Result<(Vec<GenericRequest>, HashMap<DefId, usize>)> {
    let owner = crate::support::crates::CrateIdentity::new("provider", "ordering")?;
    let mut values = Vec::new();
    let mut keys = HashMap::default();
    for index in (0..width).rev() {
        let external = ExternalDef::new(
            owner.clone(),
            vec![format!("helper{index:08}")],
            ExternalKind::Function,
        )?;
        let cost = external.path().iter().map(String::len).sum::<usize>()
            + owner.name().len()
            + owner.disambiguator().len()
            + 1;
        keys.insert(DefId(index), cost);
        values.push(GenericRequest {
            definition: DefId(index),
            external,
        });
    }
    Ok((values, keys))
}
#[test]
fn dependency_ordering_charges_observed_keys_and_retains_failed_work() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let (mut values, keys) = fixture(width)?;
        let mut steps = 0;
        order(&mut values, &keys, &mut steps)?;
        assert!(values.windows(2).all(|p| p[0].external < p[1].external));
        let width = width as usize;
        let key = keys
            .values()
            .copied()
            .max()
            .ok_or("missing observed key sizes")?;
        // Reverse keys cover the small insertion case and larger monotone runs.
        // Every possible pair is charged at its complete observed key bound.
        let bound = width + width * (width - 1) / 2 * (2 * key + 1);
        assert!(steps > width && steps <= bound);
        if let Some(prior) = previous {
            assert!(steps > prior && steps < 5 * prior);
        }
        eprintln!("dependencies={width} sorting_work={steps}");
        previous = Some(steps);
        let (mut values, keys) = fixture(u32::try_from(width)?)?;
        let mut steps = crate::support::crates::MAX_COMPONENTS - 1;
        assert!(order(&mut values, &keys, &mut steps).is_err());
        assert!(steps > crate::support::crates::MAX_COMPONENTS);
    }
    let (mut values, mut keys) = fixture(1)?;
    keys.clear();
    let mut steps = 0;
    assert!(order(&mut values, &keys, &mut steps).is_err());
    assert_eq!(steps, 1);
    Ok(())
}
