//! A private source port may share the image of a directly equal visible port.
use crate::{
    support::hash::{HashMap, HashSet},
    types::{RegionId, outlives::Pair},
};

pub(crate) fn visit(
    pairs: &[Pair],
    slot: impl Fn(u16) -> Option<RegionId>,
    missing: &'static str,
    mut publish: impl FnMut(Pair),
) -> Result<usize, &'static str> {
    let mut aliases = None;
    let mut image = |region| {
        if let Some(actual) = visible(&slot, region)? {
            return Ok(actual);
        }
        if aliases.is_none() {
            aliases = Some(equal_visible_ports(&slot, pairs)?);
        }
        aliases
            .as_ref()
            .and_then(|aliases| aliases.get(&region))
            .copied()
            .ok_or(missing)
    };
    for &(longer, shorter) in pairs {
        publish((image(longer)?, image(shorter)?));
    }
    Ok(aliases.map_or(0, |_| pairs.len().saturating_mul(3)))
}

fn visible(
    slot: &impl Fn(u16) -> Option<RegionId>,
    region: RegionId,
) -> Result<Option<RegionId>, &'static str> {
    if region == RegionId::STATIC {
        return Ok(Some(region));
    }
    let Some((0, source)) = region.bound_parts() else {
        return Err("declared callable condition escapes its source binder");
    };
    Ok(slot(source))
}

fn equal_visible_ports(
    slot: &impl Fn(u16) -> Option<RegionId>,
    pairs: &[Pair],
) -> Result<HashMap<RegionId, RegionId>, &'static str> {
    let relations: HashSet<_> = pairs.iter().copied().collect();
    let mut aliases = HashMap::default();
    for &(private, anchor) in pairs {
        if let Some(actual) = visible(slot, anchor)?
            && visible(slot, private)?.is_none()
            && relations.contains(&(anchor, private))
        {
            // Every original pair is still required. If this port has two
            // visible anchors, the other anchor must prove both directions.
            aliases.entry(private).or_insert(actual);
        }
    }
    Ok(aliases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn private_slot_images_require_reciprocal_proved_conditions() -> Result<(), &'static str> {
        let a = RegionId::bound(0, 0)?;
        let b = RegionId::bound(0, 1)?;
        let private = RegionId::bound(0, 2)?;
        let slot = |index| [a, b].get(usize::from(index)).copied();
        let mut proof = Vec::new();
        let missing = "missing port";
        visit(
            &[(a, private), (private, a), (b, private)],
            slot,
            missing,
            |pair| {
                proof.push(pair);
            },
        )?;
        assert_eq!(proof, [(a, a), (a, a), (b, a)]);
        assert_eq!(
            visit(&[(a, private), (b, private)], slot, missing, |_| {}),
            Err(missing)
        );
        Ok(())
    }
}
