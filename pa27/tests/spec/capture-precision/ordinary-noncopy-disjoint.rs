struct Token(u32);struct P{a:Token,b:u32}fn main(){let p=P{a:Token(1),b:7};let c=||p.b;let owned=p.a;let _=c();let _=owned.0;}
