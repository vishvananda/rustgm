//! Optional typed diagnostic events; the caller owns all output and I/O.
use super::{Census, Location, Record};
use crate::resolve::{expansion::Report, model::DefId, package::Error, unit::NodeKey};

#[derive(Clone, Copy, Debug)]
pub enum ItemKind {
    Implementation,
    Function,
    Initializer,
}
#[derive(Clone, Copy)]
pub enum Progress<'a> {
    Item {
        kind: ItemKind,
        definition: DefId,
        key: NodeKey,
        location: &'a Location,
    },
    Record {
        index: usize,
        record: &'a Record,
    },
}
pub type Observer = Box<dyn for<'a> FnMut(Progress<'a>) -> Result<(), String>>;
#[derive(Default)]
pub(super) struct State {
    observer: Option<Observer>,
    error: Option<String>,
}
impl State {
    fn notify(&mut self, event: Progress<'_>) {
        if self.error.is_none()
            && let Some(observer) = &mut self.observer
        {
            self.error = observer(event).err();
        }
    }
    pub(super) fn record(&mut self, index: usize, record: &Record) {
        self.notify(Progress::Record { index, record });
    }
}
impl Census {
    /// The observer receives diagnostics only and cannot supply frontend facts.
    #[must_use]
    pub fn with_observer(observer: Observer) -> Self {
        Self {
            progress: State {
                observer: Some(observer),
                error: None,
            },
            ..Self::default()
        }
    }
    #[must_use]
    pub fn progress_error(&self) -> Option<&str> {
        self.progress.error.as_deref()
    }
    pub(crate) fn progress_item(
        &mut self,
        report: &Report,
        kind: ItemKind,
        definition: DefId,
        key: Option<NodeKey>,
    ) -> Result<(), Error> {
        if let Some(error) = &self.progress.error {
            return Err(Error::unlocated(error));
        }
        if self.progress.observer.is_none() {
            return Ok(());
        }
        let Some(key) = key else {
            return Ok(());
        };
        let offset = report
            .package
            .units
            .get(key.file.0 as usize)
            .and_then(|unit| unit.configured.ast.nodes.get(key.node.get() as usize))
            .map(|node| node.span.start)
            .ok_or_else(|| Error::unlocated("census progress item lacks its source node"))?;
        let location = Location::of(&report.diagnostic(Error::at(key.file, offset, "")));
        self.progress.notify(Progress::Item {
            kind,
            definition,
            key,
            location: &location,
        });
        self.progress
            .error
            .as_ref()
            .map_or(Ok(()), |error| Err(Error::unlocated(error)))
    }
}
