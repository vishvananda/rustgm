const fn word(x:u8)->u8{x}
struct Row{data:&'static [u8]}
struct Table{rows:[Row;2]}
const DATA:Table=Table{rows:[Row{data:&[word(7),word(11)]},Row{data:&[word(13),word(14)]}]};
fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}
fn main(){check(DATA.rows[0].data[0]==7);check(DATA.rows[0].data[1]==11);check(DATA.rows[1].data[0]==13);check(DATA.rows[1].data[1]==14);}
