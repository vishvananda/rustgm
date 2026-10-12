macro_rules! m { ($v:expr) => {} }
m!('a: loop {break 'a 1});
