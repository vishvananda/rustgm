//! Inert data controls establish ownership/cost only, never const Drop permission.
use super::*;
use crate::{
    body::{
        drops::{glue::Range, schedule::Event},
        ir::BlockId,
        storage::Location,
    },
    const_eval::ctglue::tests,
};
type Result<T = ()> = std::result::Result<T, String>;
pub(in crate::const_eval) fn fixture() -> Result<Rc<Code>> {
    let (_, body, bundle) = tests::inert()?;
    let mut bundle = Rc::try_unwrap(bundle).map_err(|_| "shared fixture")?;
    bundle.view.flag_count = 65;
    let location = Location::At {
        block: BlockId(0),
        before: 0,
    };
    bundle.view.events = vec![
        Event {
            location,
            effect: Effect::Flags {
                range: Range { start: 0, end: 65 },
                initialized: true,
            },
        },
        Event {
            location,
            effect: Effect::Flags {
                range: Range { start: 64, end: 65 },
                initialized: false,
            },
        },
    ];
    let bundle = Rc::new(bundle);
    let mut code = Code::snapshot(&body)?;
    code.operands
        .push(crate::const_eval::code::PreparedOperand::Cleanup(
            Rc::clone(&bundle),
        ));
    let code = Rc::new(code);
    bundle.bind(&code)?;
    Ok(code)
}
#[test]
fn each_frame_owns_flag_storage_and_writes_only_its_current_plan_event() -> Result {
    let code = fixture()?;
    let mut parent = Cursor::new(&code, 7, None)
        .map_err(|e| e.message().to_owned())?
        .ok_or("parent")?;
    let child = Cursor::new(&code, 8, None)
        .map_err(|e| e.message().to_owned())?
        .ok_or("child")?;
    assert_eq!(
        parent.retained_slots(),
        Cursor::slots(&code, None).map_err(|e| e.message().to_owned())?
    );
    parent.event = NonZeroU32::new(1);
    assert!(parent.write(&code, 8, 0).is_err());
    assert!(parent.write(&code, 7, 1).is_err());
    parent
        .write(&code, 7, 0)
        .map_err(|e| e.message().to_owned())?;
    for index in [0, 63, 64] {
        let guard = Guard::initialized(index)?;
        assert!(parent.bit(guard, 65).map_err(|e| e.message().to_owned())?);
        assert!(!child.bit(guard, 65).map_err(|e| e.message().to_owned())?);
    }
    parent.event = NonZeroU32::new(2);
    parent
        .write(&code, 7, 1)
        .map_err(|e| e.message().to_owned())?;
    assert!(
        parent
            .bit(Guard::initialized(63)?, 65)
            .map_err(|e| e.message().to_owned())?
    );
    assert!(
        !parent
            .bit(Guard::initialized(64)?, 65)
            .map_err(|e| e.message().to_owned())?
    );
    assert!(parent.bit(Guard::initialized(65)?, 65).is_err());
    Ok(())
}
#[test]
fn foreign_footer_high_bits_and_changed_storage_extent_never_become_flags() -> Result {
    let code = fixture()?;
    let other = fixture()?;
    let mut state = Cursor::new(&code, 3, None)
        .map_err(|e| e.message().to_owned())?
        .ok_or("state")?;
    state.event = NonZeroU32::new(1);
    assert!(state.write(&other, 3, 0).is_err());
    assert!(state.flags.iter().all(|word| *word == 0));
    state.flags[1] = 2;
    assert!(state.require_frame(&code, 3).is_err());
    state.flags[1] = 0;
    state.flags.pop();
    assert!(state.require_frame(&code, 3).is_err());
    Ok(())
}
#[test]
fn no_cleanup_path_owns_no_cold_allocation_and_existing_widths_are_preserved() -> Result {
    let (_, body, _) = tests::inert()?;
    let code = Code::snapshot(&body)?;
    assert_eq!(
        Cursor::slots(&code, None).map_err(|e| e.message().to_owned())?,
        0
    );
    assert!(
        Cursor::new(&code, 0, None)
            .map_err(|e| e.message().to_owned())?
            .is_none()
    );
    assert_eq!(size_of::<Cursor>(), 48);
    assert_eq!(size_of::<Frame>(), 248);
    assert_eq!(size_of::<Code>(), 304);
    assert_eq!(size_of::<Guard>(), 4);
    Ok(())
}
