// tinyvec, the small-vector crate unicode-normalization buffers through.
use tinyvec::{ArrayVec, SliceVec, TinyVec, array_vec, tiny_vec};

fn describe<A: tinyvec::Array>(vector: &TinyVec<A>) -> &'static str {
    if vector.is_inline() { "inline" } else { "heap" }
}

fn main() {
    let mut array: ArrayVec<[u32; 6]> = ArrayVec::new();
    println!("new {} {} {:?}", array.len(), array.capacity(), array.pop());
    array.push(3);
    array.push(1);
    array.extend([4, 1, 5]);
    println!("push {:?} {}", array, array.len());
    println!("try_push {:?} {:?} {:?}", array.try_push(9), array.try_push(2), array);
    println!("full {} {:?} {:?}", array.len() == array.capacity(), array.pop(), array.as_slice());
    array.insert(1, 100);
    println!("insert {:?}", array);
    println!("remove {} {} {:?}", array.remove(0), array.swap_remove(0), array);
    array.sort_unstable();
    array.retain(|value| *value != 1);
    println!("retain {:?} {:?} {:?}", array, array.iter().sum::<u32>(), array.binary_search(&100));
    let drained: ArrayVec<[u32; 6]> = array.drain(1..).collect();
    println!("drain {:?} {:?}", array, drained);
    array.truncate(0);
    array.resize(3, 7);
    let mut rest = array.fill(8..);
    println!("fill {:?} {:?} {:?} {:?}", array, rest.next(), array.clone().into_inner(), array_vec!([u8; 4] => 1, 2).len());
    let from_array = ArrayVec::from([1u8, 2, 3]);
    let prefix = ArrayVec::from_array_len([9u8, 8, 7, 6], 2);
    println!("from {:?} {:?} {:?} {}", from_array, prefix, ArrayVec::<[u8; 2]>::try_from_array_len([1, 2], 3).is_err(), from_array.as_slice() == prefix.as_slice());

    let mut tiny: TinyVec<[u16; 4]> = TinyVec::new();
    for value in 1..=4 {
        tiny.push(value * 10);
    }
    println!("tiny {:?} {} {} {}", tiny, describe(&tiny), tiny.len(), tiny.capacity());
    tiny.push(50);
    println!("spilled {:?} {} {} {}", tiny, describe(&tiny), tiny.len(), tiny.capacity() >= 5);
    tiny.insert(2, 25);
    println!("edit {:?} {:?} {} {:?}", tiny.pop(), tiny.remove(0), tiny.swap_remove(1), tiny);
    tiny.extend_from_slice(&[1, 2, 3]);
    tiny.sort_unstable_by(|a, b| b.cmp(a));
    tiny.retain(|value| *value != 2);
    println!("sorted {:?} {:?} {:?}", tiny, tiny.first(), tiny.iter().rev().take(2).collect::<Vec<_>>());
    tiny.truncate(2);
    tiny.shrink_to_fit();
    println!("truncate {:?} {}", tiny, tiny.len());
    let mut inline = tiny_vec!([char; 8] => 'a', 'b', 'c');
    inline.extend("def".chars());
    let text: String = inline.iter().collect();
    println!("inline {} {} {:?}", text, describe(&inline), inline.splice(1..4, ['X']).collect::<String>());
    inline.move_to_the_heap();
    println!("moved {:?} {} {:?}", inline, describe(&inline), inline.clone().into_iter().rev().collect::<String>());
    let mut collected: TinyVec<[u64; 3]> = (1..=10u64).map(|n| n * n).collect();
    collected.reserve(100);
    println!("collected {} {} {} {:?}", collected.len(), describe(&collected), collected.iter().sum::<u64>(), &collected[7..]);
    collected.clear();
    println!("cleared {} {}", collected.is_empty(), describe(&collected));
    let nested: TinyVec<[TinyVec<[u8; 2]>; 2]> = (0..3u8).map(|n| (0..n).collect()).collect();
    println!("nested {:?} {:?}", nested, nested.iter().map(|inner| describe(inner)).collect::<Vec<_>>());

    let mut backing = [0i32; 5];
    let mut slice = SliceVec::from_slice_len(&mut backing, 0);
    slice.push(5);
    slice.push(-5);
    slice.extend([1, 2]);
    println!("slice vec {:?} {} {}", slice, slice.len(), slice.capacity());
    println!("slice vec {:?} {:?}", slice.pop(), slice);
    slice.insert(0, 42);
    slice.push(77);
    println!("slice vec {:?} {}", slice, slice.len() == slice.capacity());
    println!("backing {:?}", backing);
    let mut combining: TinyVec<[(u8, char); 4]> = TinyVec::new();
    for (class, mark) in [(230, '\u{301}'), (220, '\u{323}'), (230, '\u{300}'), (1, '\u{334}'), (220, '\u{325}')] {
        combining.push((class, mark));
    }
    combining.sort_by_key(|(class, _)| *class);
    println!("combining {:?} {}", combining.iter().map(|(class, mark)| format!("{}:{:04X}", class, *mark as u32)).collect::<Vec<_>>(), describe(&combining));
}
