extern crate restricted_provider as provider;
pub fn use_public<T: provider::Restricted>() {}
pub struct Local;
impl provider::Free for Local {}
