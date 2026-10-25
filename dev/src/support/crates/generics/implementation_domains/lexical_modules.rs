//! One bounded forest walk derives real defining modules and ancestry intervals.
use super::{DeclarationRef, HeaderIndex, SourceRows, Work, s};
pub(super) type Frame = (usize, usize, u32, bool);
pub(super) fn scratch_bytes(count: usize) -> usize {
    count
        .saturating_add(1)
        .saturating_mul(2 * size_of::<usize>())
        .saturating_add(count.saturating_sub(1).saturating_mul(size_of::<usize>()))
        .saturating_add(count.saturating_mul(size_of::<Frame>()))
        .saturating_add(4 * size_of::<Vec<usize>>())
}
#[derive(Debug, PartialEq, Eq)]
pub(super) struct ModuleIndex {
    modules: Vec<DeclarationRef>,
    starts: Vec<u32>,
    ends: Vec<u32>,
}
impl ModuleIndex {
    pub(super) fn new(
        rows: &SourceRows,
        headers: &HeaderIndex<'_>,
        work: &mut Work,
    ) -> Result<Self, &'static str> {
        let count = rows.declarations.len();
        work.charge(1)?;
        // Offset and cursor initialization, child storage, all three retained
        // arrays, and the prefix pass are prepaid before their first allocation.
        work.charge(count.saturating_mul(7).saturating_add(1))?;
        let mut offsets = vec![0_usize; count + 1];
        let mut root = None;
        for (index, row) in rows.declarations.iter().enumerate() {
            work.charge(1)?;
            if let Some(parent) = row.definition.parent {
                offsets[parent.0 as usize + 1] += 1;
            } else if root.replace(index).is_some() {
                return Err("module index has multiple roots");
            }
        }
        for index in 1..offsets.len() {
            offsets[index] += offsets[index - 1];
        }
        let mut cursors = offsets.clone();
        let mut children = vec![0_usize; count.saturating_sub(1)];
        for (index, row) in rows.declarations.iter().enumerate() {
            work.charge(1)?;
            if let Some(parent) = row.definition.parent {
                let cursor = &mut cursors[parent.0 as usize];
                children[*cursor] = index;
                *cursor += 1;
            }
        }
        let root = root.ok_or("module index has no actual root")?;
        let mut result = Self {
            modules: vec![DeclarationRef(0); count],
            starts: vec![0; count],
            ends: vec![0; count],
        };
        let mut stack = Vec::<Frame>::with_capacity(count);
        stack.push((root, root, 0, false));
        let mut time = 0_u32;
        while let Some((index, parent_module, depth, leaving)) = stack.pop() {
            work.charge(1)?;
            if leaving {
                result.ends[index] = time;
                continue;
            }
            if rows.declarations[index].definition.kind == s::DefinitionKind::Function {
                let row = DeclarationRef(
                    u32::try_from(index).map_err(|_| "function declaration overflow")?,
                );
                if depth > 64
                    || !matches!(headers.header(row, work)?, Some(s::HeaderRef::Function(_)))
                {
                    return Err(
                        "source canonical function path exceeds profile depth or lacks its actual header",
                    );
                }
            }
            let module = if rows.declarations[index].definition.kind == s::DefinitionKind::Module {
                index
            } else {
                parent_module
            };
            result.modules[index] =
                DeclarationRef(u32::try_from(module).map_err(|_| "module index overflow")?);
            result.starts[index] = time;
            time = time.checked_add(1).ok_or("module interval overflow")?;
            stack.push((index, module, depth, true));
            for &child in children[offsets[index]..offsets[index + 1]].iter().rev() {
                work.charge(1)?;
                stack.push((
                    child,
                    module,
                    depth
                        .checked_add(1)
                        .ok_or("source lexical depth overflow")?,
                    false,
                ));
            }
        }
        if time as usize != count {
            return Err("module index omitted an actual declaration");
        }
        Ok(result)
    }
    pub(super) fn module(&self, row: DeclarationRef) -> Result<DeclarationRef, &'static str> {
        self.modules
            .get(row.0 as usize)
            .copied()
            .ok_or("source defining-module coordinate is unknown")
    }
    pub(super) fn within(
        &self,
        current: DeclarationRef,
        ancestor: DeclarationRef,
    ) -> Result<bool, &'static str> {
        let start = *self
            .starts
            .get(current.0 as usize)
            .ok_or("source current module is unknown")?;
        let ancestor_start = *self
            .starts
            .get(ancestor.0 as usize)
            .ok_or("source ancestor module is unknown")?;
        let end = *self
            .ends
            .get(ancestor.0 as usize)
            .ok_or("source ancestor module is unknown")?;
        Ok(ancestor_start <= start && start < end)
    }
}
