pub trait Value {
    type Item<'a>
    where
        Self: 'a;
}
