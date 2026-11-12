fn make(value:u8)->impl Fn(bool)->u8{move|flag|match flag{true=>value,false=>7}}
