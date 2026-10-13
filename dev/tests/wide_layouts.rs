use rustgm::{
    abi::layout::{Layouts, Shape},
    types::{Kind, Primitive, RegionId, Store, TypeId, aggregate::Definitions},
};
use std::{path::Path, process::Command};

#[test]
fn wide_pointer_layouts_match_the_pinned_target_and_keep_unsized_referents()
-> Result<(), Box<dyn std::error::Error>> {
    let mut types = Store::default();
    let definitions = Definitions::default();
    let mut layouts = Layouts::default();
    let slice = types.intern(Kind::Slice(TypeId::primitive(Primitive::U8)))?;
    let text = TypeId::primitive(Primitive::Str);
    for referent in [slice, text] {
        for mutable in [false, true] {
            let reference = types.intern(Kind::Reference {
                region: RegionId::STATIC,
                mutable,
                referent,
            })?;
            let pointer = types.intern(Kind::Pointer {
                mutable,
                pointee: referent,
            })?;
            for (ty, contains) in [(reference, true), (pointer, false)] {
                let id = layouts.demand(&types, &definitions, ty)?;
                let layout = layouts.at(id).ok_or("wide layout")?;
                assert_eq!(
                    (layout.size, layout.align, layout.contains_reference),
                    (16, 8, contains)
                );
                assert!(matches!(layout.shape, Shape::WidePointer));
            }
            for (length, expected) in [(0, 0), (3, 48)] {
                let ty = types.intern(Kind::Array {
                    element: reference,
                    length,
                })?;
                let id = layouts.demand(&types, &definitions, ty)?;
                let layout = layouts.at(id).ok_or("array layout")?;
                assert_eq!((layout.size, layout.align), (expected, 8));
                assert!(layout.contains_reference);
            }
        }
        assert!(layouts.demand(&types, &definitions, referent).is_err());
    }
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/wide-layouts-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let source = folder.join("probe.rs");
    std::fs::write(
        &source,
        "use std::mem::{size_of,align_of};fn check<T>(n:usize){assert_eq!(size_of::<T>(),n);assert_eq!(align_of::<T>(),8);}fn main(){check::<&[u8]>(16);check::<&mut [u8]>(16);check::<&str>(16);check::<&mut str>(16);check::<*const [u8]>(16);check::<*mut [u8]>(16);check::<*const str>(16);check::<*mut str>(16);check::<[&str;0]>(0);check::<[&[u8];3]>(48);}",
    )?;
    let binary = folder.join("rustc");
    let compiled = Command::new("rustc")
        .args(["+1.98.0", "--edition=2024"])
        .arg(source)
        .arg("-o")
        .arg(&binary)
        .output()?;
    assert!(
        compiled.status.success(),
        "{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let executed = Command::new(binary).output()?;
    assert!(executed.status.success() && executed.stdout.is_empty() && executed.stderr.is_empty());
    Ok(())
}
