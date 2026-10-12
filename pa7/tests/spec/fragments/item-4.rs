macro_rules! m { ($v:item) => {} }
m!(#[cfg(nope)] fn hidden() {});
