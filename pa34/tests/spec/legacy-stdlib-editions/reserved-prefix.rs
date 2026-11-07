macro_rules! skip { ($($t:tt)*) => {}; }
skip!(k#a f"x" z'c');
