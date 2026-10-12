macro_rules! ignore {($($t:tt)*)=>{}} ignore!(#[cfg(bad())] 1suffix);
