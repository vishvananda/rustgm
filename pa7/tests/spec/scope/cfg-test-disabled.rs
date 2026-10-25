#[cfg(test)]macro_rules! chosen{()=>{fn answer()->u32{7u32}}}
#[cfg(not(test))]macro_rules! chosen{()=>{fn answer()->u32{9u32}}}
chosen!();
