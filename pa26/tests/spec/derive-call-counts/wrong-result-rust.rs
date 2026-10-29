const fn count()->bool{true}
#[derive(Clone,Copy)]struct Bad([u8;count()]);
