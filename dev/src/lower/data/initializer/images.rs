//! Frozen union words use ordinary checked function/global/vtable relocations.
use super::{Context, Initializer, Target};
use crate::{
    sema::values::{Constant, View, WordTarget},
    types::TypeId,
};
impl Context<'_> {
    pub(super) fn initialize_image(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        ty: TypeId,
        value: Constant,
        size: u64,
    ) -> Result<(), &'static str> {
        self.constants.require_publishable_array(value)?;
        let image = self.constants.image(value)?;
        if image.ty != ty || size != self.layout(ty)?.size || image.bytes.len() as u64 != size {
            return Err("constant union image differs from emitted storage");
        }
        let words = image.complete_words()?;
        self.data
            .charge(image.bytes.len().saturating_add(words.len()))?;
        let mut cursor = 0usize;
        for fragment in words {
            let start = usize::try_from(fragment.offset)
                .map_err(|_| "constant image relocation exceeds host size")?;
            for index in cursor..start {
                let byte = if image.initialized(index) {
                    image.bytes[index]
                } else {
                    0
                };
                init.bits(offset + index as u64, 1, u128::from(byte))?;
            }
            self.initialize_word(init, offset + fragment.offset, fragment.target)?;
            cursor = start
                .checked_add(8)
                .ok_or("constant image relocation extent overflow")?;
        }
        for index in cursor..image.bytes.len() {
            let byte = if image.initialized(index) {
                image.bytes[index]
            } else {
                0
            };
            init.bits(offset + index as u64, 1, u128::from(byte))?;
        }
        Ok(())
    }
    fn initialize_word(
        &mut self,
        init: &mut Initializer,
        offset: u64,
        target: WordTarget,
    ) -> Result<(), &'static str> {
        match target {
            WordTarget::Value(value) => match self.constants.view(value)? {
                View::Reference(id) if value.metadata().is_none() => {
                    self.initialize_reference(init, offset, id, false, None)
                }
                View::Function(instance) => {
                    let symbol = self.demand(instance)?;
                    self.initialize_pointer(init, offset, Target::Function(symbol), 0)
                }
                _ => Err("constant image relocation lacks a canonical target"),
            },
            WordTarget::Vtable { concrete, target } => {
                let table = self.object_table(concrete, target)?;
                self.initialize_pointer(init, offset, Target::Global(table), 0)
            }
        }
    }
}
