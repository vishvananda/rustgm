pub struct ManuallyDrop{pub value:u16}fn main(){let x=ManuallyDrop{value:7};let ManuallyDrop{value}=x;let _=value;}
