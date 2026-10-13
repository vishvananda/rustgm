fn required<'a:'static>(){}fn invalid<'a>(){required::<'a>();}
fn main() {}
