macro_rules! select{($p:pat,$x:ident)=>{match(0,1){$p=>$x,_=>0}}}fn f()->i32{select!((x,0)|(0,x),x)}
