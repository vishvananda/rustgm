macro_rules! m { ($v:ty) => {} }
m!(dyn Send + Sync);
