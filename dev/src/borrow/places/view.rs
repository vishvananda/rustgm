//! Render retained canonical paths without repeating alias expansion or overlap queries.
use super::{Index, Place, Selection, Step};
use std::io::{self, Write};

impl Index {
    pub(in crate::borrow) fn count(&self) -> usize {
        self.paths.len()
    }
    pub(in crate::borrow) fn write(&self, out: &mut impl Write) -> io::Result<()> {
        for (id, path) in self.paths.iter().enumerate() {
            write!(out, "  place p{id} root=")?;
            match path.root {
                Place::Local(id) => write!(out, "l{}", id.0)?,
                Place::Temporary(id) => write!(out, "v{}", id.0)?,
                Place::Static(id) => write!(out, "static:{}", id.0)?,
                Place::Projection(id) => write!(out, "p{}", id.0)?,
            }
            if let Some(parent) = path.parent {
                write!(out, " parent=p{}", parent.0)?;
            }
            match path.step {
                Some(Step::Dereference { mutable }) => {
                    write!(out, " dereference mutable={mutable}")?;
                }
                Some(Step::Field(Selection::Tuple(index))) => write!(out, " tuple-field={index}")?,
                Some(Step::Field(Selection::Nominal(id))) => {
                    write!(out, " nominal-field={}", id.0)?;
                }
                Some(Step::UnionField(field)) => write!(out, " union-field={field:?}")?,
                Some(Step::Element(index)) => write!(out, " index={index}")?,
                Some(Step::Range(start, end)) => write!(out, " range={start}..{end}")?,
                Some(Step::FromEnd { offset, minimum }) => {
                    write!(out, " from-end={offset} minimum={minimum}")?;
                }
                Some(Step::Subslice { prefix, suffix }) => {
                    write!(out, " subslice prefix={prefix} suffix={suffix}")?;
                }
                Some(Step::Dynamic(value)) => write!(out, " dynamic=v{}", value.0)?,
                None => {}
            }
            writeln!(out, " shared={}", path.shared)?;
        }
        Ok(())
    }
}
