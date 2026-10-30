pub struct S;macro_rules! make{($a:meta)=>{impl S{#[$a]pub fn f()->u32{7}}}}make!(rustc_should_not_be_called_on_const_items);
