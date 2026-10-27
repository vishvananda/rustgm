#![feature(cfg_target_has_atomic)]
const _: [();1]=[();cfg!(all(target_has_atomic_load_store="8",target_has_atomic_load_store="16",target_has_atomic_load_store="32",target_has_atomic_load_store="64",target_has_atomic_load_store="ptr",not(target_has_atomic_load_store="128"))) as usize];fn main(){}
