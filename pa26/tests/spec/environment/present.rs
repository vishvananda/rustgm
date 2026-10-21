fn main(){let x:Option<&'static str>=option_env!("KEY");match x{Some(_)=>{},None=>{}}}
