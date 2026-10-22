pub struct Record { pub value:u32 }
static PRIVATE:Record=Record{value:31};
pub static FIELD:&u32=&PRIVATE.value;
pub static ALIAS:&u32=FIELD;
pub static WHOLE:&Record=&PRIVATE;
