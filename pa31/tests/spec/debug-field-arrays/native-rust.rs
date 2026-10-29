include!("record-5.rs");
include!("record-6.rs");
include!("record-7.rs");
include!("tuple-5.rs");
include!("tuple-6.rs");
include!("tuple-7.rs");
include!("enum-5.rs");
include!("enum-6.rs");
include!("enum-7.rs");
include!("unsized-tail.rs");
include!("labels.rs");
fn main() {
    println!("{:?}", R7 { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7 });
    println!("{:?}", T7(1,2,3,4,5,6,7));
    println!("{:?}", E7::Record { a: 1, b: 2, c: 3, d: 4, e: 5, f: 6, g: 7 });
    println!("{:?}", E7::Tuple(1,2,3,4,5,6,7));
    let tail: &Tail<[u8]> = &Tail { a: 1,b: 2,c: 3,d: 4,e: 5,f: 6,tail: [7,8] };
    println!("{:?}", tail);
    println!("{:?}", Labels { r#type: 0, café: 1, δ: 2, fourth: 3, fifth: 4, sixth: 5 });
}
