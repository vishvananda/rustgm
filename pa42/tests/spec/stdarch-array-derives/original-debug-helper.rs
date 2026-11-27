#[inline]
pub(crate) fn debug_simd_finish<T: crate::fmt::Debug, const N: usize>(
    formatter: &mut crate::fmt::Formatter<'_>,
    type_name: &str,
    array: &[T; N],
) -> crate::fmt::Result {
    crate::fmt::Formatter::debug_tuple_fields_finish(
        formatter,
        type_name,
        &crate::array::from_fn::<&dyn crate::fmt::Debug, N, _>(|i| &array[i]),
    )
}
