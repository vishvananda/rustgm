pub struct S;macro_rules! make{($a:meta)=>{impl S{#[$a]pub fn replace(&self)->u32{7}}}}make!(rustc_confusables("swap"));
