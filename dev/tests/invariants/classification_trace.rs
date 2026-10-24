//! Bounded diagnostics observe an existing proof attempt without changing its outcome.
use std::{cell::RefCell, rc::Rc};

const LIMIT: usize = 256;
#[derive(Default)]
struct Events {
    rows: Vec<String>,
    overflow: bool,
}
thread_local! {
    static ACTIVE: RefCell<Option<Rc<RefCell<Events>>>> = const { RefCell::new(None) };
}
pub(crate) struct Capture(Rc<RefCell<Events>>);
impl Capture {
    pub(crate) fn new() -> Result<Self, &'static str> {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active.is_some() {
                return Err("opaque Copy diagnostic capture is already active");
            }
            let events = Rc::new(RefCell::new(Events::default()));
            *active = Some(Rc::clone(&events));
            Ok(Self(events))
        })
    }
    pub(crate) fn rows(&self) -> Result<Vec<String>, &'static str> {
        let events = self.0.borrow();
        if events.overflow {
            return Err("opaque Copy diagnostic capture exceeded its bound");
        }
        Ok(events.rows.clone())
    }
}
impl Drop for Capture {
    fn drop(&mut self) {
        ACTIVE.with(|active| {
            let mut active = active.borrow_mut();
            if active
                .as_ref()
                .is_some_and(|actual| Rc::ptr_eq(actual, &self.0))
            {
                *active = None;
            }
        });
    }
}
pub(crate) fn record(row: impl FnOnce() -> String) {
    record_checked(|| Some(row()));
}
pub(crate) fn record_checked(row: impl FnOnce() -> Option<String>) {
    ACTIVE.with(|active| {
        let active = active.borrow();
        let Some(events) = active.as_ref() else {
            return;
        };
        let mut events = events.borrow_mut();
        if events.rows.len() == LIMIT {
            events.overflow = true;
        } else if let Some(row) = row() {
            events.rows.push(row);
        } else {
            events.overflow = true;
        }
    });
}

#[test]
fn diagnostic_scope_rejects_reentry_and_clears_after_error() -> Result<(), &'static str> {
    fn interrupted() -> Result<(), &'static str> {
        let _capture = Capture::new()?;
        assert!(Capture::new().is_err());
        record(|| "interrupted".into());
        Err("retained diagnostic error")
    }
    assert!(interrupted().is_err());
    {
        let capture = Capture::new()?;
        record_checked(|| None);
        assert!(capture.rows().is_err());
    }
    let capture = Capture::new()?;
    assert!(capture.rows()?.is_empty());
    for _ in 0..=LIMIT {
        record(|| "bounded".into());
    }
    assert!(capture.rows().is_err());
    Ok(())
}
