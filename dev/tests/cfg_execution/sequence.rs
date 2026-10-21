//! Reading one immutable element must not expand the repeated backing array.
use super::{Limits, execute, rustc};

#[test]
fn immutable_indexing_retains_compressed_storage_after_other_fields_change()
-> Result<(), Box<dyn std::error::Error>> {
    for family in ["array", "slice", "mixed"] {
        let mut baseline = None;
        for width in [8, 1024, 1_000_000] {
            let last = width - 1;
            let source = match family {
                "array" => format!("const fn run()->u32{{let a=[7u32;{width}];a[{last}]}}"),
                "slice" => format!(
                    "const fn read(a:&[u32],i:usize)->u32{{a[i]}}\
                     const fn run()->u32{{let a=[7u32;{width}];read(&a,{last})}}"
                ),
                _ => format!(
                    "const fn run()->u32{{let mut a=([3u32;{width}],1u32);\
                     a.1=4;a.0[{last}]+a.1}}"
                ),
            };
            rustc::qualify(
                &format!("compressed-read-{family}-{width}"),
                &format!("{source}const _: [();7]=[();run() as usize];"),
                true,
            )?;
            let (value, work) = execute(
                &source,
                Limits {
                    steps: 2048,
                    slots: 128,
                    ..Limits::default()
                },
            )?;
            assert_eq!(value, 7);
            let measured = (work.steps, work.peak_slots);
            if let Some(previous) = baseline {
                assert_eq!(measured, previous, "{family}/{width}");
            }
            baseline = Some(measured);
            println!(
                "{family}/{width}: steps={} slots={}",
                work.steps, work.peak_slots
            );
        }
    }
    Ok(())
}
