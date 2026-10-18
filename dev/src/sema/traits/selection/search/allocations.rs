//! A paused search reserves its high watermark without rebasing the retained proof graph.
use crate::types::lifetime_proof::Obligations;

pub(super) struct Allocations {
    first: u32,
    floor: u32,
    end: u32,
    stream: Option<Box<Obligations>>,
}
impl Allocations {
    pub fn new(first: u32) -> Self {
        Self {
            first,
            floor: first,
            end: first,
            stream: None,
        }
    }
    pub fn floor(&self) -> u32 {
        self.floor
    }
    pub fn end(&self) -> u32 {
        self.end
    }
    pub fn record(&mut self, end: u32) {
        self.end = self.end.max(end);
    }
    /// The caller reserved every trial allocation before doing foreign work.
    /// Recording a gap touches only the last interval, independent of stack depth.
    pub fn resume_at(&mut self, first: u32) -> Result<(), &'static str> {
        if first < self.end || first == 0 || first >= 0x8000_0000 {
            return Err("resumed trait selection has an invalid lifetime frontier");
        }
        let stream = self
            .stream
            .get_or_insert_with(|| Box::new(Obligations::new(self.first)));
        stream.regions.end = self.end;
        stream.resume_at(first)?;
        self.floor = first;
        self.end = first;
        Ok(())
    }
    pub fn finish(self, selected_end: u32) -> Obligations {
        if let Some(mut stream) = self.stream {
            stream.regions.end = self.end;
            *stream
        } else {
            let mut proof = Obligations::new(self.first);
            proof.regions.end = selected_end;
            proof
        }
    }
}
