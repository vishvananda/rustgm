//! A graph's permission follows its actual impl site, independently of Min checks.
use super::{
    bindings::FeatureName,
    expansion::{MacroAllowanceSession, Report},
    model::{DefId, DefKind},
    package::Error,
    unit::{FileId, NodeKey, contexts},
};
use crate::{
    expand::marks::SyntaxContextId,
    support::source::OriginalImage,
    syntax::{ast::Kind, item::ItemKind, token_tree::Span},
};
use std::sync::Arc;

#[derive(Clone)]
pub(crate) struct Requests {
    root: NodeKey,
    image: Arc<OriginalImage>,
    minimal: bool,
    full: bool,
}
impl Requests {
    pub(crate) fn new(report: &Report) -> Result<Self, Error> {
        let unit = report
            .package
            .units
            .first()
            .ok_or_else(|| Error::unlocated("specialization requires its actual crate root"))?;
        let (minimal, full) = report
            .package
            .specialization_requests()
            .map_err(Error::unlocated)?;
        Ok(Self {
            root: NodeKey {
                file: FileId(0),
                node: unit.configured.ast.root,
            },
            image: Arc::clone(unit.source.original_image()),
            minimal,
            full,
        })
    }
    pub(crate) fn min_checks(&self) -> bool {
        self.minimal
    }
    pub(crate) fn validate(&self, report: &Report) -> Result<(), Error> {
        let unit = report
            .package
            .units
            .first()
            .ok_or_else(|| Error::unlocated("specialization root disappeared"))?;
        if unit.configured.ast.root != self.root.node
            || !Arc::ptr_eq(&self.image, unit.source.original_image())
            || report
                .package
                .specialization_requests()
                .map_err(Error::unlocated)?
                != (self.minimal, self.full)
        {
            return Err(Error::unlocated(
                "specialization crate request origin changed",
            ));
        }
        Ok(())
    }
    pub(crate) fn capture(
        &self,
        report: &Report,
        session: &mut MacroAllowanceSession<'_>,
        definition: DefId,
        mut charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<ImplPolicy, Error> {
        self.validate(report)?;
        let (source, syntax, span) = implementation(report, definition)?;
        let gate = if self.minimal || self.full {
            Some(Admission::CrateRequest)
        } else {
            let mut found = None;
            for (name, gate) in [
                ("specialization", TokenGate::Full),
                ("min_specialization", TokenGate::Minimal),
            ] {
                let feature = FeatureName::new(name).map_err(Error::unlocated)?;
                if session.allows_at(source.file, span, &feature)? {
                    found = Some(gate);
                    break;
                }
            }
            if let Some(gate) = found {
                let context = contexts::at_counted(
                    &report.package.units,
                    source.file,
                    span.start,
                    |steps, links, lookups| {
                        charge(steps.saturating_add(links).saturating_add(lookups), 0)
                    },
                )?;
                Some(Admission::Token { gate, context })
            } else {
                None
            }
        };
        charge(1, size_of::<ImplPolicy>())
            .map_err(|message| Error::at(source.file, span.start, message))?;
        Ok(ImplPolicy {
            definition,
            source,
            syntax,
            span,
            image: Arc::clone(report.package.unit(source.file).source.original_image()),
            admission: gate,
        })
    }
}
#[derive(Clone, Copy)]
enum TokenGate {
    Minimal,
    Full,
}
enum Admission {
    CrateRequest,
    Token {
        gate: TokenGate,
        context: SyntaxContextId,
    },
}
pub(crate) struct ImplPolicy {
    definition: DefId,
    pub(crate) source: NodeKey,
    syntax: ItemKind,
    span: Span,
    image: Arc<OriginalImage>,
    admission: Option<Admission>,
}
impl ImplPolicy {
    pub(crate) fn may_specialize(&self) -> bool {
        self.admission.is_some()
    }
    pub(crate) fn validate(
        &self,
        report: &Report,
        requests: &Requests,
        mut charge: impl FnMut(usize, usize) -> Result<(), &'static str>,
    ) -> Result<(), Error> {
        requests.validate(report)?;
        let (source, syntax, span) = implementation(report, self.definition)?;
        let error = |message| Error::at(self.source.file, self.span.start, message);
        charge(1, 0).map_err(error)?;
        if source != self.source
            || syntax != self.syntax
            || span != self.span
            || !Arc::ptr_eq(
                &self.image,
                report.package.unit(source.file).source.original_image(),
            )
        {
            return Err(error("specialization actual impl source changed"));
        }
        match self.admission {
            Some(Admission::CrateRequest) if !requests.minimal && !requests.full => {
                return Err(error("specialization crate request disappeared"));
            }
            Some(Admission::Token { gate, context }) => {
                let actual = contexts::at_counted(
                    &report.package.units,
                    source.file,
                    span.start,
                    |steps, links, lookups| {
                        charge(steps.saturating_add(links).saturating_add(lookups), 0)
                    },
                )?;
                let feature = FeatureName::new(match gate {
                    TokenGate::Minimal => "min_specialization",
                    TokenGate::Full => "specialization",
                })
                .map_err(error)?;
                if actual != context
                    || !report
                        .macro_allowances()
                        .allows_at(source.file, span, &feature)?
                {
                    return Err(error("specialization actual impl allowance changed"));
                }
            }
            Some(Admission::CrateRequest) | None => {}
        }
        Ok(())
    }
}
fn implementation(report: &Report, definition: DefId) -> Result<(NodeKey, ItemKind, Span), Error> {
    let invalid =
        || Error::unlocated("specialization requires its real local impl Self declaration");
    let actual = report
        .resolution
        .definitions
        .get(definition.0 as usize)
        .ok_or_else(invalid)?;
    let key = actual
        .node
        .filter(|_| actual.kind == DefKind::SelfType)
        .ok_or_else(invalid)?;
    let unit = report
        .package
        .units
        .get(key.file.0 as usize)
        .ok_or_else(invalid)?;
    let node = unit
        .configured
        .ast
        .nodes
        .get(key.node.get() as usize)
        .ok_or_else(invalid)?;
    let Kind::Item(index) = node.kind else {
        return Err(invalid());
    };
    let item = unit
        .configured
        .ast
        .items
        .get(index as usize)
        .ok_or_else(invalid)?;
    if unit.configured.active.get(key.node.get() as usize) != Some(&true)
        || !matches!(
            item.kind,
            ItemKind::Impl {
                trait_path: Some(_),
                ..
            }
        )
    {
        return Err(invalid());
    }
    Ok((key, item.kind, node.span))
}
