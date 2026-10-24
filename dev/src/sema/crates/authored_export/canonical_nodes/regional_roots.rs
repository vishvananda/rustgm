//! Each free region belongs to one actual original allocation; binders stay exact.
use super::{BodyRef, Context, FrameRef, RegionId, Stage, retained, s};

impl Stage {
    pub(crate) fn owned_slot(
        &mut self,
        body: BodyRef,
        region: RegionId,
    ) -> Result<u32, &'static str> {
        self.charge(1)?;
        self.graph
            .variables
            .get(&(body, region))
            .copied()
            .ok_or("source input region has no exact owned allocation")
    }
    pub(crate) fn owned_region(
        &mut self,
        body: BodyRef,
        region: RegionId,
        slot: u32,
    ) -> Result<(), &'static str> {
        self.charge(1)?;
        if region == RegionId::STATIC || region.bound_parts().is_some() {
            return Err("source owned allocation is not an actual free region");
        }
        self.reserve(retained::new_map::<(BodyRef, RegionId), u32>(1))?;
        if self.graph.variables.insert((body, region), slot).is_some() {
            return Err("source original has a duplicate owned region allocation");
        }
        Ok(())
    }

    pub(crate) fn region(
        &mut self,
        region: RegionId,
        context: Context,
    ) -> Result<s::RegionRef, &'static str> {
        self.charge(1)?;
        let key = (region, (region != RegionId::STATIC).then_some(context));
        if let Some(&row) = self.graph.regions.get(&key) {
            return Ok(row);
        }
        let row = if region == RegionId::STATIC {
            s::Region::Static
        } else if let Some((depth, slot)) = region.bound_parts() {
            let frame = context
                .frame
                .ok_or("source bound region has no actual header frame")?;
            let target = self.ancestor(frame, depth)?;
            if u32::from(slot) >= self.graph.frames[target.0 as usize].count {
                return Err("source bound region exceeds its exact binder frame");
            }
            s::Region::Bound {
                binder: self.graph.frames[frame.0 as usize].binder,
                depth,
                slot: u32::from(slot),
            }
        } else {
            let body = context
                .body
                .ok_or("source free region has no actual original owner")?;
            let slot = self
                .graph
                .variables
                .get(&(body, region))
                .copied()
                .ok_or("source free region is not owned by this original input")?;
            s::Region::Variable { body, slot }
        };
        self.reserve(
            4 * size_of::<s::Region>()
                + retained::new_map::<(RegionId, Option<Context>), s::RegionRef>(1),
        )?;
        let id = s::RegionRef(
            u32::try_from(self.rows.regions.len()).map_err(|_| "source region table overflow")?,
        );
        self.rows.regions.push(row);
        self.graph.regions.insert(key, id);
        Ok(id)
    }

    fn ancestor(&mut self, frame: FrameRef, depth: u16) -> Result<FrameRef, &'static str> {
        self.charge(1)?;
        if let Some(&ancestor) = self.graph.ancestors.get(&(frame, depth)) {
            return Ok(ancestor);
        }
        let mut ancestor = frame;
        for _ in 0..depth {
            self.charge(1)?;
            self.graph.ancestor_steps = self.graph.ancestor_steps.saturating_add(1);
            ancestor = self
                .graph
                .frames
                .get(ancestor.0 as usize)
                .and_then(|frame| frame.parent)
                .ok_or("source region escapes its complete binder ancestry")?;
        }
        self.reserve(retained::new_map::<(FrameRef, u16), FrameRef>(1))?;
        self.graph.ancestors.insert((frame, depth), ancestor);
        Ok(ancestor)
    }
}
