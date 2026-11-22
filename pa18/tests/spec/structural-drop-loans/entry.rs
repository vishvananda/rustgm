struct Table<K>(K);
struct Occupied<'a, K> {
    hash: u64,
    table: &'a mut Table<K>,
}
struct Vacant<'a, K> {
    hash: u64,
    key: K,
    table: &'a mut Table<K>,
}
enum Entry<'a, K> {
    Occupied(Occupied<'a, K>),
    Vacant(Vacant<'a, K>),
}

fn into_map<'a, K>(entry: Entry<'a, K>) -> &'a mut Table<K> {
    match entry {
        Entry::Occupied(entry) => entry.table,
        Entry::Vacant(entry) => entry.table,
    }
}

struct Wrapper<'a, K>(Vacant<'a, K>);

fn nested<'a, K>(entry: Wrapper<'a, K>) -> &'a mut Table<K> {
    entry.0.table
}

struct Flag<'a>(&'a mut u8);

impl Drop for Flag<'_> {
    fn drop(&mut self) {
        *self.0 = 1;
    }
}

struct Owned<'a, 'key> {
    key: Flag<'key>,
    table: &'a mut u8,
}

fn into_ref<'a>(entry: Owned<'a, '_>) -> &'a mut u8 {
    entry.table
}

#[cfg(custom_drop)]
fn conflict(entry: Flag<'_>) -> &mut u8 {
    entry.0
}

#[cfg(nested_drop)]
struct Layer<'a>(Flag<'a>);

#[cfg(nested_drop)]
fn conflict(entry: Layer<'_>) -> &mut u8 {
    entry.0.0
}

fn main() {
    let mut table = Table(0);
    let entry = Entry::Vacant(Vacant {
        hash: 0,
        key: 41,
        table: &mut table,
    });
    into_map(entry).0 = 42;

    let mut dropped = 0;
    let mut value = 0;
    *into_ref(Owned {
        key: Flag(&mut dropped),
        table: &mut value,
    }) = 5;
    match (table.0, dropped, value) {
        (42, 1, 5) => (),
        _ => loop {},
    }
}
