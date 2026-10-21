fn main(){let x:Option<&'static str>=option_env!("MISSING");match x{Some(_)=>{},None=>{}}}
