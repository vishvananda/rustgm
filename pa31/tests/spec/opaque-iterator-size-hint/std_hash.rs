use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct WordHasher(u64);

impl Hasher for WordHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.wrapping_mul(31).wrapping_add(u64::from(byte));
        }
    }
}

#[derive(Clone, Copy)]
struct Key(u32);

#[derive(Clone, Copy)]
enum Subject {
    Direct(u32),
    Indirect(usize),
}

#[derive(Clone, Copy)]
struct Syntax {
    subject: Subject,
    source: Key,
}

struct Checker<'a> {
    rows: HashMap<usize, Vec<Syntax>, BuildHasherDefault<WordHasher>>,
    definitions: &'a [u32],
}

impl Checker<'_> {
    #[inline(never)]
    fn declared_self_bounds(
        &self,
        owner: usize,
        definition: u32,
    ) -> impl Iterator<Item = (Key, u32)> + '_ {
        self.rows
            .get(&owner)
            .into_iter()
            .flatten()
            .filter_map(move |syntax| {
                let subject = match syntax.subject {
                    Subject::Direct(id) => Some(id),
                    Subject::Indirect(key) => self.definitions.get(key).copied(),
                };
                (subject == Some(definition)).then_some((syntax.source, definition))
            })
    }
}

fn checker(definitions: &[u32]) -> Checker<'_> {
    let mut rows = HashMap::default();
    rows.insert(
        0,
        vec![
            Syntax {
                subject: Subject::Direct(11),
                source: Key(30),
            },
            Syntax {
                subject: Subject::Indirect(0),
                source: Key(42),
            },
            Syntax {
                subject: Subject::Direct(11),
                source: Key(17),
            },
        ],
    );
    Checker { rows, definitions }
}

#[inline(never)]
fn bounds(checker: &Checker<'_>, owner: usize, definition: u32) -> Vec<Key> {
    checker
        .declared_self_bounds(owner, definition)
        .map(|(source, _)| source)
        .collect()
}

fn main() {
    let definitions = [19];
    let checker = checker(&definitions);
    let rows = bounds(&checker, 0, 11);
    if rows.len() != 2
        || rows[0].0 != 30
        || rows[1].0 != 17
        || bounds(&checker, 0, 19)[0].0 != 42
        || !bounds(&checker, 1, 11).is_empty()
    {
        std::process::exit(1);
    }
}
