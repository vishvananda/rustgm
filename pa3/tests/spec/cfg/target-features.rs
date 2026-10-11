#[cfg(all(target_feature="sse",target_feature="sse2",target_feature="fxsr",not(target_feature="avx")))] fn selected() {}
