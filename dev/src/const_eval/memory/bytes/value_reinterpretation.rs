//! One checked value move shares byte codecs and the existing paid reference cursor.
use super::continuations::{Key, ReadInput, Source};
use super::{Constant, Failure, Id, Machine, Read, TypeId};
use crate::const_eval::static_reads::ReadPurpose;
impl Machine<'_, '_> {
    pub(in crate::const_eval) fn transmute_value(
        &mut self,
        source: TypeId,
        destination: TypeId,
        value: Constant,
    ) -> Result<Read<Constant>, Failure> {
        // Charge the demanded operation before even a failed layout/cache lookup.
        self.charge(1)?;
        let key = Key {
            input: ReadInput::Transmute { source, value },
            offset: 0,
            ty: destination,
            length: None,
            purpose: ReadPurpose::Value,
            context: self.context,
        };
        if let Some(answer) = self.resumed_byte_read(key)? {
            return Ok(answer);
        }
        let a = self.footprint(source)?;
        let b = self.footprint(destination)?;
        if a.size != b.size {
            return Err("source transmute sizes changed before execution".into());
        }
        if a.size == 0 {
            return Err("zero-sized transmute values require a supported validity profile".into());
        }
        if !self.byte_family(source)? || !self.byte_family(destination)? {
            return Err(
                "source transmute requires supported source and destination byte families".into(),
            );
        }
        let mut backing = self.empty_backing(source)?;
        let previous = self.slots;
        self.slots = previous
            .checked_add(backing.retained_slots())
            .ok_or("transmute scratch overflow")?;
        let encoded = self.encode_bytes(&mut backing, 0, source, value);
        if let Err(error) = encoded {
            self.slots = previous;
            return Err(error);
        }
        let decoded = self.decode_transmute_backing(backing, source, destination);
        self.slots = previous;
        let (value, duties) = decoded?;
        self.complete_byte_read(key, value, duties)
    }
    fn decode_transmute_backing(
        &mut self,
        backing: super::Backing,
        source: TypeId,
        destination: TypeId,
    ) -> Result<(Constant, Vec<super::continuations::Duty>), Failure> {
        self.reserve_transmute_root()?;
        let count = self.memory.bytes.roots.len();
        let id = Id(u32::try_from(count).map_err(|_| "transmute backing identity exhausted")?);
        // No LocationId is registered for this scratch. Every reconstructed
        // pointer/duty comes from an existing canonical symbolic word target.
        self.memory.bytes.roots.push(backing);
        let decoded = (|| {
            let (_, mut source_duties) = self.decode_strict_value(Source::Backing(id), source)?;
            let (value, destination_duties) =
                self.decode_strict_value(Source::Backing(id), destination)?;
            let slots = source_duties
                .capacity()
                .saturating_add(destination_duties.capacity())
                .saturating_add(destination_duties.len())
                .saturating_mul(size_of::<super::continuations::Duty>())
                .div_ceil(size_of::<Constant>());
            self.reserve_cells(slots)?;
            source_duties
                .try_reserve(destination_duties.len())
                .map_err(|_| "transmute reference duty allocation failed")?;
            source_duties.extend(destination_duties);
            Ok((value, source_duties))
        })();
        self.memory
            .bytes
            .roots
            .pop()
            .ok_or("transmute scratch disappeared")?;
        if self.memory.bytes.roots.len() != count {
            return Err("transmute scratch nesting changed".into());
        }
        // Decode has retained values and duties only. The suspended caller frame
        // and Evaluator arenas own their original roots; scratch never owned them.
        decoded
    }
    fn reserve_transmute_root(&mut self) -> Result<(), Failure> {
        let roots = &self.memory.bytes.roots;
        if roots.len() < roots.capacity() {
            return Ok(());
        }
        let count = roots.len();
        let old = roots.capacity();
        let anticipated = roots.len().saturating_add(1).saturating_mul(4);
        self.charge(count.saturating_add(1))?;
        self.reserve_cells(
            anticipated
                .saturating_add(old)
                .saturating_mul(size_of::<super::Backing>())
                .div_ceil(size_of::<Constant>()),
        )?;
        self.memory
            .bytes
            .roots
            .try_reserve(1)
            .map_err(|_| "transmute backing index allocation failed")?;
        let actual = self.memory.bytes.roots.capacity();
        // Preserve the actual amortized Vec capacity payment after scratch pop.
        let slots = actual
            .saturating_sub(old)
            .saturating_mul(size_of::<super::Backing>())
            .div_ceil(size_of::<Constant>());
        self.memory.bytes.retained = self
            .memory
            .bytes
            .retained
            .checked_add(slots)
            .ok_or("transmute backing index retention overflow")?;
        self.reserve_cells(0)
    }
}
