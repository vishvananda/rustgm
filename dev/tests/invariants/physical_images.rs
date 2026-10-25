//! Immutable input sharing retains owned incarnation identity without byte copies.
use super::*;
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn physical_copies_share_bytes_but_own_fresh_retained_source_incarnations() -> Result {
    let input = b"fn source(){let x=7;}".to_vec();
    let buffer = input.as_ptr();
    let source = SourceFile::new(input)?;
    assert_eq!(source.original().as_ptr(), buffer);
    let retained = Arc::clone(source.original_image());
    let copy = source.physical_copy();
    assert!(!Arc::ptr_eq(source.original_image(), copy.original_image()));
    assert!(Arc::ptr_eq(&source.original.bytes, &copy.original.bytes));
    assert_eq!(source.original().as_ptr(), copy.original().as_ptr());
    assert_eq!(Arc::strong_count(&source.original.bytes), 2);
    assert_eq!(Arc::strong_count(source.original_image()), 2);
    drop(source);
    assert_eq!(Arc::strong_count(&retained), 1);
    let replacement = SourceFile::new(copy.original().as_bytes().to_vec())?;
    assert!(!Arc::ptr_eq(&retained, replacement.original_image()));
    assert!(!Arc::ptr_eq(&retained, copy.original_image()));
    assert_eq!(replacement.original(), copy.original());
    println!(
        "source image Arc allocations: constructor2 physical_copy1; original byte buffers: constructor reuses input, physical_copy0"
    );
    Ok(())
}
#[test]
fn shared_originals_preserve_crlf_bom_positions_and_independent_generated_views() -> Result {
    let text = "\u{feff}a\r\né\r\n";
    let mut source = SourceFile::new(text.as_bytes().to_vec())?;
    assert_eq!(source.original(), text);
    assert_eq!(source.text(), "\u{feff}a\né\n");
    let end = u32::try_from(source.physical_len())?;
    let before = source.position(end)?;
    let original = Arc::clone(source.original_image());
    source.append("generated\r\ntext")?;
    assert!(Arc::ptr_eq(&original, source.original_image()));
    assert_eq!(source.position(end)?, before);
    assert!(source.position(end + 1).is_err());
    let mut copy = source.physical_copy();
    assert_eq!(copy.text(), "\u{feff}a\né\n");
    assert_eq!(copy.original(), text);
    assert_eq!(copy.position(end)?, before);
    copy.append("different")?;
    assert!(!source.text().contains("different"));
    assert!(!copy.text().contains("generated"));
    assert!(Arc::ptr_eq(&source.original.bytes, &copy.original.bytes));
    Ok(())
}
