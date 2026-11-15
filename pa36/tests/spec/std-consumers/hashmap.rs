use std::collections::hash_map::{DefaultHasher, Entry};
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher, RandomState};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum Key {
    Id(u32),
    Name(String),
    Pair(u8, char),
}

#[derive(Default)]
struct Fnv(u64);

impl Hasher for Fnv {
    fn finish(&self) -> u64 {
        self.0
    }
    fn write(&mut self, bytes: &[u8]) {
        if self.0 == 0 {
            self.0 = 0xcbf2_9ce4_8422_2325;
        }
        for byte in bytes {
            self.0 = (self.0 ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3);
        }
    }
}

fn sorted<K: Ord + Clone, V: Clone>(map: &HashMap<K, V>) -> Vec<(K, V)> {
    let mut entries: Vec<(K, V)> = map.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    entries.sort_by(|a, b| a.0.cmp(&b.0));
    entries
}

fn sorted_set<T: Ord + Clone, S: BuildHasher>(set: &HashSet<T, S>) -> Vec<T> {
    let mut items: Vec<T> = set.iter().cloned().collect();
    items.sort();
    items
}

fn main() {
    let mut map: HashMap<String, i32> = HashMap::new();
    println!("empty {} {} {:?}", map.len(), map.is_empty(), map.get("missing"));
    for (index, word) in ["one", "two", "three", "four", "five"].iter().enumerate() {
        map.insert(word.to_string(), index as i32 + 1);
    }
    println!("insert {:?} {:?} {}", map.insert("two".to_string(), 22), map.insert("six".to_string(), 6), map.len());
    println!("get {:?} {:?} {} {}", map.get("three"), map.get("seven"), map.contains_key("four"), map["one"]);
    println!("remove {:?} {:?} {:?}", map.remove("one"), map.remove("one"), map.remove_entry("five"));
    if let Some(value) = map.get_mut("six") {
        *value *= 100;
    }
    *map.entry("two".to_string()).or_insert(0) += 1;
    *map.entry("ten".to_string()).or_insert(0) += 10;
    map.entry("six".to_string()).and_modify(|v| *v += 1).or_default();
    map.entry("zero".to_string()).or_default();
    match map.entry("four".to_string()) {
        Entry::Occupied(slot) => println!("occupied {} {}", slot.key(), slot.get()),
        Entry::Vacant(_) => println!("vacant"),
    }
    println!("entries {:?}", sorted(&map));
    let mut keys: Vec<&String> = map.keys().collect();
    keys.sort();
    let mut values: Vec<i32> = map.values().copied().collect();
    values.sort_unstable();
    println!("keys {:?} values {:?} sum {}", keys, values, map.values().sum::<i32>());
    map.retain(|key, value| key.len() == 3 && *value > 0);
    println!("retain {:?}", sorted(&map));
    for value in map.values_mut() {
        *value = -*value;
    }
    let drained: i32 = map.drain().map(|(_, value)| value).sum();
    println!("drain {} {} {}", drained, map.len(), map.capacity() > 0);

    let mut large: HashMap<u64, u64> = HashMap::with_capacity(16);
    let mut state = 1u64;
    for index in 0..20_000u64 {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        *large.entry(state >> 50).or_insert(0) += index;
    }
    let mut checksum = 0u64;
    let mut ordered: Vec<(&u64, &u64)> = large.iter().collect();
    ordered.sort();
    for (key, value) in &ordered {
        checksum = checksum.wrapping_mul(1_000_003).wrapping_add(**key ^ **value);
    }
    println!("large {} {} {:?} {:?}", large.len(), checksum, ordered.first(), ordered.last());
    let mut removed = 0;
    for key in 0..8192 {
        if key % 3 == 0 && large.remove(&key).is_some() {
            removed += 1;
        }
    }
    large.shrink_to_fit();
    println!("removed {} {} {}", removed, large.len(), large.keys().all(|k| k % 3 != 0));

    let mut typed: HashMap<Key, Vec<u8>> = HashMap::new();
    typed.entry(Key::Id(7)).or_default().push(1);
    typed.entry(Key::Name("seven".into())).or_default().push(2);
    typed.entry(Key::Pair(7, 'x')).or_default().push(3);
    typed.entry(Key::Id(7)).or_default().push(4);
    println!("typed {:?} {:?}", sorted(&typed), typed.get(&Key::Pair(7, 'y')));
    let tuple_keys: HashMap<(i32, i32), &str> = [((0, 0), "origin"), ((1, -1), "south-east"), ((0, 0), "replaced")].into_iter().collect();
    println!("tuples {:?} {}", sorted(&tuple_keys), tuple_keys.len());
    let borrowed: HashMap<&str, usize> = "the quick brown the lazy the end".split(' ').fold(HashMap::new(), |mut counts, word| {
        *counts.entry(word).or_insert(0) += 1;
        counts
    });
    let mut top: Vec<(&str, usize)> = borrowed.into_iter().collect();
    top.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(b.0)));
    println!("counts {:?}", top);

    let mut set: HashSet<i32> = [5, 1, 9, 3, 7, 3, 1].into_iter().collect();
    println!("set {:?} {} {} {}", sorted_set(&set), set.len(), set.insert(4), set.insert(5));
    println!("set {} {} {:?} {}", set.contains(&9), set.remove(&2), set.take(&7), set.len());
    let evens: HashSet<i32> = (0..10).filter(|v| v % 2 == 0).collect();
    let mut union: Vec<i32> = set.union(&evens).copied().collect();
    union.sort_unstable();
    let mut common: Vec<i32> = set.intersection(&evens).copied().collect();
    common.sort_unstable();
    let mut only: Vec<i32> = set.difference(&evens).copied().collect();
    only.sort_unstable();
    let mut either: Vec<i32> = set.symmetric_difference(&evens).copied().collect();
    either.sort_unstable();
    println!("union {:?} intersection {:?} difference {:?} symmetric {:?}", union, common, only, either);
    println!(
        "relations {} {} {} {}",
        HashSet::from([4]).is_subset(&set),
        set.is_superset(&evens),
        set.is_disjoint(&HashSet::from([100])),
        set == [1, 3, 4, 5, 9].into_iter().collect::<HashSet<_>>()
    );
    let words: HashSet<String> = ["apple", "Apple", "APPLE", "pear"].iter().map(|w| w.to_lowercase()).collect();
    println!("words {:?} {}", sorted_set(&words), words.contains("pear"));
    let mut seen = HashSet::new();
    let duplicates: Vec<char> = "mississippi".chars().filter(|c| !seen.insert(*c)).collect();
    println!("duplicates {:?} unique {}", duplicates, seen.len());

    let mut fixed: HashMap<&str, u32, BuildHasherDefault<Fnv>> = HashMap::default();
    for (index, name) in ["alpha", "beta", "gamma", "delta"].iter().enumerate() {
        fixed.insert(name, index as u32);
    }
    let mut fixed_entries: Vec<(&&str, &u32)> = fixed.iter().collect();
    fixed_entries.sort();
    println!("fixed hasher {:?} {:?}", fixed_entries, fixed.get("gamma"));
    let fixed_set: HashSet<u8, BuildHasherDefault<Fnv>> = (0..50).map(|v| v % 7).collect();
    println!("fixed set {:?}", sorted_set(&fixed_set));
    let mut fnv = Fnv::default();
    "hash me".hash(&mut fnv);
    42u32.hash(&mut fnv);
    println!("fnv {:016x}", fnv.finish());
    let state = RandomState::new();
    let same = state.hash_one("stable within one state") == state.hash_one("stable within one state");
    let mut first = DefaultHasher::new();
    let mut second = DefaultHasher::new();
    (1u8, "x").hash(&mut first);
    (1u8, "x").hash(&mut second);
    println!("hashers {} {} {:016x}", same, first.finish() == second.finish(), first.finish());
    let nested: HashMap<u8, HashMap<u8, HashSet<u8>>> = (0..3).map(|a| (a, (0..2).map(|b| (b, (0..=a + b).collect())).collect())).collect();
    let total: usize = nested.values().flat_map(|inner| inner.values()).map(HashSet::len).sum();
    println!("nested {} {:?}", total, nested.get(&2).and_then(|inner| inner.get(&1)).map(sorted_set));
    let single: HashMap<&str, i32> = HashMap::from([("only", 1)]);
    println!("debug {:?} {:?} {:?}", single, HashSet::from(['x']), HashMap::<u8, u8>::new());
}
