//! Reached cleanup demands are tied to the actual retained caller and event.
use super::{Bundle, Method};
use crate::{
    body::{
        ir::{Place, Terminator},
        storage::Location,
    },
    const_eval::{
        Frame, Machine, State,
        code::{Catalog, Code},
    },
    sema::model::Declarations,
    types::{TypeId, instance::Instance},
};
pub(crate) struct Reached {
    frame: usize,
    code: usize,
    event: u32,
    method: Instance,
    target: TypeId,
}
pub(super) fn cursor(frame: &Frame) -> Result<usize, &'static str> {
    frame
        .cleanup_cursor
        .map(|n| (n.get() - 1) as usize)
        .ok_or("cleanup continuation has no actual event cursor")
}
pub(super) fn location(code: &Code, frame: &Frame, event: Location) -> Result<(), &'static str> {
    if frame.operand_cursor != u32::MAX {
        return Err("cleanup was reached before operand readiness");
    }
    match event {
        Location::At { block, before } if block == frame.block && before == frame.next => Ok(()),
        Location::Edge { block, target } if block == frame.block => {
            let source = code
                .blocks
                .get(block.0 as usize)
                .ok_or("invalid cleanup edge block")?;
            if frame.next != source.end {
                return Err("cleanup edge precedes its real terminator");
            }
            let actual = match source.terminator {
                Terminator::Jump(target) | Terminator::FalseEdge { real: target, .. } => target,
                Terminator::Branch { condition, yes, no } => {
                    if crate::const_eval::values::boolean(
                        Machine::value(frame, condition)
                            .map_err(|_| "cleanup branch value is unavailable")?,
                    )
                    .map_err(|_| "invalid cleanup branch condition")?
                    {
                        yes
                    } else {
                        no
                    }
                }
                _ => return Err("cleanup edge is not a real selected successor"),
            };
            if target != actual {
                return Err("cleanup belongs to another selected edge");
            }
            Ok(())
        }
        _ => Err("cleanup event belongs to another current CFG point"),
    }
}
pub(super) fn current(
    facts: &Declarations,
    code: &Code,
    frame: &Frame,
    bundle: &Bundle,
    work: &mut crate::body::defining_cleanups::Work,
) -> Result<(usize, Method, Place), &'static str> {
    let event = cursor(frame)?;
    let recorded = bundle
        .view
        .events
        .get(event)
        .ok_or("cleanup cursor escaped its actual plan")?;
    location(code, frame, recorded.location)?;
    let (method, place) = bundle.reached(facts, code, event, work)?;
    Ok((event, method, place))
}
impl State {
    pub(crate) fn reached_cleanup(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        method: Instance,
        work: &mut crate::body::defining_cleanups::Work,
    ) -> Result<Reached, &'static str> {
        work.charge(8)?;
        let frame = self
            .frames
            .last()
            .ok_or("cleanup preparation has no actual caller frame")?;
        catalog.validate_entry(frame.body)?;
        let code = catalog
            .code(frame.body)
            .ok_or("cleanup caller code is missing")?;
        let bundle = code
            .cleanup()
            .ok_or("cleanup caller has no owned execution footer")?;
        let (event, actual, _) = current(facts, code, frame, bundle, work)?;
        if actual.instance != method {
            return Err("cleanup preparation changed its actual selected method");
        }
        Ok(Reached {
            frame: frame.instance,
            code: frame.body,
            event: u32::try_from(event).map_err(|_| "cleanup event identity overflow")?,
            method,
            target: actual.target,
        })
    }
    pub(crate) fn require_cleanup(
        &self,
        facts: &Declarations,
        catalog: &Catalog,
        reached: &Reached,
        work: &mut crate::body::defining_cleanups::Work,
    ) -> Result<(), &'static str> {
        let actual = self.reached_cleanup(facts, catalog, reached.method, work)?;
        if (actual.frame, actual.code, actual.event, actual.target)
            != (reached.frame, reached.code, reached.event, reached.target)
        {
            return Err("cleanup body preparation changed its exact caller continuation");
        }
        Ok(())
    }
}
