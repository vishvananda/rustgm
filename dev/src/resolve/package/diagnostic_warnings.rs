//! Terminal error owners move warning payloads once; merges scan only new records.
use super::{DiagnosticFile, Error, Warning};
const RECORDS: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;
const WORK: usize = 10_000_000;
#[cfg(test)]
#[path = "../../../tests/invariants/error_warning_transfer.rs"]
mod tests;

impl Error {
    /// Retained semantic warnings precede this unchanged primary error in the driver.
    #[must_use]
    pub fn warnings(&self) -> &[Warning] {
        &self.retained_warnings
    }

    pub(crate) fn retain_warnings(mut self, mut warnings: Vec<Warning>) -> Self {
        if warnings.is_empty() {
            return self;
        }
        let records = self.retained_warnings.len().saturating_add(warnings.len());
        if self.warning_limit_note || records > RECORDS {
            self.warning_limit_note = true;
            return self;
        }
        let mut work = 0_usize;
        let mut bytes = 0_usize;
        for warning in &warnings {
            let Some((steps, payload)) = warning.diagnostic.warning_payload() else {
                self.warning_limit_note = true;
                return self;
            };
            work = work.saturating_add(steps);
            bytes = bytes
                .saturating_add(payload)
                .saturating_add(4 * size_of::<Warning>());
        }
        self.warning_work = self.warning_work.saturating_add(work);
        self.warning_bytes = self.warning_bytes.saturating_add(bytes);
        if self.warning_work > WORK || self.warning_bytes > BYTES {
            self.warning_limit_note = true;
            return self;
        }
        if self.retained_warnings.is_empty() {
            std::mem::swap(&mut self.retained_warnings, &mut warnings);
        } else {
            self.retained_warnings.append(&mut warnings);
        }
        self
    }

    /// A warning diagnostic has no nested sink. Its physical trace is already bounded.
    pub(crate) fn warning_payload(&self) -> Option<(usize, usize)> {
        if !self.retained_warnings.is_empty() || self.expansions.len() >= WORK {
            return None;
        }
        let work = 1 + self.expansions.len();
        let mut bytes = size_of::<super::ErrorData>().saturating_add(self.message.len());
        if let DiagnosticFile::Imported(label) = &self.file {
            bytes = bytes.saturating_add(label.len());
        }
        bytes = bytes.saturating_add(
            self.expansions
                .capacity()
                .saturating_mul(size_of::<super::ExpansionTrace>()),
        );
        for frame in &self.expansions {
            bytes = bytes.saturating_add(frame.macro_name.len());
        }
        Some((work, bytes))
    }
}
