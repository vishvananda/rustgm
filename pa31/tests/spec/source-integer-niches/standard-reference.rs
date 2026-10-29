use std::num::*;
type Nz_u8=NonZeroU8;
#[unsafe(no_mangle)]pub extern "C" fn echo_u8(x:Option<Nz_u8>)->Option<Nz_u8>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_u8(x:u8)->Option<Nz_u8>{Nz_u8::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_u8(x:Option<Nz_u8>)->u8{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_u8(x:Option<Nz_u8>)->Option<Nz_u8>{let f:extern "C" fn(Option<Nz_u8>)->Option<Nz_u8>=echo_u8;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_u8()->usize{std::mem::size_of::<Option<Nz_u8>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_u8()->usize{std::mem::align_of::<Option<Nz_u8>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_u8()->u8{unsafe{std::mem::transmute::<Option<Nz_u8>,u8>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_u8()->u8{unsafe{std::mem::transmute::<Option<Nz_u8>,u8>(Some(Nz_u8::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_u8()->u8{const VALUE:Option<Nz_u8>=unsafe{std::mem::transmute::<u8,Option<Nz_u8>>(255)};get_u8(VALUE)}
type Nz_u16=NonZeroU16;
#[unsafe(no_mangle)]pub extern "C" fn echo_u16(x:Option<Nz_u16>)->Option<Nz_u16>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_u16(x:u16)->Option<Nz_u16>{Nz_u16::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_u16(x:Option<Nz_u16>)->u16{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_u16(x:Option<Nz_u16>)->Option<Nz_u16>{let f:extern "C" fn(Option<Nz_u16>)->Option<Nz_u16>=echo_u16;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_u16()->usize{std::mem::size_of::<Option<Nz_u16>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_u16()->usize{std::mem::align_of::<Option<Nz_u16>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_u16()->u16{unsafe{std::mem::transmute::<Option<Nz_u16>,u16>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_u16()->u16{unsafe{std::mem::transmute::<Option<Nz_u16>,u16>(Some(Nz_u16::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_u16()->u16{const VALUE:Option<Nz_u16>=unsafe{std::mem::transmute::<u16,Option<Nz_u16>>(65535)};get_u16(VALUE)}
type Nz_u32=NonZeroU32;
#[unsafe(no_mangle)]pub extern "C" fn echo_u32(x:Option<Nz_u32>)->Option<Nz_u32>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_u32(x:u32)->Option<Nz_u32>{Nz_u32::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_u32(x:Option<Nz_u32>)->u32{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_u32(x:Option<Nz_u32>)->Option<Nz_u32>{let f:extern "C" fn(Option<Nz_u32>)->Option<Nz_u32>=echo_u32;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_u32()->usize{std::mem::size_of::<Option<Nz_u32>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_u32()->usize{std::mem::align_of::<Option<Nz_u32>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_u32()->u32{unsafe{std::mem::transmute::<Option<Nz_u32>,u32>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_u32()->u32{unsafe{std::mem::transmute::<Option<Nz_u32>,u32>(Some(Nz_u32::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_u32()->u32{const VALUE:Option<Nz_u32>=unsafe{std::mem::transmute::<u32,Option<Nz_u32>>(4294967295)};get_u32(VALUE)}
type Nz_u64=NonZeroU64;
#[unsafe(no_mangle)]pub extern "C" fn echo_u64(x:Option<Nz_u64>)->Option<Nz_u64>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_u64(x:u64)->Option<Nz_u64>{Nz_u64::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_u64(x:Option<Nz_u64>)->u64{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_u64(x:Option<Nz_u64>)->Option<Nz_u64>{let f:extern "C" fn(Option<Nz_u64>)->Option<Nz_u64>=echo_u64;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_u64()->usize{std::mem::size_of::<Option<Nz_u64>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_u64()->usize{std::mem::align_of::<Option<Nz_u64>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_u64()->u64{unsafe{std::mem::transmute::<Option<Nz_u64>,u64>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_u64()->u64{unsafe{std::mem::transmute::<Option<Nz_u64>,u64>(Some(Nz_u64::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_u64()->u64{const VALUE:Option<Nz_u64>=unsafe{std::mem::transmute::<u64,Option<Nz_u64>>(18446744073709551615)};get_u64(VALUE)}
type Nz_u128=NonZeroU128;
#[unsafe(no_mangle)]pub extern "C" fn echo_u128(x:Option<Nz_u128>)->Option<Nz_u128>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_u128(x:u128)->Option<Nz_u128>{Nz_u128::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_u128(x:Option<Nz_u128>)->u128{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_u128(x:Option<Nz_u128>)->Option<Nz_u128>{let f:extern "C" fn(Option<Nz_u128>)->Option<Nz_u128>=echo_u128;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_u128()->usize{std::mem::size_of::<Option<Nz_u128>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_u128()->usize{std::mem::align_of::<Option<Nz_u128>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_u128()->u128{unsafe{std::mem::transmute::<Option<Nz_u128>,u128>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_u128()->u128{unsafe{std::mem::transmute::<Option<Nz_u128>,u128>(Some(Nz_u128::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_u128()->u128{const VALUE:Option<Nz_u128>=unsafe{std::mem::transmute::<u128,Option<Nz_u128>>(340282366920938463463374607431768211455)};get_u128(VALUE)}
type Nz_usize=NonZeroUsize;
#[unsafe(no_mangle)]pub extern "C" fn echo_usize(x:Option<Nz_usize>)->Option<Nz_usize>{x}
#[unsafe(no_mangle)]pub extern "C" fn make_usize(x:usize)->Option<Nz_usize>{Nz_usize::new(x)}
#[unsafe(no_mangle)]pub extern "C" fn get_usize(x:Option<Nz_usize>)->usize{match x{Some(x)=>x.get(),None=>0}}
#[unsafe(no_mangle)]pub extern "C" fn invoke_usize(x:Option<Nz_usize>)->Option<Nz_usize>{let f:extern "C" fn(Option<Nz_usize>)->Option<Nz_usize>=echo_usize;f(x)}
#[unsafe(no_mangle)]pub extern "C" fn size_usize()->usize{std::mem::size_of::<Option<Nz_usize>>()}
#[unsafe(no_mangle)]pub extern "C" fn align_usize()->usize{std::mem::align_of::<Option<Nz_usize>>()}
#[unsafe(no_mangle)]pub extern "C" fn constant_none_usize()->usize{unsafe{std::mem::transmute::<Option<Nz_usize>,usize>(None)}}
#[unsafe(no_mangle)]pub extern "C" fn constant_some_usize()->usize{unsafe{std::mem::transmute::<Option<Nz_usize>,usize>(Some(Nz_usize::new_unchecked(1)))}}
#[unsafe(no_mangle)]pub extern "C" fn constant_round_usize()->usize{const VALUE:Option<Nz_usize>=unsafe{std::mem::transmute::<usize,Option<Nz_usize>>(18446744073709551615)};get_usize(VALUE)}
#[repr(transparent)]pub struct Outer(Nz_u8);
#[unsafe(no_mangle)]pub extern "C" fn nested(x:Option<Outer>)->Option<Outer>{x}
#[repr(C)]pub struct Pair{pub a:u64,pub b:u64}
#[unsafe(no_mangle)]pub extern "C" fn mixed(x:Option<Nz_u8>,p:Pair)->Option<Nz_u8>{let _=p;x}
#[repr(C)]pub struct Packet{pub value:Option<Nz_u8>,pub stamp:u32}
#[unsafe(no_mangle)]pub extern "C" fn packet(x:Packet)->Packet{x}
