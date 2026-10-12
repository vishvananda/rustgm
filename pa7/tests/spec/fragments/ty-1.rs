macro_rules! m { ($v:ty) => {} }
m!(unsafe extern "C" fn(i32)->i32);
