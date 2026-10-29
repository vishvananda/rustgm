//! Repetition consumes checked usize counts in its actual declaration context.
use super::source;
use std::fmt::Write;

#[test]
fn count_queries_share_one_callee_and_scale_with_actual_repeat_sites()
-> Result<(), Box<dyn std::error::Error>> {
    for width in [8, 32, 128] {
        let mut text = String::from("const fn count(n:usize)->usize{n+1}fn main(){");
        for index in 0..width {
            write!(text, "let _=[0u8;count({index})];")?;
        }
        text.push('}');
        let mut report = source::resolve(&text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert_eq!(facts.constant_work.expressions, width);
        assert_eq!(facts.body_work.cfg_functions.get(), 2);
        assert!(facts.constant_work.steps <= width * 128);
        // Distinct query roots and the runtime caller retain their real code;
        // only the shared callee's CFG construction is constant above.
        assert!(facts.constant_work.code_bytes <= width * 8192);
    }
    Ok(())
}
