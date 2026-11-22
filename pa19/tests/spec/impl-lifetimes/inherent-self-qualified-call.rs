struct DomainIndex<'index, 'rows> {
    first: &'index &'rows u32,
}

struct Work {
    visits: u32,
}

impl DomainIndex<'_, '_> {
    fn scan(&self, original: &u32, work: &mut Work) -> bool {
        **self.first != 0 && Self::scopes(original, work)
    }

    fn scopes(original: &u32, work: &mut Work) -> bool {
        Self::site(original, work) && Self::site(original, work)
    }
}

impl DomainIndex<'_, '_> {
    fn site(original: &u32, work: &mut Work) -> bool {
        work.visits += 1;
        *original != 0
    }
}

fn fail(zero: u32) -> u32 {
    1 / zero
}

fn main() {
    let value = 7;
    let root = &value;
    let mut work = Work { visits: 0 };
    let original = 9;
    let index = DomainIndex { first: &root };
    if !index.scan(&original, &mut work) || work.visits != 2 {
        let _ = fail(0);
    }
}
