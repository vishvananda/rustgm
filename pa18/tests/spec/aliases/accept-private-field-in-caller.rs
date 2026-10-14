type A<const N:usize>=[u8;N];mod m{struct S{n:usize}fn f(){let _:super::A<{S{n:3}.n}>;}}
fn main(){}
