//! A reservation message is cold declaration data, never implementation evidence.
use super::{Attribute, Input, NodeKey, Unit, Violation, Work, invalid};
pub(super) struct Message {
    pub effect: u32,
    pub text: Box<str>,
}
pub(super) fn capture(
    unit: &Unit,
    key: NodeKey,
    witness: &super::super::producer::Witness,
    work: &mut Work,
) -> Result<Message, Violation> {
    let attribute = witness.attribute;
    let length = attribute.span.end.saturating_sub(attribute.span.start) as usize;
    work.charge(key, length.saturating_add(1), length.saturating_mul(3))?;
    if attribute.unsafe_wrapper {
        return Err(invalid(
            key,
            "rustc_reservation_impl cannot use an unsafe wrapper",
        ));
    }
    let Input::Equals(range) = attribute.input else {
        return Err(invalid(
            key,
            "rustc_reservation_impl requires a string message",
        ));
    };
    let forest = &unit.configured.ast.attributes[witness.capture as usize].forest;
    let token = (range.end == range.start + 1)
        .then(|| forest.token(range.start))
        .flatten()
        .ok_or_else(|| invalid(key, "rustc_reservation_impl requires one string literal"))?;
    let Some(crate::literal::Value::String(bytes)) =
        crate::literal::interpret(unit.source.text(), token)
            .map_err(|_| invalid(key, "rustc_reservation_impl has an invalid literal"))?
    else {
        return Err(invalid(
            key,
            "rustc_reservation_impl requires a string literal",
        ));
    };
    let text = std::str::from_utf8(bytes.as_ref())
        .map_err(|_| invalid(key, "reservation message is not UTF-8"))?;
    Ok(Message {
        effect: witness.effect,
        text: text.into(),
    })
}
pub(super) fn current<'a>(
    producer: &super::Producer,
    unit: &Unit,
    owner: &super::super::producer::Owner,
    message: &'a Message,
) -> Result<(Attribute, &'a str), Violation> {
    Ok((
        super::attribute(
            producer,
            unit,
            owner,
            message.effect,
            "rustc_reservation_impl",
        )?,
        &message.text,
    ))
}
