fn check(ok: bool) { if !ok { let _ = fail(0); } } fn fail(x: u8) -> u8 { 1 / x }
fn main() {
    check(facade::value(facade::make()) == 7);
    check(facade::other().value == 9);
    check(facade::api::again::make().value == 7);
    check(facade::nested::build().value == 7);
    check(facade::original::NUMBER == 7);
    check((&facade::DATA as *const u32) == (&one::DATA as *const u32));
    check((&facade::DATA as *const u32) != (&two::DATA as *const u32));
    check((facade::alias as fn() -> facade::Handle as usize) == (one::make as fn() -> one::Handle as usize));
}
