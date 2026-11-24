//! Field queries use the accounting owner of the actual declaration.
use crate::{
    resolve::body_origin::DeclarationOrigin,
    sema::{crates, header_attributes, model::Declarations, must_use},
};
use std::cell::RefCell;

pub(super) fn query(
    facts: &Declarations,
    origin: DeclarationOrigin,
    steps: usize,
) -> Result<(), String> {
    match origin {
        DeclarationOrigin::Source(_) => {
            header_attributes::charge_source_contract_query(facts, steps)
        }
        DeclarationOrigin::Imported(_) => crates::charge_source_header_query(facts, steps),
    }
    .map_err(|error| error.to_string())
}

pub(super) fn retain<'a>(
    origin: DeclarationOrigin,
    contracts: Option<&'a must_use::Contracts>,
    linkage: Option<&'a mut crates::Linkage>,
) -> Result<impl Fn(usize, usize) -> Result<(), &'static str> + 'a, String> {
    enum Account<Source, Imported> {
        Source(Source),
        Imported(Imported),
    }
    let account = match origin {
        DeclarationOrigin::Source(_) => Account::Source(
            header_attributes::source_payments(contracts).map_err(|error| error.to_string())?,
        ),
        DeclarationOrigin::Imported(_) => Account::Imported(RefCell::new(
            crates::source_header_payments(linkage).map_err(str::to_owned)?,
        )),
    };
    Ok(move |steps, bytes| match &account {
        Account::Source(payments) => payments.charge(steps, bytes),
        Account::Imported(payments) => payments
            .try_borrow_mut()
            .map_err(|_| "imported Field payment is already borrowed")?
            .pay(steps, bytes),
    })
}
