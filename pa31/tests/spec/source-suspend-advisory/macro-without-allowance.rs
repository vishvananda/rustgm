macro_rules! make{($a:meta)=>{#[$a]pub struct S;}}make!(must_not_suspend);
