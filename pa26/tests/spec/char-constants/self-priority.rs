fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Local{const MIN:char;const MAX:char;fn f()->char;}impl Local for char{const MIN:char='a';const MAX:char='z';fn f()->char{Self::MAX}}fn main(){check(<char as Local>::MIN=='a');check(<char as Local>::MAX=='z');check(<char as Local>::f()=='\u{10ffff}');check(char::MIN=='\0');}
