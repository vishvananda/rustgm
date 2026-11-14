const fn word(x:u8)->u8{x}
#[repr(C)]struct Row{data:&'static [u8]}
#[repr(C)]struct Table{rows:[Row;2]}
const DATA:Table=Table{rows:[Row{data:&[word(7),word(11)]},Row{data:&[word(13),word(14)]}]};
#[unsafe(no_mangle)]pub extern "C" fn table()->*const Table{&DATA as *const Table}
