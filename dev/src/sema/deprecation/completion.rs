//! Join pending lexical expectations with unfinished bodies once per request.
use crate::{
    resolve::{
        expansion::Report,
        lint_policy::{ExpectationId, PendingExpectation},
        model::DefId,
        package::Error,
        unit::NodeKey,
    },
    sema::{model::Declarations, safety::Completion},
    support::{hash::HashSet, retained},
    syntax::{ast::Kind, item::ItemKind},
};

#[derive(Default)]
pub(super) struct Scopes {
    whole: bool,
    blocked: HashSet<ExpectationId>,
}
impl Scopes {
    pub(super) fn collect(
        report: &Report,
        facts: &Declarations,
        pending: &[PendingExpectation<'_>],
        bodies: bool,
    ) -> Result<Self, Error> {
        let mut result = Self::default();
        if !bodies || pending.is_empty() {
            return Ok(result);
        }
        let producer = report
            .lint_producer()
            .ok_or_else(|| Error::unlocated("lint completion lost its source producer"))?;
        let root = NodeKey {
            file: report.package.modules[0].file,
            node: report.package.modules[0].root,
        };
        producer.attribute_read(root, pending.len().saturating_mul(4))?;
        let candidate = |row: &&PendingExpectation<'_>| {
            let completion = facts.safety_policies.completion(row.owner);
            !row.resolution_complete
                && match completion {
                    Completion::Whole => true,
                    Completion::Function(_) | Completion::Initializer(_) => {
                        checked(facts, completion)
                    }
                    Completion::Declaration => false,
                }
        };
        let count = pending.iter().filter(candidate).count();
        if count == 0 {
            return Ok(result);
        }
        producer.diagnostic_payload(
            root,
            count.saturating_mul(4),
            retained::new_map::<ExpectationId, ()>(count)
                .saturating_mul(2)
                .saturating_add(2 * size_of::<HashSet<ExpectationId>>()),
        )?;
        let mut wanted = HashSet::with_capacity_and_hasher(
            count,
            crate::support::hash::BuildFxHasher::default(),
        );
        result.blocked = HashSet::with_capacity_and_hasher(
            count,
            crate::support::hash::BuildFxHasher::default(),
        );
        wanted.extend(pending.iter().filter(candidate).map(|row| row.id));
        result.whole = true;
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            producer.attribute_read(root, 1)?;
            let Some(key) = definition.node else {
                continue;
            };
            let unit = report.package.unit(key.file);
            let Kind::Item(item) = unit.configured.ast.node(key.node).kind else {
                continue;
            };
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::at(key.file, 0, "lint completion definition overflow"))?,
            );
            let complete = match unit.configured.ast.items[item as usize].kind {
                ItemKind::Function { body: Some(_), .. } => {
                    checked(facts, Completion::Function(id))
                }
                ItemKind::Const { value: Some(_), .. }
                | ItemKind::Static { value: Some(_), .. } => {
                    checked(facts, Completion::Initializer(id))
                }
                _ => continue,
            };
            if complete {
                continue;
            }
            result.whole = false;
            let expectations = facts.safety_policies.expectation_ids(key)?;
            producer.attribute_read(key, expectations.len().saturating_mul(2))?;
            for id in expectations.iter().flatten() {
                if wanted.contains(id) {
                    result.blocked.insert(*id);
                }
            }
        }
        Ok(result)
    }
    pub(super) fn completed(
        &self,
        facts: &Declarations,
        completion: Completion,
        id: ExpectationId,
        bodies: bool,
    ) -> bool {
        match completion {
            Completion::Declaration => true,
            Completion::Whole => bodies && self.whole,
            Completion::Function(_) | Completion::Initializer(_) => {
                bodies && checked(facts, completion) && !self.blocked.contains(&id)
            }
        }
    }
}
fn checked(facts: &Declarations, completion: Completion) -> bool {
    match completion {
        Completion::Function(id) => facts.typed_function_bodies.contains(&id),
        Completion::Initializer(id) => facts
            .initializer_index(id)
            .is_some_and(|index| facts.initializers[index].type_checked()),
        Completion::Whole | Completion::Declaration => false,
    }
}
