macro_rules! skip { ($($t:tt)*) => {}; }
skip!(#"x"# ##);
