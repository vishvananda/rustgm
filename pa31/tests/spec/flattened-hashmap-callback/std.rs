use std::collections::HashMap;
use std::hash::{BuildHasherDefault, Hasher};

#[derive(Default)]
struct NumberHasher(u64);

impl Hasher for NumberHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.0 = self.0.wrapping_mul(17).wrapping_add(u64::from(byte));
        }
    }
}

#[derive(Clone, Copy)]
struct Group {
    key: u8,
}

struct Entry {
    item: u16,
    groups: Vec<Group>,
}

struct Checker<'a> {
    entries: Vec<Entry>,
    targets: &'a mut [u16; 4],
}

impl Checker<'_> {
    fn index(&mut self) {
        let groups: HashMap<_, _, BuildHasherDefault<NumberHasher>> = self
            .entries
            .iter()
            .flat_map(|entry| entry.groups.iter().map(|group| (group.key, entry.item)))
            .collect();
        for (key, item) in groups {
            self.targets[usize::from(key)] = item;
        }
    }
}

fn main() {
    let entries = vec![
        Entry { item: 11, groups: vec![Group { key: 0 }, Group { key: 2 }] },
        Entry { item: 37, groups: vec![Group { key: 1 }, Group { key: 3 }] },
    ];
    let mut targets = [0; 4];
    Checker { entries, targets: &mut targets }.index();
    if targets != [11, 37, 11, 37] {
        std::process::exit(1);
    }
}
