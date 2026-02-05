use super::*;

/// A 256-bit integer sort (bitvector semantics) backed by [`ruint::Uint<256, 4>`].
///
/// This sort is intentionally *signless*: signed/unsigned interpretation is
/// provided by distinct primitives.
///
/// Primitives provided:
/// - Con/Destruction: `i256`, `i256-from-string`
/// - Arithmetic: `+`, `-`, `*`, `/`, `%`
/// - Bitwise: `&`, `|`, `^`, `<<`, `>>`, `not-i256`
/// - Unsigned comparisons: `ult`, `ugt`, `ule`, `uge`
/// - Signed comparisons (two's complement): `slt`, `sgt`, `sle`, `sge`
/// - Other: `min`, `max`, `to-string`, `to-hex-string`
#[derive(Debug)]
pub struct I256Sort;

type U256 = ruint::Uint<256, 4>;

fn i256_from_i64(n: i64) -> I256 {
    if n >= 0 {
        I256::new(U256::from(n as u64))
    } else {
        I256::new(U256::from_limbs([n as u64, u64::MAX, u64::MAX, u64::MAX]))
    }
}

fn i256_from_str(s: &str) -> Option<I256> {
    let s = s.trim();
    if let Some(rest) = s.strip_prefix('-') {
        let mag = rest.parse::<U256>().ok()?;
        Some(I256::new(U256::ZERO.wrapping_sub(mag)))
    } else {
        s.parse::<U256>().ok().map(I256::new)
    }
}

fn signed_lt(a: &U256, b: &U256) -> bool {
    let a_sign = (a.as_limbs()[3] >> 63) != 0;
    let b_sign = (b.as_limbs()[3] >> 63) != 0;
    match (a_sign, b_sign) {
        (false, false) => a < b,
        (true, true) => a < b,
        (true, false) => true,
        (false, true) => false,
    }
}

fn signed_le(a: &U256, b: &U256) -> bool {
    !signed_lt(b, a)
}

impl BaseSort for I256Sort {
    type Base = I256;

    fn name(&self) -> &str {
        "i256"
    }

    #[rustfmt::skip]
    fn register_primitives(&self, eg: &mut EGraph) {
        add_primitive!(eg, "i256" = |a: i64| -> I256 { i256_from_i64(a) });
        add_primitive!(eg, "i256-from-string" = |a: S| -?> I256 { i256_from_str(a.as_str()) });

        add_primitive!(eg, "+" = |a: I256, b: I256| -> I256 { I256::new(a.0 + b.0) });
        add_primitive!(eg, "-" = |a: I256, b: I256| -> I256 { I256::new(a.0 - b.0) });
        add_primitive!(eg, "*" = |a: I256, b: I256| -> I256 { I256::new(a.0 * b.0) });
        add_primitive!(eg, "/" = |a: I256, b: I256| -?> I256 { a.0.checked_div(b.0).map(I256::new) });
        add_primitive!(eg, "%" = |a: I256, b: I256| -?> I256 { a.0.checked_rem(b.0).map(I256::new) });

        add_primitive!(eg, "&" = |a: I256, b: I256| -> I256 { I256::new(a.0 & b.0) });
        add_primitive!(eg, "|" = |a: I256, b: I256| -> I256 { I256::new(a.0 | b.0) });
        add_primitive!(eg, "^" = |a: I256, b: I256| -> I256 { I256::new(a.0 ^ b.0) });
        add_primitive!(eg, "<<" = |a: I256, b: i64| -?> I256 { usize::try_from(b).ok().map(|b| I256::new(a.0 << b)) });
        add_primitive!(eg, ">>" = |a: I256, b: i64| -?> I256 { usize::try_from(b).ok().map(|b| I256::new(a.0 >> b)) });
        add_primitive!(eg, "not-i256" = |a: I256| -> I256 { I256::new(!a.0) });

        add_primitive!(eg, "ult" = |a: I256, b: I256| -?> () { (a.0 < b.0).then_some(()) });
        add_primitive!(eg, "ugt" = |a: I256, b: I256| -?> () { (a.0 > b.0).then_some(()) });
        add_primitive!(eg, "ule" = |a: I256, b: I256| -?> () { (a.0 <= b.0).then_some(()) });
        add_primitive!(eg, "uge" = |a: I256, b: I256| -?> () { (a.0 >= b.0).then_some(()) });

        add_primitive!(eg, "slt" = |a: I256, b: I256| -?> () { signed_lt(&a.0, &b.0).then_some(()) });
        add_primitive!(eg, "sgt" = |a: I256, b: I256| -?> () { signed_lt(&b.0, &a.0).then_some(()) });
        add_primitive!(eg, "sle" = |a: I256, b: I256| -?> () { signed_le(&a.0, &b.0).then_some(()) });
        add_primitive!(eg, "sge" = |a: I256, b: I256| -?> () { signed_le(&b.0, &a.0).then_some(()) });

        add_primitive!(eg, "min" = |a: I256, b: I256| -> I256 { I256::new(a.0.min(b.0)) });
        add_primitive!(eg, "max" = |a: I256, b: I256| -> I256 { I256::new(a.0.max(b.0)) });

        add_primitive!(eg, "to-string" = |a: I256| -> S { S::new(a.0.to_string()) });
        add_primitive!(eg, "to-hex-string" = |a: I256| -> S { S::new(format!("{:#x}", a.0)) });
    }

    fn reconstruct_termdag(
        &self,
        base_values: &BaseValues,
        value: Value,
        termdag: &mut TermDag,
    ) -> TermId {
        let n = base_values.unwrap::<I256>(value);
        let hex = termdag.lit(Literal::String(format!("{:#x}", n.0)));
        termdag.app("i256-from-string".to_owned(), vec![hex])
    }
}

