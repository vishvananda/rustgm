//! Range tags authenticate ordinary source structs and a real inherent constructor.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::Declarations,
    support::bindings::LangItem,
};

pub(super) const NOMINALS: [LangItem; 10] = [
    LangItem::RangeFrom,
    LangItem::RangeFull,
    LangItem::RangeInclusiveStruct,
    LangItem::Range,
    LangItem::RangeToInclusive,
    LangItem::RangeTo,
    LangItem::RangeFromCopy,
    LangItem::RangeCopy,
    LangItem::RangeInclusiveCopy,
    LangItem::RangeToInclusiveCopy,
];

pub(in crate::sema) fn complete_constructor(
    report: &Report,
    facts: &mut Declarations,
) -> Result<(), Error> {
    let role = LangItem::RangeInclusiveNew;
    let Some(declaration) = super::headers::functions::inherent(report, facts, role)? else {
        return Ok(());
    };
    super::roles::complete_constructor(facts, role, declaration)
}
