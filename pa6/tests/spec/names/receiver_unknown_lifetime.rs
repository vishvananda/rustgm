struct S;
impl S {
    fn borrow(&'missing self) -> &S { self }
}
