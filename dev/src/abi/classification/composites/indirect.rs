//! Argument-policy summaries share the existing optional completed-layout cache.
use super::{Cache, Definitions, Layouts, Store, TypeId};
use crate::types::{Kind, aggregate::Representation};

impl Cache {
    pub(in crate::abi::classification) fn indirect_argument(
        &mut self,
        types: &Store,
        definitions: &Definitions,
        layouts: &mut Layouts,
        ty: TypeId,
        steps: &mut usize,
        retained: &mut usize,
    ) -> Result<bool, &'static str> {
        let mut head = types.storage_head(ty)?;
        if let Some(&answer) = self.indirect.get(&head) {
            return answer;
        }
        let mut path = Vec::new();
        let answer = (|| loop {
            super::super::account_work(steps, 1)?;
            if let Some(&answer) = self.indirect.get(&head) {
                return answer;
            }
            super::super::storage(retained, 64)?;
            path.push(head);
            let id = layouts
                .demand(types, definitions, head)
                .map_err(|e| e.message)?;
            let layout = *layouts.at(id).ok_or("missing argument policy layout")?;
            let Some(Kind::Adt(item, _)) = types.kind(head) else {
                return Ok(false);
            };
            let definition = definitions
                .get(item)
                .ok_or("missing argument policy declaration")?;
            if definition.repr.arguments == crate::types::aggregate::ArgumentPassing::Indirect {
                return Ok(true);
            }
            if definition.repr.representation != Representation::Transparent {
                return Ok(false);
            }
            let [variant] = layouts.variants(&layout) else {
                return Err("transparent argument policy requires one physical variant");
            };
            let fields = types
                .list(variant.fields)
                .ok_or("missing transparent argument fields")?;
            super::super::account_work(steps, fields.len())?;
            let mut child = None;
            for &field in fields {
                let field = types.storage_head(field)?;
                let layout = layouts
                    .get(field)
                    .ok_or("missing transparent argument child")?;
                if layout.size != 0 && child.replace(field).is_some() {
                    return Err("transparent argument policy has multiple payloads");
                }
            }
            let Some(next) = child else {
                return Ok(false);
            };
            // Completed finite layout has already rejected by-value cycles.
            head = next;
        })();
        for head in path {
            self.indirect.insert(head, answer);
        }
        answer
    }
}
