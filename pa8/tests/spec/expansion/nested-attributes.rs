macro_rules! m{()=>{#[cfg_attr(all(),cfg(any()))] fn off(){missing!();}}} m!();
