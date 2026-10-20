trait T{}fn check<'a>(_: &'a (dyn T+'a)){}
fn main(){}
