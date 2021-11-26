// x^128 + x^7 + x^2 + x + 1
// const generatorPolynomial: u128 = /*(1 << (128 - 128 - 1)) + */ (1 << (128 - 7 - 1)) + (1 << (128 - 2 - 1)) + (1 << (128 - 1 - 1)) + (1 << (128 - 0 - 1));

use std::fmt;
use std::ops;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct GFPoly {
    poly: u128,
}

impl fmt::Debug for GFPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = "".to_string();

        for i in 0..128 {
            if (self.poly >> i) & 1 == 1 {
                if !res.is_empty() {
                    res += " + "
                }
                res += &format!("x^{}", 127 - i)
            }
        }

        f.debug_tuple("").field(&res).finish()
    }
}

impl From<u128> for GFPoly {
    fn from(from: u128) -> GFPoly {
        GFPoly { poly: from }
    }
}

impl From<GFPoly> for u128 {
    fn from(from: GFPoly) -> u128 {
        from.poly
    }
}

impl ops::Add<GFPoly> for GFPoly {
    type Output = GFPoly;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn add(self, rhs: GFPoly) -> GFPoly {
        GFPoly {
            poly: self.poly ^ rhs.poly,
        }
    }
}

impl ops::AddAssign<GFPoly> for GFPoly {
    fn add_assign(&mut self, rhs: GFPoly) {
        *self = *self + rhs;
    }
}

const X128: u128 = 0xe1000000000000000000000000000000u128;

impl ops::Mul<GFPoly> for GFPoly {
    type Output = GFPoly;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: GFPoly) -> GFPoly {
        let mut cum: GFPoly = GFPoly { poly: 0 };
        let mut lhs = self;
        for b in (0..128).rev() {
            if (rhs.poly >> b) & 0b1 == 0b1 {
                cum += lhs;
            }
            lhs = lhs.rightshift();
        }
        cum
    }
}

impl ops::MulAssign<GFPoly> for GFPoly {
    fn mul_assign(&mut self, rhs: GFPoly) {
        let temp = *self * rhs;
        self.poly = temp.poly;
    }
}

impl GFPoly {
    fn rightshift(self) -> GFPoly {
        let shifted = GFPoly {
            poly: self.poly >> 1,
        };
        if self.poly & 0b1 == 0b1 {
            shifted + GFPoly { poly: X128 }
        } else {
            shifted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x128_test() {
        let lhs = GFPoly::from(1);
        let rhs = GFPoly::from(1u128 << 126);

        let actual: u128 = (lhs * rhs).into();
        let expect: u128 = 0xe1000000000000000000000000000000u128;
        assert_eq!(actual, expect);
    }

    #[test]
    fn random_test() {
        let lhs = GFPoly::from(1242141243);
        let rhs = GFPoly::from(123);

        let actual: u128 = (lhs * rhs).into();
        let expect: u128 = 0xfa2800000000000000000028b7d0d77bu128;
        assert_eq!(actual, expect);
    }

    #[test]
    fn simple_test() {
        let cases = vec![
            (
                GFPoly::from(0b1u128 << 127),
                GFPoly::from(0b1u128 << 127),
                GFPoly::from(0b1u128 << 127),
            ),
            (
                GFPoly::from(0b1u128 << 127),
                GFPoly::from(0b0u128 << 127),
                GFPoly::from(0b0u128 << 127),
            ),
            (
                GFPoly::from(0b0u128 << 127),
                GFPoly::from(0b0u128 << 127),
                GFPoly::from(0b0u128 << 127),
            ),
            (
                GFPoly::from(0b01u128 << 126),
                GFPoly::from(0b11u128 << 126),
                GFPoly::from(0b011u128 << 125),
            ),
            (
                GFPoly::from(0b11u128 << 126),
                GFPoly::from(0b11u128 << 126),
                GFPoly::from(0b101u128 << 125),
            ),
        ];
        for (lhs, rhs, expected) in cases {
            let actual = lhs * rhs;
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn overflow_test() {
        let cases = vec![(
            GFPoly::from(0b1u128),
            GFPoly::from(0b01u128 << 126),
            GFPoly::from(X128),
        )];
        for (lhs, rhs, expected) in cases {
            let actual = lhs * rhs;
            assert_eq!(actual, expected);
        }
    }
}
