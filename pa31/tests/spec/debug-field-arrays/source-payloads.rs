// Authored consumer payload; append only after a genuine source Debug/Formatter provider.
// No library implementation is supplied here. Root owns the source-provider consumer.
#[derive(Debug)]
struct R7 { a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8 }
#[derive(Debug)]
struct T7(u8, u8, u8, u8, u8, u8, u8);
#[derive(Debug)]
enum E7 { Record { a: u8, b: u8, c: u8, d: u8, e: u8, f: u8, g: u8 }, Tuple(u8, u8, u8, u8, u8, u8, u8) }
