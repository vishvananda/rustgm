use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body::{
        self,
        ir::{Action, Owner, Place, Rvalue},
        storage::Location,
    },
    borrow::ownership::{
        PathId,
        initialization::{self, Status},
    },
};

#[test]
fn sparse_partitions_retain_gap_flags_without_expanding_array_elements()
-> Result<(), Box<dyn std::error::Error>> {
    let mut observed = None;
    for length in [8, 1_000_000_000] {
        let source = format!(
            "struct D;impl Drop for D{{fn drop(&mut self){{}}}}fn mark(){{}}fn f(flag:bool,items:[D;{length}]){{if flag{{let [first,..,last]=items;}}mark();}}"
        );
        rustc::qualify(&format!("drop-array-{length}"), &source, true)?;
        let (report, mut facts) = semantic::analyze(&source)?;
        let program = body::build(&report, &facts)?;
        let named = |id, name| {
            report
                .package
                .names
                .get(report.resolution.definitions[id as usize].name)
                == name
        };
        let cfg = program
            .bodies
            .iter()
            .find(|b| matches!(b.owner,Owner::Function{definition,..} if named(definition.0,"f")))
            .ok_or("f")?;
        let local = cfg
            .locals
            .iter()
            .position(|l| l.definition.is_some_and(|id| named(id.0, "items")))
            .ok_or("items")?;
        let mark = facts
            .functions
            .iter()
            .find(|f| named(f.definition.0, "mark"))
            .ok_or("mark")?
            .definition;
        let init = initialization::analyze(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            cfg,
        )
        .map_err(|e| e.message)?;
        let segments = init
            .ownership
            .paths
            .array_segments(PathId(u32::try_from(local)?))
            .ok_or("array partitions")?;
        assert_eq!(segments.len(), 3);
        assert_eq!(
            segments
                .iter()
                .map(|s| (s.start, s.end, s.element.is_some()))
                .collect::<Vec<_>>(),
            vec![
                (0, 1, true),
                (1, length - 1, false),
                (length - 1, length, true)
            ]
        );
        let size = (
            init.ownership.paths.entries.len(),
            init.ownership.paths.bits,
        );
        if let Some(previous) = observed {
            assert_eq!(size, previous);
        } else {
            observed = Some(size);
        }
        assert!(size.1 < 32);
        let mut checked = 0;
        init.visit(|location, point| {
            let Location::At { block, before } = location else {
                return Err("edge");
            };
            if before == cfg.blocks[block.0 as usize].end {
                return Ok(());
            }
            if let Action::Evaluate(id) = cfg.statements[before as usize].action
                && let Rvalue::Call { callee, .. } = cfg.values[id.0 as usize].kind
                && matches!(cfg.values[callee.0 as usize].kind,Rvalue::Function(id) if id==mark)
            {
                for (index, segment) in segments.iter().enumerate() {
                    let expected = if index == 1 {
                        Status::Always
                    } else {
                        Status::Conditional
                    };
                    assert_eq!(point.bit(segment.bit)?, expected);
                    assert_eq!(init.bit_at_entry(block, segment.bit)?, expected);
                }
                checked += 1;
            }
            Ok(())
        })?;
        assert_eq!(checked, 1);
        assert!(
            cfg.values
                .iter()
                .any(|v| matches!(v.kind, Rvalue::Read(Place::Projection(_))))
        );
    }
    Ok(())
}
