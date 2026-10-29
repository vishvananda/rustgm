#![feature(staged_api)]#![unstable(feature="associated_derive_fixture",issue="none")]
#[stable(feature = "decode_utf16", since = "1.9.0")]
#[derive(Clone, Debug)]
pub struct DecodeUtf16<I>
where
    I: Iterator<Item = u16>,
{
    iter: I,
    buf: Option<u16>,
}
