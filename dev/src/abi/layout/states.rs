//! Layout state is allocated by demand, in pages of consecutive type identities.
use super::{Budget, State, TypeId};

const PAGE_LEN: usize = 32;
type Page = [State; PAGE_LEN];

#[derive(Default)]
pub(super) struct States {
    pages: Vec<Option<Box<Page>>>,
}
impl States {
    pub(super) fn get(&self, ty: TypeId) -> Option<&State> {
        self.pages
            .get(ty.0 as usize / PAGE_LEN)?
            .as_ref()
            .map(|page| &page[ty.0 as usize % PAGE_LEN])
    }

    pub(super) fn get_mut(&mut self, ty: TypeId) -> Option<&mut State> {
        self.pages
            .get_mut(ty.0 as usize / PAGE_LEN)?
            .as_mut()
            .map(|page| &mut page[ty.0 as usize % PAGE_LEN])
    }

    pub(super) fn prepare<const LIMITED: bool>(
        &mut self,
        ty: TypeId,
        budget: &mut Budget,
        steps: &mut usize,
    ) -> Result<(), &'static str> {
        let page = ty.0 as usize / PAGE_LEN;
        if self.pages.get(page).is_some_and(Option::is_some) {
            return Ok(());
        }
        let missing = (page + 1).saturating_sub(self.pages.len());
        if LIMITED {
            let copied = if page >= self.pages.capacity() {
                self.pages.len()
            } else {
                0
            };
            budget.charge(
                steps,
                PAGE_LEN.saturating_add(missing).saturating_add(copied),
            )?;
        }
        self.pages.try_reserve(missing).map_err(|_| {
            budget.exhausted = true;
            "layout state allocation failed"
        })?;
        self.pages.resize_with(self.pages.len() + missing, || None);
        self.pages[page] = Some(Box::new([State::Unseen; PAGE_LEN]));
        Ok(())
    }
}
