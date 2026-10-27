#![allow(dead_code,internal_features)]
fn f()->Result<u8,u16>{Ok(Err::<u8,u8>(3)?)}
