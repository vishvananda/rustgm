macro_rules! m { ($v:meta) => {} }
m!(cfg(any(unix, windows)));
