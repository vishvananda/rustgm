//! Symbolic byte rows name allocation/function/vtable targets without pointer bits.
use super::Graph;
use crate::{
    sema::values::{Constant, View, WordTarget, WritePermission},
    types::TypeId,
};
use std::fmt::Write;
impl Graph<'_> {
    pub(super) fn image_row(
        &mut self,
        row: &mut String,
        ty: TypeId,
        value: Constant,
        size: u64,
    ) -> Result<(), &'static str> {
        let image = self.reader.image(value)?;
        if image.ty != ty || image.bytes.len() as u64 != size {
            return Err("constant view image differs from its stored type");
        }
        self.charge(
            image
                .bytes
                .len()
                .saturating_add(image.initialized.len())
                .saturating_add(image.fragments.len()),
        )?;
        self.storage(
            image
                .bytes
                .len()
                .saturating_mul(2)
                .saturating_add(image.initialized.len().saturating_mul(16))
                .saturating_add(image.fragments.len().saturating_mul(128))
                .saturating_add(64),
        )?;
        write!(row, "image size={size} bytes=").map_err(|_| "constant view formatting failed")?;
        let mut cursor = 0;
        for (index, byte) in image.bytes.iter().enumerate() {
            while image.fragments.get(cursor).is_some_and(|fragment| {
                fragment.offset + u64::from(fragment.length) <= index as u64
            }) {
                cursor += 1;
            }
            if image
                .fragments
                .get(cursor)
                .is_some_and(|fragment| fragment.offset <= index as u64)
            {
                row.push_str("@@");
            } else if image.initialized(index) {
                write!(row, "{byte:02x}").map_err(|_| "constant view formatting failed")?;
            } else {
                row.push_str("??");
            }
        }
        row.push_str(" initialized=");
        for word in image.initialized {
            write!(row, "{word:016x}").map_err(|_| "constant view formatting failed")?;
        }
        if !image.fragments.is_empty() {
            row.push_str(" fragments=[");
            for (index, fragment) in image.fragments.iter().enumerate() {
                if index != 0 {
                    row.push(',');
                }
                write!(
                    row,
                    "{}:{}:{}:",
                    fragment.offset, fragment.source_offset, fragment.length
                )
                .map_err(|_| "constant view formatting failed")?;
                self.image_target(row, fragment.target)?;
            }
            row.push(']');
        }
        Ok(())
    }
    fn image_target(&mut self, row: &mut String, target: WordTarget) -> Result<(), &'static str> {
        match target {
            WordTarget::Value(value) => match self.reader.view(value)? {
                View::Reference(location) => {
                    let allocation = self.location(location)?;
                    let permission = match value
                        .write_permission()
                        .ok_or("symbolic view word lacks pointer authority")?
                    {
                        WritePermission::Writable => "writable",
                        WritePermission::SharedDerived => "shared-derived",
                    };
                    write!(row, "pointer:a{allocation}:write={permission}")
                        .map_err(|_| "constant view formatting failed")
                }
                View::Function(instance) => write!(row, "function:{}", instance.ty().0)
                    .map_err(|_| "constant view formatting failed"),
                _ => Err("constant view image contains a non-symbolic target"),
            },
            WordTarget::Vtable { concrete, target } => {
                write!(row, "vtable:{}:{}", concrete.0, target.0)
                    .map_err(|_| "constant view formatting failed")
            }
        }
    }
}
