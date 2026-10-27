#![feature(cfg_target_has_atomic)]
const _: [();1]=[();cfg!(target_has_atomic_load_store) as usize];fn main(){}
