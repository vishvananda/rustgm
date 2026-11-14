const fn word(x:u8)->u8{x}
#[repr(C)]struct Row{data:&'static [u8]}
#[repr(C)]struct Table{rows:[Row;2]}
static A:[u8;2]=[word(7),word(11)];static B:[u8;2]=[word(13),word(14)];static DATA:Table=Table{rows:[Row{data:&A},Row{data:&B}]};
#[unsafe(no_mangle)]pub extern "C" fn table()->*const Table{&DATA as *const Table}
