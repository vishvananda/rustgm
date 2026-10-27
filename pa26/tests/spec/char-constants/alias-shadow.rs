fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct C;impl C{const MIN:u8=7;const MAX:u8=9;}type Glyph=C;fn main(){check(Glyph::MIN==7);check(Glyph::MAX==9);}
