//! PA22 native controls distinguish semantic size proofs from physical metadata.
use crate::native_references as native;

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn projected_sized_wrappers_keep_thin_references_and_owned_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "trait Value{type Out:?Sized;}impl Value for u8{type Out=u16;}struct S<T:Value>{tag:u8,tail:T::Out}";
    let sources = [
        (
            "reference",
            "fn read(s:&S<u8>)->u16{s.tail}fn main(){let s=S::<u8>{tag:7,tail:31};check(read(&s)==31);check(s.tag==7);}",
        ),
        (
            "array",
            "fn read(s:&[S<u8>;2])->u16{s[0].tail+s[1].tail}fn main(){let values=[S::<u8>{tag:1,tail:11},S::<u8>{tag:2,tail:19}];check(read(&values)==30);}",
        ),
        (
            "mutable",
            "fn write(s:&mut S<u8>){s.tail=53;}fn main(){let mut s=S::<u8>{tag:7,tail:31};write(&mut s);check(s.tail==53);check(s.tag==7);}",
        ),
    ];
    let sources = sources.map(|(name, source)| (name, format!("{prefix}{source}")));
    let cases = sources
        .iter()
        .map(|(name, source)| (*name, source.as_str(), false))
        .collect::<Vec<_>>();
    native::check_cases("pa22-native/projected-storage", &cases)
}

#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn unsized_nominals_are_disjoint_from_sized_blanket_implementations()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases(
        "pa22-native/sized-selection",
        &[
            (
                "concrete",
                "struct S{prefix:u16,tail:str}trait Flag{fn n()->u8;}impl<T>Flag for T{fn n()->u8{1}}impl Flag for S{fn n()->u8{2}}fn main(){check(<S as Flag>::n()==2);check(<u8 as Flag>::n()==1);}",
                false,
            ),
            (
                "instantiated",
                "struct S<T:?Sized>{prefix:u16,tail:T}trait Flag{fn n()->u8;}impl<T>Flag for T{fn n()->u8{1}}impl Flag for S<[u8]>{fn n()->u8{3}}fn main(){check(<S<[u8]> as Flag>::n()==3);check(<S<[u8;1]> as Flag>::n()==1);}",
                false,
            ),
        ],
    )
}
