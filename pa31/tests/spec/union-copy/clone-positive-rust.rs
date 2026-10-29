#[derive(Copy,Clone)]union Value<T:Copy>{value:T,empty:()}
fn copy_uninitialized(){let value=Value::<u32>{empty:()};let _=value.clone();let _=value;}
