struct S;
impl S {
    fn borrow<'a>(&'a self) -> &'a S { self }
}
