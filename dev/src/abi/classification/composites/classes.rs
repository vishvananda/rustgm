//! Offset phase preserves mixed nested fields crossing an outer eightbyte boundary.
use super::{Class, Key, Kind, Node, Query};
struct Child {
    key: Key,
    word: usize,
}
enum Work {
    Enter(Key),
    Finish(Key, Vec<Child>),
}
impl Query<'_> {
    pub(super) fn placed(&mut self, root: Key) -> Result<[Class; 2], &'static str> {
        let mut work = vec![Work::Enter(root)];
        while let Some(next) = work.pop() {
            self.charge(1)?;
            match next {
                Work::Enter(key) => {
                    if self.cache.classes.contains_key(&key) {
                        continue;
                    }
                    let node = self
                        .cache
                        .nodes
                        .get(&key.ty)
                        .ok_or("missing validated C field graph")?;
                    if node
                        .bytes
                        .checked_add(u64::from(key.phase))
                        .is_none_or(|end| end > 16)
                    {
                        return Err("placed C field exceeds two eightbytes");
                    }
                    if matches!(node.kind, Kind::Vector) {
                        if key.phase != 0 {
                            return Err("C SIMD field is not aligned to its vector boundary");
                        }
                        self.publish(key, [Class::Sse, Class::SseUp])?;
                    } else if let Kind::Scalar(class) = node.kind {
                        let mut classes = [Class::None; 2];
                        if node.bytes != 0 {
                            let last = (u64::from(key.phase) + node.bytes - 1) / 8;
                            for slot in &mut classes
                                [..=usize::try_from(last).map_err(|_| "C class index overflow")?]
                            {
                                *slot = class;
                            }
                        }
                        self.publish(key, classes)?;
                    } else {
                        let children = children(node, key.phase)?;
                        self.charge(children.len())?;
                        let keys = children
                            .iter()
                            .rev()
                            .map(|child| Work::Enter(child.key))
                            .collect::<Vec<_>>();
                        work.push(Work::Finish(key, children));
                        work.extend(keys);
                    }
                }
                Work::Finish(key, children) => {
                    let mut classes = [Class::None; 2];
                    for child in children {
                        self.charge(1)?;
                        let summary = self
                            .cache
                            .classes
                            .get(&child.key)
                            .ok_or("missing placed C child classes")?;
                        for (index, class) in summary.iter().copied().enumerate() {
                            if class == Class::None {
                                continue;
                            }
                            let slot = classes
                                .get_mut(child.word + index)
                                .ok_or("nested C class exceeds object span")?;
                            *slot = match (*slot, class) {
                                (Class::Integer, _) | (_, Class::Integer) => Class::Integer,
                                (Class::Sse, _) | (_, Class::Sse) => Class::Sse,
                                _ => class,
                            };
                        }
                    }
                    self.publish(key, classes)?;
                }
            }
        }
        self.cache
            .classes
            .get(&root)
            .copied()
            .ok_or("missing C object classes")
    }
    fn publish(&mut self, key: Key, classes: [Class; 2]) -> Result<(), &'static str> {
        self.retain(size_of::<Key>() + size_of::<[Class; 2]>() + 48)?;
        self.cache.classes.insert(key, classes);
        Ok(())
    }
}
fn children(node: &Node, phase: u8) -> Result<Vec<Child>, &'static str> {
    let mut children = Vec::new();
    let mut push = |ty, offset: u64| -> Result<(), &'static str> {
        let offset = u64::from(phase)
            .checked_add(offset)
            .ok_or("C field placement overflow")?;
        children.push(Child {
            key: Key {
                ty,
                phase: u8::try_from(offset % 8).map_err(|_| "C phase overflow")?,
            },
            word: usize::try_from(offset / 8).map_err(|_| "C class index overflow")?,
        });
        Ok(())
    };
    match &node.kind {
        Kind::Scalar(_) | Kind::Vector => return Err("scalar C field has no children"),
        Kind::Struct(fields) => {
            for field in fields {
                push(field.ty, field.offset)?;
            }
        }
        Kind::Array {
            element,
            stride,
            length,
        } => {
            if *stride != 0 {
                for index in 0..*length {
                    push(
                        *element,
                        index
                            .checked_mul(*stride)
                            .ok_or("C array placement overflow")?,
                    )?;
                }
            }
        }
    }
    Ok(children)
}
