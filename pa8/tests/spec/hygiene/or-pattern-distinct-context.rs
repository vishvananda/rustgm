macro_rules! p{($x:ident)=>{(x,0)|(0,$x)}}fn f(){match(0,0){p!(x)=>{},_=>{}}}
