struct Token(u32);#[repr(packed)]struct P{a:Token,b:u32}fn main(){let p=P{a:Token(1),b:7};let c=move||p.b;let owned=p.a;let _=c();let _=owned.0;}
