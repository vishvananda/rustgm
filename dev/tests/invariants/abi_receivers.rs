//! Layout coincidence cannot replace the actual slice metadata/representation.
use super::*;
use crate::{abi::layout::Layout, lowir::program::MemorySpan};

#[test]
fn source_slice_reference_requires_length_and_actual_wide_layout() -> Result<(), &'static str> {
    let layout = Layout {
        size: 16,
        align: 8,
        inhabited: true,
        contains_explicit_align: false,
        contains_reference: true,
        shape: Shape::WidePointer,
    };
    let object = Representation::Object(MemorySpan {
        bytes: 16,
        align: 8,
    });
    wide_reference(Metadata::Length, Metadata::Length, layout, object)?;
    for metadata in [
        Metadata::Thin,
        Metadata::VTable(TypeId::UNIT),
        Metadata::Unknown(TypeId::UNIT),
    ] {
        assert!(wide_reference(Metadata::Length, metadata, layout, object).is_err());
    }
    for shape in [
        Shape::Scalar,
        Shape::Array {
            element: TypeId::UNIT,
            stride: 1,
            length: 16,
        },
    ] {
        assert!(
            wide_reference(
                Metadata::Length,
                Metadata::Length,
                Layout { shape, ..layout },
                object
            )
            .is_err()
        );
    }
    for span in [
        MemorySpan { bytes: 8, align: 8 },
        MemorySpan {
            bytes: 16,
            align: 4,
        },
    ] {
        assert!(
            wide_reference(
                Metadata::Length,
                Metadata::Length,
                layout,
                Representation::Object(span)
            )
            .is_err()
        );
    }
    assert!(
        wide_reference(
            Metadata::Length,
            Metadata::Length,
            layout,
            Representation::Scalar(Type::Ptr)
        )
        .is_err()
    );
    assert!(
        wide_reference(
            Metadata::Length,
            Metadata::Length,
            Layout { align: 4, ..layout },
            object
        )
        .is_err()
    );
    println!(
        "Input={} State={} Context={} Body={}",
        size_of::<Input>(),
        size_of::<super::super::State<'_>>(),
        size_of::<Context<'_>>(),
        size_of::<crate::body::ir::Body>()
    );
    Ok(())
}

#[test]
fn source_dynamic_reference_requires_exact_typed_vtable_metadata() -> Result<(), &'static str> {
    let layout = Layout {
        size: 16,
        align: 8,
        inhabited: true,
        contains_explicit_align: false,
        contains_reference: true,
        shape: Shape::WidePointer,
    };
    let object = Representation::Object(MemorySpan {
        bytes: 16,
        align: 8,
    });
    let expected = Metadata::VTable(TypeId(101));
    wide_reference(expected, expected, layout, object)?;
    for found in [
        Metadata::Length,
        Metadata::Thin,
        Metadata::VTable(TypeId(102)),
        Metadata::Unknown(TypeId(101)),
    ] {
        assert!(wide_reference(expected, found, layout, object).is_err());
    }
    assert!(
        wide_reference(
            expected,
            expected,
            layout,
            Representation::Scalar(Type::Ptr)
        )
        .is_err()
    );
    assert!(wide_reference(expected, expected, Layout { size: 8, ..layout }, object).is_err());
    assert!(wide_reference(Metadata::Thin, Metadata::Thin, layout, object).is_err());
    Ok(())
}
