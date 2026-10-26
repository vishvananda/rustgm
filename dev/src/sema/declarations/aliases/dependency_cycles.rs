//! The canonical owner graph is checked once, without flattening shared ancestors.
use super::Pending;
use crate::support::retained::Growth;
impl Pending {
    pub(super) fn close(&mut self) -> Result<(), &'static str> {
        if self.closed {
            return Err("trait alias graph is already closed");
        }
        let count = self.headers.len();
        self.budget
            .charge(count, count.saturating_mul(size_of::<u8>()))?;
        let mut states = vec![0_u8; count];
        let mut stack = Vec::<(usize, usize)>::new();
        for root in 0..count {
            if states[root] != 0 {
                continue;
            }
            self.push_frame(&mut stack, (root, 0))?;
            states[root] = 1;
            while let Some((index, next)) = stack.last_mut() {
                self.budget.charge(1, 0)?;
                let header = &self.headers[*index];
                if header.schema.is_none() || !header.completed {
                    return Err("trait alias graph contains an unfinished declaration");
                }
                let Some(edge) = header.edges.get(*next) else {
                    states[*index] = 2;
                    stack.pop();
                    continue;
                };
                *next += 1;
                let child = *self
                    .index
                    .get(&edge.target)
                    .ok_or("alias dependency has no canonical owner")?;
                match states[child] {
                    1 => return Err("cycle in trait alias declarations"),
                    2 => {}
                    _ => {
                        self.push_frame(&mut stack, (child, 0))?;
                        states[child] = 1;
                    }
                }
            }
        }
        self.closed = true;
        Ok(())
    }
    fn push_frame(
        &self,
        stack: &mut Vec<(usize, usize)>,
        frame: (usize, usize),
    ) -> Result<(), &'static str> {
        self.budget.grow(Growth::vector::<(usize, usize)>(
            stack.len(),
            stack.capacity(),
            1,
        ))?;
        stack.push(frame);
        Ok(())
    }
}
