#![allow(internal_features,dead_code)]
#![feature(link_llvm_intrinsics)]
unsafe extern "C"{#[link_name="llvm.ctpop.i8"]fn ctpop_8(x:i8)->i8;
#[link_name="llvm.ctpop.i16"]fn ctpop_16(x:i16)->i16;
#[link_name="llvm.ctpop.i32"]fn ctpop_32(x:i32)->i32;
#[link_name="llvm.ctpop.i64"]fn ctpop_64(x:i64)->i64;
#[link_name="llvm.ctpop.i128"]fn ctpop_128(x:i128)->i128;
#[link_name="llvm.bitreverse.i8"]fn bitreverse_8(x:i8)->i8;
#[link_name="llvm.bitreverse.i16"]fn bitreverse_16(x:i16)->i16;
#[link_name="llvm.bitreverse.i32"]fn bitreverse_32(x:i32)->i32;
#[link_name="llvm.bitreverse.i64"]fn bitreverse_64(x:i64)->i64;
#[link_name="llvm.bitreverse.i128"]fn bitreverse_128(x:i128)->i128;
#[link_name="llvm.bswap.i16"]fn bswap_16(x:i16)->i16;
#[link_name="llvm.bswap.i32"]fn bswap_32(x:i32)->i32;
#[link_name="llvm.bswap.i64"]fn bswap_64(x:i64)->i64;
#[link_name="llvm.bswap.i128"]fn bswap_128(x:i128)->i128;
fn word(i:u8)->u64;fn observe(x:u8);}
fn main(){unsafe{let ok=(ctpop_8((word(2) as u8) as i8) as u8)==3u8&&
(ctpop_8((0) as i8) as u8)==0u8&&
(ctpop_8((!0u8) as i8) as u8)==8u8&&
(ctpop_8((1u8<<7) as i8) as u8)==1u8&&
(ctpop_16((word(2) as u16) as i16) as u16)==6u16&&
(ctpop_16((0) as i16) as u16)==0u16&&
(ctpop_16((!0u16) as i16) as u16)==16u16&&
(ctpop_16((1u16<<15) as i16) as u16)==1u16&&
(ctpop_32((word(2) as u32) as i32) as u32)==13u32&&
(ctpop_32((0) as i32) as u32)==0u32&&
(ctpop_32((!0u32) as i32) as u32)==32u32&&
(ctpop_32((1u32<<31) as i32) as u32)==1u32&&
(ctpop_64((word(2) as u64) as i64) as u64)==30u64&&
(ctpop_64((0) as i64) as u64)==0u64&&
(ctpop_64((!0u64) as i64) as u64)==64u64&&
(ctpop_64((1u64<<63) as i64) as u64)==1u64&&
(ctpop_128((((word(2) as u128)<<64)|(word(3) as u128)) as i128) as u128)==62u128&&
(ctpop_128((0) as i128) as u128)==0u128&&
(ctpop_128((!0u128) as i128) as u128)==128u128&&
(ctpop_128((1u128<<127) as i128) as u128)==1u128&&
(bitreverse_8((word(2) as u8) as i8) as u8)==200u8&&
(bitreverse_8((0) as i8) as u8)==0u8&&
(bitreverse_8((!0u8) as i8) as u8)==255u8&&
(bitreverse_8((1u8<<7) as i8) as u8)==1u8&&
(bitreverse_16((word(2) as u16) as i16) as u16)==51368u16&&
(bitreverse_16((0) as i16) as u16)==0u16&&
(bitreverse_16((!0u16) as i16) as u16)==65535u16&&
(bitreverse_16((1u16<<15) as i16) as u16)==1u16&&
(bitreverse_32((word(2) as u32) as i32) as u32)==3366498319u32&&
(bitreverse_32((0) as i32) as u32)==0u32&&
(bitreverse_32((!0u32) as i32) as u32)==4294967295u32&&
(bitreverse_32((1u32<<31) as i32) as u32)==1u32&&
(bitreverse_64((word(2) as u64) as i64) as u64)==14459000186291154305u64&&
(bitreverse_64((0) as i64) as u64)==0u64&&
(bitreverse_64((!0u64) as i64) as u64)==18446744073709551615u64&&
(bitreverse_64((1u64<<63) as i64) as u64)==1u64&&
(bitreverse_128((((word(2) as u128)<<64)|(word(3) as u128)) as i128) as u128)==329253067809259423328036374365149916545u128&&
(bitreverse_128((0) as i128) as u128)==0u128&&
(bitreverse_128((!0u128) as i128) as u128)==340282366920938463463374607431768211455u128&&
(bitreverse_128((1u128<<127) as i128) as u128)==1u128&&
(bswap_16((word(2) as u16) as i16) as u16)==4885u16&&
(bswap_16((0) as i16) as u16)==0u16&&
(bswap_16((!0u16) as i16) as u16)==65535u16&&
(bswap_16((1u16<<15) as i16) as u16)==128u16&&
(bswap_32((word(2) as u32) as i32) as u32)==320146928u32&&
(bswap_32((0) as i32) as u32)==0u32&&
(bswap_32((!0u32) as i32) as u32)==4294967295u32&&
(bswap_32((1u32<<31) as i32) as u32)==128u32&&
(bswap_64((word(2) as u64) as i64) as u64)==1375020589685454721u64&&
(bswap_64((0) as i64) as u64)==0u64&&
(bswap_64((!0u64) as i64) as u64)==18446744073709551615u64&&
(bswap_64((1u64<<63) as i64) as u64)==128u64&&
(bswap_128((((word(2) as u128)<<64)|(word(3) as u128)) as i128) as u128)==318753391026855559373516001042668238721u128&&
(bswap_128((0) as i128) as u128)==0u128&&
(bswap_128((!0u128) as i128) as u128)==340282366920938463463374607431768211455u128&&
(bswap_128((1u128<<127) as i128) as u128)==128u128;observe(if ok{1}else{0});}}
