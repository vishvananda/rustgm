use rustgm::{
    literal::value::FloatBits,
    sema::{
        model::Scalar,
        patterns::{
            Interval,
            scalar::{Coverage, Domain},
        },
    },
    types::Primitive,
};

fn scalar(ty: Primitive, bits: u64) -> Result<Scalar, &'static str> {
    Ok(Scalar::Float(match ty {
        Primitive::F32 => FloatBits::F32(u32::try_from(bits).map_err(|_| "wide f32 sample")?),
        Primitive::F64 => FloatBits::F64(bits),
        _ => return Err("not a float"),
    }))
}
fn number(value: Scalar) -> Result<f64, &'static str> {
    match value {
        Scalar::Float(FloatBits::F32(bits)) => Ok(f64::from(f32::from_bits(bits))),
        Scalar::Float(FloatBits::F64(bits)) => Ok(f64::from_bits(bits)),
        _ => Err("not a floating value"),
    }
}
const WIDTHS: &[(Primitive, u64, u64)] = &[
    (Primitive::F32, 1 << 31, 0x7f80_0000),
    (Primitive::F64, 1 << 63, 0x7ff0_0000_0000_0000),
];
#[test]
fn floating_ranks_agree_with_numeric_order_and_preserve_nonzero_bits() -> Result<(), &'static str> {
    for &(ty, sign, infinity) in WIDTHS {
        let domain = Domain::new(ty).ok_or("missing float domain")?;
        let mask = sign | (sign - 1);
        let mut state = 0x7836_2719_abe5_4823_u64;
        let mut previous: Option<(Scalar, u128)> = None;
        for bits in [
            0,
            sign,
            1,
            sign | 1,
            infinity - 1,
            sign | (infinity - 1),
            infinity,
            sign | infinity,
        ]
        .into_iter()
        .chain((0..16_384).map(|_| {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            state & mask
        })) {
            let value = scalar(ty, bits)?;
            if number(value)?.is_nan() {
                assert_eq!(
                    domain.rank(value, false),
                    Err("NaN is not a valid pattern value")
                );
                continue;
            }
            let rank = domain.rank(value, false)?;
            let canonical = if bits & !sign == 0 {
                scalar(ty, 0)?
            } else {
                value
            };
            assert_eq!(domain.value(rank)?, canonical);
            assert_eq!(
                domain.rank(value, true)?,
                domain.rank(scalar(ty, bits ^ sign)?, false)?
            );
            if let Some((earlier, earlier_rank)) = previous {
                assert_eq!(
                    number(earlier)?.partial_cmp(&number(value)?),
                    Some(earlier_rank.cmp(&rank))
                );
            }
            previous = Some((value, rank));
        }
        assert_eq!(
            domain.rank(scalar(ty, 0)?, false)?,
            domain.rank(scalar(ty, sign)?, false)?
        );
        assert_eq!(domain.rank(scalar(ty, sign | infinity)?, false)?, 0);
        assert_eq!(
            domain.rank(scalar(ty, infinity)?, false)?,
            domain.ordered().upper
        );
        assert_eq!(domain.witness(0)?, "-inf");
        assert_eq!(domain.witness(domain.ordered().upper)?, "inf");
        assert_eq!(domain.witness(domain.full().upper)?, "NaN");
        assert!(number(domain.value(domain.full().upper)?)?.is_nan());
        assert!(domain.value(domain.full().upper + 1).is_err());
    }
    Ok(())
}
#[test]
fn nan_payloads_are_unordered_and_only_wildcards_finish_coverage() -> Result<(), &'static str> {
    for &(ty, sign, infinity) in WIDTHS {
        let domain = Domain::new(ty).ok_or("missing float domain")?;
        for payload in [1, 2, (sign - 1) ^ infinity] {
            for negative in [0, sign] {
                for negate in [false, true] {
                    assert_eq!(
                        domain.rank(scalar(ty, infinity | payload | negative)?, negate),
                        Err("NaN is not a valid pattern value")
                    );
                }
            }
        }
        let mut coverage = Coverage::default();
        coverage.insert(domain.ordered())?;
        assert_eq!(
            coverage.missing(domain),
            Some(Interval {
                lower: domain.full().upper,
                upper: domain.full().upper,
            })
        );
        assert!(!coverage.useful(domain.ordered())?);
        assert!(coverage.useful(domain.full())?);
        coverage.insert(domain.full())?;
        assert!(coverage.missing(domain).is_none());
        let other = if ty == Primitive::F32 {
            Primitive::F64
        } else {
            Primitive::F32
        };
        assert!(domain.rank(scalar(other, 0)?, false).is_err());
    }
    Ok(())
}
