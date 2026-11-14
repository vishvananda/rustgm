//! Exact constant products and divisions are observed through an integer-only ABI.
use crate::{provider_traits, resolve};
use rustgm::lowir::{data::Item, program::Program};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKETS: [(&str, &str); 4] = [
    ("wide-constant-division", "pa25"),
    ("wide-constant-division", "pa31"),
    ("wide-constant-products", "pa25"),
    ("wide-constant-products", "pa31"),
];

fn prepare(input: &str, source: bool) -> Result<Program> {
    if source {
        provider_traits::lower(input)
    } else {
        provider_traits::lower_report(&resolve::resolve(input, true)?)
    }
}
fn matrix(packet: &str, owner: &str, width: usize) -> Result<Program> {
    let input = resolve::fixture(&format!("{owner}/tests/spec/{packet}/f{width}.rs"))?;
    prepare(&input, owner == "pa31")
}
fn bytes(program: &Program) -> Result<Vec<u8>> {
    let global = program
        .globals
        .iter()
        .find(|global| global.name == "TABLE")
        .ok_or("missing actual TABLE storage")?;
    let range = global.items.ok_or("missing constant table initializer")?;
    let mut output = Vec::new();
    for item in &program.data[range.start as usize..range.end as usize] {
        match *item {
            Item::Zero(n) => output.resize(output.len() + usize::try_from(n)?, 0),
            Item::Bits { ty, bits } => {
                output.extend_from_slice(&bits.to_le_bytes()[..(ty.bits() / 8) as usize]);
            }
            Item::Address { .. } => return Err("float table contains a relocation".into()),
        }
    }
    assert_eq!(output.len(), usize::try_from(global.span.bytes)?);
    Ok(output)
}
#[test]
fn direct_and_source_const_calls_match_independent_interchange_bits() -> Result {
    for (packet, owner) in PACKETS {
        for width in [16, 128] {
            let program = matrix(packet, owner, width)
                .map_err(|error| format!("{packet} {owner} f{width}: {error}"))?;
            let data = bytes(&program)?;
            let expected = resolve::fixture(&format!("{owner}/tests/spec/{packet}/f{width}.bits"))?;
            assert_eq!(data.len(), expected.lines().count() * width / 8);
            for (index, (value, expected)) in data
                .chunks_exact(width / 8)
                .zip(expected.lines())
                .enumerate()
            {
                let mut full = [0; 16];
                full[..value.len()].copy_from_slice(value);
                let value = u128::from_le_bytes(full);
                if expected == "nan" {
                    let infinity = if width == 16 {
                        0x7c00
                    } else {
                        0x7fff_u128 << 112
                    };
                    let sign = 1_u128 << (width - 1);
                    assert!(
                        value & !sign > infinity,
                        "{owner} f{width} row{index}: {value:x}"
                    );
                } else {
                    assert_eq!(
                        value,
                        u128::from_str_radix(expected, 16)?,
                        "{owner} f{width} row{index}"
                    );
                }
            }
        }
    }
    Ok(())
}
#[test]
fn interchange_retains_feature_type_const_call_and_source_role_requirements() -> Result {
    for (packet, owner) in PACKETS {
        let prefix = format!("{owner}/tests/spec/{packet}");
        let table = resolve::fixture(&format!("{prefix}/cases.tsv"))?;
        let mut rejected = 0;
        for row in table.lines() {
            let columns: Vec<_> = row.split('\t').collect();
            let [path, status, reason] = columns.as_slice() else {
                return Err("invalid interchange case".into());
            };
            if *status == "0" {
                continue;
            }
            let input = resolve::fixture(path)?;
            let error = prepare(&input, owner == "pa31")
                .err()
                .ok_or_else(|| format!("{path}: accepted"))?;
            assert!(
                *reason == "-" || error.to_string().contains(reason),
                "{path}: {error}"
            );
            println!("{path}: {error}");
            rejected += 1;
        }
        assert_eq!(rejected, if owner == "pa25" { 4 } else { 2 });
    }
    Ok(())
}

#[test]
fn interchange_work_tracks_actual_initializers_and_output_bytes() -> Result {
    for packet in ["wide-constant-division", "wide-constant-products"] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let prefix = format!("pa25/tests/spec/{packet}/scaling-{width}");
            let input = resolve::fixture(&format!("{prefix}.rs"))?;
            let report = resolve::resolve(&input, true)?;
            let rustgm::pipeline::Analysis {
                mut facts,
                prepared,
            } = rustgm::pipeline::analyze_source(&report)?;
            let program = rustgm::lower::instances::build_roots_prepared(
                &report,
                &mut facts,
                rustgm::lower::Roots {
                    entry: None,
                    exports: &[],
                },
                prepared,
            )?;
            let data = bytes(&program)?;
            let expected = resolve::fixture(&format!("{prefix}.bits"))?;
            assert_eq!(data.len(), width * 16);
            assert_eq!(expected.lines().count(), width);
            for (value, expected) in data.as_chunks::<16>().0.iter().zip(expected.lines()) {
                let value = u128::from_le_bytes(*value);
                assert_eq!(value, u128::from_str_radix(expected, 16)?);
            }
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                data.len(),
            ];
            println!("{packet} width={width} work={work:?}");
            if let Some(old) = previous {
                for (before, after) in std::iter::zip(old, work) {
                    assert!(after <= 5 * before + 1024, "{old:?}->{work:?}");
                }
            }
            previous = Some(work);
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM and the host C linker"]
fn constant_interchange_images_execute_without_wide_runtime_arithmetic() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/wide-interchange-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (packet, owner) in PACKETS {
        for width in [16, 128] {
            let prefix = format!("{owner}/tests/spec/{packet}");
            let source = resolve::fixture(&format!("{prefix}/observe-{width}.c"))?;
            let expected = resolve::fixture(&format!("{prefix}/f{width}.bits"))?;
            let count = expected.lines().count();
            let observer = format!("#define COUNT {count}\n{source}");
            crate::ffi_program::check(
                root,
                &folder,
                &format!("{packet}-{owner}-{width}"),
                &matrix(packet, owner, width)?,
                &observer,
                &expected,
            )?;
        }
    }
    Ok(())
}
