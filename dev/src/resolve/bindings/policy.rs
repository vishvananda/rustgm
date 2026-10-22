//! Source-provider admission consumes retained root attributes before any fallback.
use super::{Context, SourceCore};
use crate::{
    resolve::{
        package::Error,
        unit::{FileId, Unit},
    },
    support::bindings::Mode,
    syntax::attribute::Input,
};

#[derive(Default)]
pub(in crate::resolve) struct Features(u8);
#[derive(Clone, Copy)]
pub(in crate::resolve) enum Feature {
    NoCore,
    LangItems,
    Intrinsics,
    RustcAttributes,
}
impl Features {
    pub(in crate::resolve) fn has(&self, feature: Feature) -> bool {
        self.0 & (1 << feature as u8) != 0
    }
    fn insert(&mut self, feature: Feature) {
        self.0 |= 1 << feature as u8;
    }
}

pub(in crate::resolve) fn prepare(
    unit: &Unit,
    context: &Context,
) -> Result<Option<Box<SourceCore>>, Error> {
    let root = unit.configured.ast.root;
    let no_core = unit
        .word_attribute(Some(root), "no_core")
        .map_err(|e| Error::at(FileId(0), e.offset, e.message))?;
    let Mode::Source { provider } = &context.mode else {
        if no_core {
            return Err(Error::at(
                FileId(0),
                0,
                "no_core requires explicit source-core configuration",
            ));
        }
        return Ok(None);
    };
    let current = context.current.as_ref().ok_or_else(|| {
        Error::at(
            FileId(0),
            0,
            "source-core configuration requires complete current crate identity",
        )
    })?;
    if current != provider || context.crate_name.as_deref() != Some(current.name()) {
        return Err(Error::at(
            FileId(0),
            0,
            "source-core provider differs from current crate identity",
        ));
    }
    if !no_core {
        return Err(Error::at(
            FileId(0),
            0,
            "source-core provider requires no_core",
        ));
    }
    let features = features(unit)?;
    if !features.has(Feature::NoCore) {
        return Err(Error::at(
            FileId(0),
            0,
            "no_core requires its explicit source feature",
        ));
    }
    Ok(Some(Box::new(SourceCore {
        current: current.clone(),
        provider: provider.clone(),
        features,
    })))
}

fn features(unit: &Unit) -> Result<Features, Error> {
    let mut result = Features::default();
    let Some(index) = unit.owner_indices[unit.configured.ast.root.0 as usize] else {
        return Ok(result);
    };
    let owner = &unit.configured.owners[index.get() as usize - 1];
    for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
        if attribute.name(forest, unit.source.text()) != Some("feature") {
            continue;
        }
        let failure = || {
            Error::at(
                FileId(0),
                attribute.span.start,
                "invalid source feature request",
            )
        };
        let Input::Delimited(group) = attribute.input else {
            return Err(failure());
        };
        let range = forest.children(group).ok_or_else(failure)?;
        let mut cursor = range.start;
        while cursor < range.end {
            match forest
                .name(unit.source.text(), cursor)
                .ok_or_else(failure)?
            {
                "no_core" => result.insert(Feature::NoCore),
                "lang_items" => result.insert(Feature::LangItems),
                "intrinsics" => result.insert(Feature::Intrinsics),
                "rustc_attrs" => result.insert(Feature::RustcAttributes),
                _ => {
                    return Err(Error::at(
                        FileId(0),
                        attribute.span.start,
                        "unsupported active source-core feature",
                    ));
                }
            }
            cursor += 1;
            if cursor < range.end {
                if !forest.punctuation(cursor, crate::lex::token::Punctuation::Comma) {
                    return Err(failure());
                }
                cursor += 1;
            }
        }
    }
    Ok(result)
}
