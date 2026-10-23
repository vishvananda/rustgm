//! Optional retained proof records; executable verification does not allocate traces.
use super::{Index, Loan, Point, Set, liveness, points};

pub struct Inspection {
    pub(super) source: crate::body::ir::BodyOrigin,
    pub(super) detail: Option<Detail>,
}
pub(super) struct Detail {
    pub places: Index,
    pub loans: Vec<Loan>,
    pub uses: liveness::Analysis,
    pub regions: points::Liveness,
    pub snapshots: Vec<Snapshot>,
}
pub(super) struct Snapshot {
    pub point: Point,
    pub word: u32,
    pub live: u64,
    pub active: u64,
}
pub(super) fn record(
    snapshots: &mut Vec<Snapshot>,
    point: Point,
    state: &Set,
) -> Result<(), &'static str> {
    for (word, state) in state.iter() {
        let word = u32::try_from(word).map_err(|_| "too many loan inspection words")?;
        if state.live != 0 {
            if snapshots.len() >= 2_000_000 {
                return Err("loan inspection exceeds 48 MB snapshot budget");
            }
            snapshots.push(Snapshot {
                point,
                word,
                live: state.live,
                active: state.active,
            });
        }
    }
    Ok(())
}
