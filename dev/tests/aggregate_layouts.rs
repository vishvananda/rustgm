use rustgm::{
    abi::layout::{Layouts, Shape},
    types::{
        ItemId, Kind, Primitive, Store, TypeId,
        aggregate::{self, Definitions, Repr, Representation, Variant},
    },
};
use std::{fmt::Write, fs, process::Command};
const U8: TypeId = TypeId::primitive(Primitive::U8);
const U16: TypeId = TypeId::primitive(Primitive::U16);
const U64: TypeId = TypeId::primitive(Primitive::U64);
const U128: TypeId = TypeId::primitive(Primitive::U128);
#[derive(Default)]
struct World {
    types: Store,
    definitions: Definitions,
}
impl World {
    fn adt(
        &mut self,
        id: u32,
        kind: aggregate::Kind,
        repr: Repr,
        fields: &[&[TypeId]],
        discriminants: &[u128],
    ) -> Result<TypeId, &'static str> {
        let mut variants = Vec::new();
        for (index, fields) in fields.iter().enumerate() {
            variants.push(Variant {
                fields: self.types.intern_list(fields)?,
                discriminant: discriminants.get(index).copied().unwrap_or(0),
            });
        }
        self.definitions.insert(ItemId(id), kind, repr, &variants)?;
        self.types.intern(Kind::Adt(ItemId(id)))
    }
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one shared rustc executable independently checks the complete labeled representation vector"
)]
fn guaranteed_representations_agree_with_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::default();
    let c = Repr {
        representation: Representation::C,
        ..Repr::default()
    };
    let record = world.adt(0, aggregate::Kind::Struct, c, &[&[U8, U64, U16]], &[])?;
    let packed = world.adt(
        1,
        aggregate::Kind::Struct,
        Repr {
            packed: Some(2),
            ..c
        },
        &[&[U8, U64, U8]],
        &[],
    )?;
    let zst = world.adt(
        2,
        aggregate::Kind::Struct,
        Repr {
            align: Some(32),
            ..Repr::default()
        },
        &[&[]],
        &[],
    )?;
    let aligned = world.adt(3, aggregate::Kind::Struct, c, &[&[U8, zst, U8]], &[])?;
    let transparent = world.adt(
        4,
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[&[TypeId::UNIT, U64, TypeId::UNIT]],
        &[],
    )?;
    let c_enum = world.adt(5, aggregate::Kind::Enum, c, &[&[U8], &[U128]], &[0, 1])?;
    let int_enum = world.adt(
        6,
        aggregate::Kind::Enum,
        Repr {
            tag: Some(Primitive::U8),
            ..Repr::default()
        },
        &[&[U8], &[U128]],
        &[0, 1],
    )?;
    let c_int_enum = world.adt(
        7,
        aggregate::Kind::Enum,
        Repr {
            tag: Some(Primitive::U8),
            ..c
        },
        &[&[U8], &[U128]],
        &[0, 1],
    )?;
    let large_enum = world.adt(8, aggregate::Kind::Enum, c, &[&[]], &[1 << 40])?;
    let negative_enum = world.adt(
        9,
        aggregate::Kind::Enum,
        c,
        &[&[], &[]],
        &[u128::from(u64::MAX), 0],
    )?;
    let array = world.types.intern(Kind::Array {
        element: record,
        length: 3,
    })?;
    let mut layouts = Layouts::default();
    let mut expected = String::new();
    for (name, ty) in [
        ("C", record),
        ("P", packed),
        ("Z", zst),
        ("A", aligned),
        ("T", transparent),
        ("CE", c_enum),
        ("IE", int_enum),
        ("CIE", c_int_enum),
        ("L", large_enum),
        ("N", negative_enum),
        ("Array", array),
        ("u128", U128),
    ] {
        let id = layouts.demand(&world.types, &world.definitions, ty)?;
        let layout = layouts.at(id).ok_or("missing demanded layout")?;
        writeln!(expected, "{name} {} {}", layout.size, layout.align)?;
    }
    for (name, ty) in [("C", record), ("P", packed), ("A", aligned)] {
        let layout = layouts.get(ty).ok_or("missing record layout")?;
        write!(expected, "{name}")?;
        for offset in layouts.offsets(&layouts.variants(layout)[0]) {
            write!(expected, " {offset}")?;
        }
        writeln!(expected)?;
    }
    for (name, ty) in [("CE", c_enum), ("IE", int_enum), ("CIE", c_int_enum)] {
        write!(expected, "{name}")?;
        for variant in layouts.variants(layouts.get(ty).ok_or("missing enum layout")?) {
            write!(expected, " {}", layouts.offsets(variant)[0])?;
        }
        writeln!(expected)?;
    }
    let source = r#"
#![allow(dead_code)]
use std::mem::{size_of,align_of,offset_of};
#[repr(C)]struct C{a:u8,b:u64,c:u16}
#[repr(C,packed(2))]struct P{a:u8,b:u64,c:u8}
#[repr(align(32))]struct Z;
#[repr(C)]struct A{a:u8,z:Z,b:u8}
#[repr(transparent)]struct T((),u64,());
#[repr(C)]enum CE{A(u8),B(u128)}
#[repr(u8)]enum IE{A(u8),B(u128)}
#[repr(C,u8)]enum CIE{A(u8),B(u128)}
#[repr(C)]enum L{A=1<<40}
#[repr(C)]enum N{A=-1,B=0}
fn main(){
 macro_rules! sizes {($name:expr,$ty:ty)=>{println!("{} {} {}",$name,size_of::<$ty>(),align_of::<$ty>());}}
 sizes!("C",C);sizes!("P",P);sizes!("Z",Z);sizes!("A",A);sizes!("T",T);
 sizes!("CE",CE);sizes!("IE",IE);sizes!("CIE",CIE);sizes!("L",L);sizes!("N",N);sizes!("Array",[C;3]);sizes!("u128",u128);
 println!("C {} {} {}",offset_of!(C,a),offset_of!(C,b),offset_of!(C,c));
 println!("P {} {} {}",offset_of!(P,a),offset_of!(P,b),offset_of!(P,c));
 println!("A {} {} {}",offset_of!(A,a),offset_of!(A,z),offset_of!(A,b));
 macro_rules! offsets {($ty:ident)=>{{
   let a=$ty::A(1);let b=$ty::B(2);
   let x=match &a{$ty::A(x)=>x as *const _ as usize,_=>0};
   let y=match &b{$ty::B(y)=>y as *const _ as usize,_=>0};
   println!("{} {} {}",stringify!($ty),x-(&a as *const _ as usize),y-(&b as *const _ as usize));
 }}}
 offsets!(CE);offsets!(IE);offsets!(CIE);
}

"#;
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../obj/layout-tests");
    fs::create_dir_all(&directory)?;
    let input = directory.join("guaranteed.rs");
    let binary = directory.join("guaranteed");
    fs::write(&input, source)?;
    let compiled = Command::new("rustc")
        .args(["--edition=2024", "--target", "x86_64-unknown-linux-gnu"])
        .arg(&input)
        .arg("-o")
        .arg(&binary)
        .output()?;
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let output = Command::new(binary).output()?;
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert_eq!(String::from_utf8(output.stdout)?, expected);
    Ok(())
}

#[test]
fn recursive_identity_does_not_imply_recursive_storage() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::default();
    let node = world.types.intern(Kind::Adt(ItemId(0)))?;
    let pointer = world.types.intern(Kind::Pointer {
        mutable: false,
        pointee: node,
    })?;
    world.adt(
        0,
        aggregate::Kind::Struct,
        Repr::default(),
        &[&[pointer]],
        &[],
    )?;
    let bad = world.types.intern(Kind::Adt(ItemId(1)))?;
    world.adt(1, aggregate::Kind::Struct, Repr::default(), &[&[bad]], &[])?;
    let mut layouts = Layouts::default();
    layouts.demand(&world.types, &world.definitions, node)?;
    assert_eq!(
        layouts
            .get(node)
            .ok_or("missing recursive node layout")?
            .size,
        8
    );
    let error = layouts
        .demand(&world.types, &world.definitions, bad)
        .err()
        .ok_or("accepted infinite type")?;
    assert_eq!(error.ty, bad);
    assert!(error.message.contains("infinite size"));
    let steps = layouts.steps;
    assert_eq!(
        layouts.demand(&world.types, &world.definitions, bad).err(),
        Some(error)
    );
    assert_eq!(layouts.steps, steps);
    assert_eq!(layouts.get(node).ok_or("lost completed layout")?.size, 8);
    Ok(())
}

#[test]
fn huge_zero_sized_arrays_and_uninhabited_values_are_distinct()
-> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::default();
    let empty = world.adt(0, aggregate::Kind::Enum, Repr::default(), &[], &[])?;
    let empty_array = world.types.intern(Kind::Array {
        element: empty,
        length: 0,
    })?;
    let impossible_array = world.types.intern(Kind::Array {
        element: empty,
        length: 1,
    })?;
    let huge = world.types.intern(Kind::Array {
        element: TypeId::UNIT,
        length: u64::MAX,
    })?;
    let oversized = world.types.intern(Kind::Array {
        element: U128,
        length: u64::MAX,
    })?;
    let mut layouts = Layouts::default();
    for ty in [empty_array, impossible_array, huge] {
        layouts.demand(&world.types, &world.definitions, ty)?;
    }
    assert!(
        layouts
            .get(empty_array)
            .ok_or("missing empty array")?
            .inhabited
    );
    assert!(
        !layouts
            .get(impossible_array)
            .ok_or("missing impossible array")?
            .inhabited
    );
    let huge_layout = layouts.get(huge).ok_or("missing ZST array")?;
    assert_eq!(huge_layout.size, 0);
    assert!(matches!(
        huge_layout.shape,
        Shape::Array {
            length: u64::MAX,
            stride: 0,
            ..
        }
    ));
    assert!(layouts.steps < 20);
    assert!(
        layouts
            .demand(&world.types, &world.definitions, oversized)
            .is_err()
    );
    Ok(())
}

#[test]
fn repr_errors_do_not_corrupt_completed_dependencies() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::default();
    let aligned = world.adt(
        0,
        aggregate::Kind::Struct,
        Repr {
            align: Some(16),
            ..Repr::default()
        },
        &[&[U8]],
        &[],
    )?;
    let packed = world.adt(
        1,
        aggregate::Kind::Struct,
        Repr {
            packed: Some(1),
            ..Repr::default()
        },
        &[&[aligned]],
        &[],
    )?;
    let transparent = world.adt(
        2,
        aggregate::Kind::Struct,
        Repr {
            representation: Representation::Transparent,
            ..Repr::default()
        },
        &[&[U8, U64]],
        &[],
    )?;
    let invalid = world.adt(
        3,
        aggregate::Kind::Struct,
        Repr {
            align: Some(3),
            ..Repr::default()
        },
        &[&[]],
        &[],
    )?;
    let mut layouts = Layouts::default();
    for ty in [packed, transparent, invalid] {
        assert!(
            layouts
                .demand(&world.types, &world.definitions, ty)
                .is_err()
        );
    }
    let id = layouts.demand(&world.types, &world.definitions, aligned)?;
    let layout = layouts.at(id).ok_or("lost valid aligned dependency")?;
    assert_eq!((layout.size, layout.align), (16, 16));
    assert_eq!(layouts.offsets(&layouts.variants(layout)[0]), [0]);
    Ok(())
}

#[test]
fn deeply_nested_layout_is_iterative_and_memoized() -> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let mut ty = U8;
    for _ in 0..20_000 {
        let fields = types.intern_list(&[ty])?;
        ty = types.intern(Kind::Tuple(fields))?;
    }
    let definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let id = layouts.demand(&types, &definitions, ty)?;
    assert_eq!(layouts.at(id).ok_or("missing deep layout")?.size, 1);
    assert_eq!(layouts.len(), 20_001);
    assert_eq!(layouts.steps, 40_002);
    let steps = layouts.steps;
    assert_eq!(layouts.demand(&types, &definitions, ty)?, id);
    assert_eq!(layouts.steps, steps);
    Ok(())
}

#[test]
fn nominal_identity_remains_distinct_during_inference() -> Result<(), Box<dyn std::error::Error>> {
    use rustgm::types::infer::{Constructor, Context, Error};
    let mut world = World::default();
    let a = world.adt(0, aggregate::Kind::Struct, Repr::default(), &[&[U8]], &[])?;
    let b = world.adt(1, aggregate::Kind::Struct, Repr::default(), &[&[U8]], &[])?;
    let mut inference = Context::default();
    let left = inference.known(&world.types, a)?;
    let same = inference.construct(Constructor::Adt(ItemId(0)), &[])?;
    inference.equate(&world.types, left, same)?;
    let right = inference.known(&world.types, b)?;
    assert_eq!(
        inference.equate(&world.types, left, right),
        Err(Error::Mismatch)
    );
    Ok(())
}
