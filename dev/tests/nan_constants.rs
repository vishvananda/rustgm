//! Compile-time float arithmetic produces rustc's positive quiet NaN, while a
//! copy, a reinterpretation or a negation keeps its exact bits into run time.
use crate::native_references as native;

/// One float width with the other width's operands for the cast cases.
struct Width {
    name: &'static str,
    float: &'static str,
    bits: &'static str,
    payload: &'static str,
    signalling: &'static str,
    other_float: &'static str,
    other_bits: &'static str,
    other_payload: &'static str,
}
const WIDTHS: [Width; 2] = [
    Width {
        name: "single",
        float: "f32",
        bits: "u32",
        payload: "0xFFC0_0001",
        signalling: "0x7FA0_0000",
        other_float: "f64",
        other_bits: "u64",
        other_payload: "0xFFF8_0000_0000_0001",
    },
    Width {
        name: "double",
        float: "f64",
        bits: "u64",
        payload: "0xFFF8_0000_0000_0001",
        signalling: "0x7FF4_0000_0000_0000",
        other_float: "f32",
        other_bits: "u32",
        other_payload: "0xFFC0_0001",
    },
];

/// `(name, expression, f32 bits, f64 bits)`, declared from the rules of rustc
/// 1.98.0's compile-time machine before either compiler ran: an operation that
/// computes a NaN gives the positive quiet NaN whatever its operands carry; a
/// value that is only read, copied or cast to its own type keeps every bit;
/// negation flips exactly the sign bit.
const CASES: &[(&str, &str, u32, u64)] = &[
    (
        "zero-quotient",
        "0.0/0.0",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "infinite-difference",
        "INF-INF",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "infinite-product",
        "INF*0.0",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "zero-remainder",
        "0.0%0.0",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "infinite-quotient",
        "INF/INF",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "payload-sum",
        "PAYLOAD+1.0",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "signalling-product",
        "SIGNAL*1.0",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "payload-width-cast",
        "OTHER as F",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    (
        "quiet-width-cast",
        "OTHER_NAN as F",
        0x7fc0_0000,
        0x7ff8_0000_0000_0000,
    ),
    ("named", "NAN", 0x7fc0_0000, 0x7ff8_0000_0000_0000),
    ("negated", "-NAN", 0xffc0_0000, 0xfff8_0000_0000_0000),
    ("payload", "PAYLOAD", 0xffc0_0001, 0xfff8_0000_0000_0001),
    ("signalling", "SIGNAL", 0x7fa0_0000, 0x7ff4_0000_0000_0000),
    ("copied", "COPY", 0xffc0_0001, 0xfff8_0000_0000_0001),
    (
        "negated-payload",
        "-PAYLOAD",
        0x7fc0_0001,
        0x7ff8_0000_0000_0001,
    ),
    (
        "twice-negated",
        "-(-PAYLOAD)",
        0xffc0_0001,
        0xfff8_0000_0000_0001,
    ),
    (
        "same-width-cast",
        "PAYLOAD as F",
        0xffc0_0001,
        0xfff8_0000_0000_0001,
    ),
];

/// Read the expression's bits at compile time, then at run time from an
/// immediate operand, static storage, a promoted borrow and an inline constant.
fn program(width: &Width, expression: &str, bits: u64) -> String {
    let Width {
        float,
        bits: carrier,
        payload,
        signalling,
        other_float,
        other_bits,
        other_payload,
        ..
    } = width;
    format!(
        "type F={float};
const fn bits(x:F)->{carrier}{{unsafe{{*(&raw const x as *const {carrier})}}}}
const fn float(x:{carrier})->F{{unsafe{{*(&raw const x as *const F)}}}}
const fn other(x:{other_bits})->{other_float}{{unsafe{{*(&raw const x as *const {other_float})}}}}
const INF:F=1.0/0.0;
const NAN:F=0.0/0.0;
const PAYLOAD:F=float({payload});
const SIGNAL:F=float({signalling});
const COPY:F=PAYLOAD;
const OTHER:{other_float}=other({other_payload});
const OTHER_NAN:{other_float}=0.0/0.0;
const V:F={expression};
static S:F={expression};
const B:{carrier}=bits(V);
fn main(){{
    let _=(INF,NAN,SIGNAL,COPY,OTHER,OTHER_NAN);
    check(B=={bits:#x});
    check(bits(V)=={bits:#x});
    check(bits(S)=={bits:#x});
    let promoted:&'static F=&({expression});
    check(bits(*promoted)=={bits:#x});
    let inline:F=const{{{expression}}};
    check(bits(inline)=={bits:#x});
}}
"
    )
}

#[test]
#[ignore = "requires the pinned CPPGM backend; run make test-maintainer-native"]
fn constant_nans_keep_rustc_bits_through_run_time() -> Result<(), Box<dyn std::error::Error>> {
    let mut sources = Vec::new();
    for width in &WIDTHS {
        for &(name, expression, single, double) in CASES {
            let bits = if width.float == "f32" {
                u64::from(single)
            } else {
                double
            };
            sources.push((
                format!("{}-{name}", width.name),
                program(width, expression, bits),
            ));
        }
    }
    let cases: Vec<_> = sources
        .iter()
        .map(|(name, source)| (name.as_str(), source.as_str(), false))
        .collect();
    native::check_cases("parallel/nan-constants/native", &cases)
}
