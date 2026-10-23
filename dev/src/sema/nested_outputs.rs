//! Output ports are semantic identities, not lifetime proof or execution authority.
use crate::support::hash::HashMap;
use crate::{
    resolve::{body_origin::BodyOrigin, unit::NodeKey},
    types::RegionId,
};
use std::sync::Arc;

#[derive(Default)]
pub struct Registry {
    links: HashMap<NodeKey, Arc<Link>>,
    pub ports: usize,
    pub bridges: usize,
    pub retained_bytes: usize,
}
impl Registry {
    pub(crate) fn reserve_port(&mut self) -> Result<(), &'static str> {
        if self.ports >= 65_536 {
            return Err("nested output port budget exceeded");
        }
        // Charge a complete minimal map per pair, covering separate interfaces
        // and geometric capacity without depending on allocator observations.
        self.charge(
            crate::support::retained::new_map::<RegionId, RegionId>(1)
                + size_of::<HashMap<RegionId, RegionId>>()
                + size_of::<NodeKey>(),
        )?;
        self.ports += 1;
        Ok(())
    }
    pub(crate) fn insert(&mut self, source: NodeKey, link: Link) -> Result<(), &'static str> {
        if link.source.source() != Some(source)
            || link.parent.source().is_none()
            || link.parent.source() == Some(source)
        {
            return Err("nested output link has mismatched source ownership");
        }
        if link
            .bridges
            .iter()
            .any(|bridge| u32::from(bridge.slot) >= link.universals)
            || link
                .bridges
                .windows(2)
                .any(|pair| pair[0].slot >= pair[1].slot)
            || usize::try_from(link.revision).ok()
                != Some(link.bridges.iter().filter(|bridge| bridge.output).count())
        {
            return Err("nested output link has inconsistent interface slots");
        }
        let bridges = self
            .bridges
            .checked_add(
                link.bridges.len()
                    + link
                        .formation
                        .as_ref()
                        .map_or(0, |formation| formation.inputs.len()),
            )
            .filter(|&count| count <= 1_000_000)
            .ok_or("nested output bridge budget exceeded")?;
        if self.links.len() >= 65_536 || self.links.contains_key(&source) {
            return Err("nested output source is duplicate or exceeds budget");
        }
        let formation = link.formation.as_ref().map_or(0, |formation| {
            size_of::<Formation>()
                + formation
                    .inputs
                    .len()
                    .saturating_mul(size_of::<super::closures::FormationInput>())
        });
        self.charge(
            size_of::<Link>()
                + crate::support::retained::new_map::<NodeKey, Arc<Link>>(1)
                + link.bridges.len().saturating_mul(size_of::<Bridge>())
                + formation,
        )?;
        self.bridges = bridges;
        self.links.insert(source, Arc::new(link));
        Ok(())
    }
    fn charge(&mut self, additional: usize) -> Result<(), &'static str> {
        self.retained_bytes = self
            .retained_bytes
            .checked_add(additional)
            .filter(|&bytes| bytes <= 64 * 1024 * 1024)
            .ok_or("nested output interfaces exceed 64 MiB budget")?;
        Ok(())
    }
    #[must_use]
    pub fn get(&self, source: &NodeKey) -> Option<&Arc<Link>> {
        self.links.get(source)
    }
}

#[derive(Clone, Copy)]
pub struct Bridge {
    pub slot: u16,
    pub parent: RegionId,
    pub output: bool,
}
pub struct Link {
    pub source: BodyOrigin,
    pub parent: BodyOrigin,
    pub revision: u32,
    pub universals: u32,
    pub bridges: Box<[Bridge]>,
    pub formation: Option<Formation>,
}
pub struct Formation {
    pub identity: crate::types::closure::ClosureId,
    pub inputs: Box<[super::closures::FormationInput]>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::unit::FileId;
    use crate::syntax::ast::NodeId;
    fn source(node: u32) -> NodeKey {
        NodeKey {
            file: FileId(0),
            node: NodeId::new(node),
        }
    }
    fn link() -> Link {
        Link {
            source: source(1).into(),
            parent: source(2).into(),
            revision: 0,
            universals: 0,
            bridges: Box::new([]),
            formation: None,
        }
    }
    #[test]
    fn foreign_links_and_changed_slots_are_rejected_before_publication() {
        let mut registry = Registry::default();
        assert!(registry.insert(source(3), link()).is_err());
        let mut changed = link();
        changed.revision = 1;
        assert!(registry.insert(source(1), changed).is_err());
        let mut changed = link();
        changed.bridges = vec![Bridge {
            slot: 0,
            parent: RegionId::STATIC,
            output: false,
        }]
        .into_boxed_slice();
        assert!(registry.insert(source(1), changed).is_err());
        assert_eq!(registry.retained_bytes, 0);
    }
    #[test]
    fn output_port_limit_is_deterministic_and_has_no_partial_charge() -> Result<(), &'static str> {
        let mut registry = Registry::default();
        for _ in 0..65_536 {
            registry.reserve_port()?;
        }
        let bytes = registry.retained_bytes;
        assert!(registry.reserve_port().is_err());
        assert_eq!(registry.ports, 65_536);
        assert_eq!(registry.retained_bytes, bytes);
        Ok(())
    }
}
