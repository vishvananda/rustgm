//! Unsized prefixes share fixed child layouts without acquiring a whole-object size.
use rustgm::{
    abi::{layout::Layouts, tails::Queries},
    types::{
        GenericArgs, ItemId, Kind, Primitive, Store, TypeId,
        aggregate::{self, Definitions, Repr, Representation, Variant},
    },
};

const U8: TypeId = TypeId::primitive(Primitive::U8);
const U16: TypeId = TypeId::primitive(Primitive::U16);
const U32: TypeId = TypeId::primitive(Primitive::U32);

#[derive(Default)]
struct Fixture {
    types: Store,
    definitions: Definitions,
    layouts: Layouts,
    tails: Queries,
    next: u32,
}
impl Fixture {
    fn record(&mut self, repr: Repr, fields: &[TypeId]) -> Result<TypeId, &'static str> {
        let item = ItemId(self.next);
        self.next += 1;
        let fields = self.types.intern_list(fields)?;
        self.definitions.insert(
            item,
            aggregate::Kind::Struct,
            repr,
            &[Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        self.types.intern(Kind::Adt(item, GenericArgs::EMPTY))
    }
    fn query(&mut self, ty: TypeId) -> Result<(u32, Vec<u64>), &'static str> {
        let id = self
            .tails
            .demand(&self.types, &self.definitions, &mut self.layouts, ty)?;
        let layout = self.tails.at(id).ok_or("missing tail layout")?;
        let offsets = layout
            .fields
            .map_or_else(Vec::new, |fields| self.tails.offsets(fields).to_vec());
        Ok((layout.align, offsets))
    }
}

#[test]
fn prefixes_keep_tail_alignment_and_nested_offsets() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let slice = f.types.intern(Kind::Slice(U32))?;
    let c = Repr {
        representation: Representation::C,
        ..Repr::default()
    };
    let inner = f.record(c, &[U16, slice])?;
    let outer = f.record(c, &[U8, inner])?;
    assert_eq!(f.query(inner)?, (4, vec![0, 4]));
    assert_eq!(f.query(outer)?, (4, vec![0, 4]));
    assert_eq!(f.query(slice)?, (4, vec![]));
    assert!(f.layouts.get(inner).is_none());
    assert!(f.layouts.get(outer).is_none());
    assert!(f.layouts.demand(&f.types, &f.definitions, inner).is_err());
    // A failed whole-value layout cannot poison the independent place query.
    assert_eq!(f.query(outer)?, (4, vec![0, 4]));
    Ok(())
}

#[test]
fn representations_preserve_packing_transparency_and_explicit_alignment() -> Result<(), &'static str>
{
    let mut f = Fixture::default();
    let bytes = f.types.intern(Kind::Slice(U8))?;
    let packed = f.record(
        Repr {
            packed: Some(1),
            ..Repr::default()
        },
        &[U16, bytes],
    )?;
    assert_eq!(f.query(packed)?, (1, vec![0, 2]));
    let transparent = Repr {
        representation: Representation::Transparent,
        ..Repr::default()
    };
    let wrapper = f.record(transparent, &[TypeId::UNIT, bytes])?;
    assert_eq!(f.query(wrapper)?, (1, vec![0, 0]));
    let bad = f.record(transparent, &[U8, bytes])?;
    assert!(f.query(bad).is_err());
    let aligned = f.record(
        Repr {
            align: Some(64),
            ..Repr::default()
        },
        &[U8, bytes],
    )?;
    assert_eq!(f.query(aligned)?, (64, vec![0, 1]));
    let bad = f.record(
        Repr {
            packed: Some(2),
            ..Repr::default()
        },
        &[U8, aligned],
    )?;
    assert!(f.query(bad).is_err());
    let zst = f.record(
        Repr {
            align: Some(64),
            ..Repr::default()
        },
        &[],
    )?;
    let slice = f.types.intern(Kind::Slice(zst))?;
    let root = f.record(Repr::default(), &[U8, slice])?;
    assert_eq!(f.query(root)?, (64, vec![0, 64]));
    Ok(())
}

#[test]
fn checked_packing_keeps_slice_element_alignment_separate_from_containment()
-> Result<(), &'static str> {
    let mut f = Fixture::default();
    let element = f.record(
        Repr {
            align: Some(64),
            ..Repr::default()
        },
        &[U8],
    )?;
    let slice = f.types.intern(Kind::Slice(element))?;
    let root = f.record(
        Repr {
            packed: Some(1),
            ..Repr::default()
        },
        &[U8, slice],
    )?;
    f.definitions
        .validate_packing(&f.types)
        .map_err(|(_, message)| message)?;
    assert_eq!(f.query(root)?, (1, vec![0, 1]));
    assert_eq!(f.query(slice)?, (64, vec![]));
    Ok(())
}

#[test]
fn shared_deep_tails_and_failures_reuse_completed_work() -> Result<(), &'static str> {
    let mut f = Fixture::default();
    let mut root = f.types.intern(Kind::Slice(U16))?;
    for _ in 0..4096 {
        root = f.record(Repr::default(), &[U8, root])?;
    }
    assert_eq!(f.query(root)?, (2, vec![0, 2]));
    assert!(f.tails.steps < 4096 * 4);
    let steps = f.tails.steps;
    for _ in 0..32 {
        assert_eq!(f.query(root)?, (2, vec![0, 2]));
    }
    assert_eq!(f.tails.steps, steps);
    let next = f
        .types
        .intern(Kind::Adt(ItemId(f.next), GenericArgs::EMPTY))?;
    let recursive = f.record(Repr::default(), &[next])?;
    assert!(f.query(recursive).is_err());
    let steps = f.tails.steps;
    assert!(f.query(recursive).is_err());
    assert_eq!(f.tails.steps, steps);
    assert!(f.query(U8).is_err());
    let steps = f.tails.steps;
    assert!(f.query(U8).is_err());
    assert_eq!(f.tails.steps, steps);
    assert_eq!(f.query(root)?, (2, vec![0, 2]));
    Ok(())
}

#[test]
fn referent_alignment_uses_tails_without_following_recursive_pointer_storage()
-> Result<(), &'static str> {
    let mut f = Fixture::default();
    let root = f.types.intern(Kind::Adt(ItemId(0), GenericArgs::EMPTY))?;
    let pointer = f.types.intern(Kind::Pointer {
        pointee: root,
        mutable: false,
    })?;
    let slice = f.types.intern(Kind::Slice(U32))?;
    assert_eq!(f.record(Repr::default(), &[pointer, U8, slice])?, root);
    let mut rules = rustgm::types::sizedness::Rules::default();
    assert_eq!(
        rules.prepare(&mut f.types, &f.definitions, ItemId(0), |_, _, _| Ok(false))?,
        None
    );
    f.types.seal_nominal_sizes(std::sync::Arc::new(rules))?;
    assert_eq!(f.query(root)?, (8, vec![0, 16, 20]));
    assert_eq!(
        f.layouts
            .reference_align(&f.types, &f.definitions, root)
            .map_err(|e| e.message)?,
        8
    );
    assert!(f.layouts.get(root).is_none());
    assert_eq!(
        f.layouts
            .unsized_field(&f.types, &f.definitions, root, 0)
            .map_err(|e| e.message)?,
        (pointer, 0)
    );
    assert_eq!(
        f.layouts
            .unsized_field(&f.types, &f.definitions, root, 2)
            .map_err(|e| e.message)?,
        (slice, 20)
    );
    assert!(
        f.layouts
            .unsized_field(&f.types, &f.definitions, root, 3)
            .is_err()
    );
    Ok(())
}

#[test]
fn concrete_prefix_offsets_match_the_pinned_rustc_abi() -> Result<(), Box<dyn std::error::Error>> {
    use std::{path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/pa22-native/tail-layout-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let source = folder.join("oracle.rs");
    let binary = folder.join("oracle");
    std::fs::write(
        &source,
        r"
        #[repr(C)] struct S<T:?Sized>{prefix:u16,tail:T}
        #[repr(C,packed)] struct P{prefix:u16,tail:[u8;2]}
        #[repr(C,align(64))] struct A<T:?Sized>{prefix:u8,tail:T}
        fn main() {
            let s=S{prefix:1,tail:[2_u32,3]}; let s:&S<[u32]>=&s;
            let base=s as *const S<[u32]> as *const u8 as usize;
            assert_eq!(std::ptr::addr_of!(s.tail) as *const u32 as usize-base,4);
            assert_eq!(std::mem::align_of_val(s),4);
            let p=&P{prefix:1,tail:[2_u8,3]};
            let base=p as *const P as *const u8 as usize;
            assert_eq!(std::ptr::addr_of!(p.tail) as *const u8 as usize-base,2);
            assert_eq!(std::mem::align_of_val(p),1);
            let a=A{prefix:1,tail:[2_u8,3]}; let a:&A<[u8]>=&a;
            let base=a as *const A<[u8]> as *const u8 as usize;
            assert_eq!(std::ptr::addr_of!(a.tail) as *const u8 as usize-base,1);
            assert_eq!(std::mem::align_of_val(a),64);
        }
    ",
    )?;
    let output = Command::new("rustc")
        .args(["--edition=2024", "--cap-lints=allow"])
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()?;
    std::fs::write(folder.join("rustc.stderr"), &output.stderr)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = Command::new(&binary).output()?;
    std::fs::write(folder.join("run.stderr"), &output.stderr)?;
    assert!(output.status.success() && output.stdout.is_empty() && output.stderr.is_empty());
    std::fs::remove_file(binary)?;
    Ok(())
}
