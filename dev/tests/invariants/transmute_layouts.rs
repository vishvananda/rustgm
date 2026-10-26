//! Observe the actual existing cold continuation and its enclosing owners.
use super::*;
#[test]
fn actual_byte_continuations_have_empty_ordinary_owners_and_measured_layouts() {
    let storage = super::super::Storage::default();
    assert!(storage.reads.is_none());
    assert!(storage.words.is_none());
    assert_eq!(storage.roots.capacity(), 0);
    let reads = Reads::default();
    assert_eq!(reads.entries.capacity(), 0);
    assert_eq!(reads.retained, 0);
    assert_eq!(size_of::<Constant>(), 32);
    eprintln!(
        "transmute after Source={} ReadInput={} Key={} Pending={} Entry={} Reads={} Decoder={:?}",
        size_of::<Source>(),
        size_of::<ReadInput>(),
        size_of::<Key>(),
        size_of::<Pending>(),
        size_of::<(ReadOperation, Pending)>(),
        size_of::<Reads>(),
        super::super::codec::decoder_layout(),
    );
    eprintln!(
        "transmute after Constant={}/{} Storage={} Backing={} Snapshot={} Memory={} Machine={} State={}",
        size_of::<Constant>(),
        std::mem::align_of::<Constant>(),
        size_of::<super::super::Storage>(),
        size_of::<super::super::Backing>(),
        size_of::<super::super::Snapshot>(),
        size_of::<crate::const_eval::memory::Memory>(),
        size_of::<crate::const_eval::Machine<'_, '_>>(),
        size_of::<crate::const_eval::State>(),
    );
}
