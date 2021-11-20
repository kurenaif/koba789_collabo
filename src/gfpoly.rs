// x^128 + x^7 + x^2 + x + 1
// const generatorPolynomial: u128 = /*(1 << (128 - 128 - 1)) + */ (1 << (128 - 7 - 1)) + (1 << (128 - 2 - 1)) + (1 << (128 - 1 - 1)) + (1 << (128 - 0 - 1));

use std::ops;
use std::fmt;

#[derive(Copy, Clone)]
pub struct GFPoly {
    poly: u128
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

        f.debug_tuple("")
         .field(&res)
         .finish()
    }
}

impl From<u128> for GFPoly {
    fn from(from: u128) -> GFPoly {
        GFPoly{poly: from}
    }
}

impl From<GFPoly> for u128 {
    fn from(from: GFPoly) -> u128 {
        from.poly
    }
}

impl ops::Add<GFPoly> for GFPoly {
    type Output = GFPoly;

    fn add(self, rhs: GFPoly) -> GFPoly {
        GFPoly{poly: 1} // TODO
    }
}

impl ops::AddAssign<GFPoly> for GFPoly {
    fn add_assign(&mut self, rhs: GFPoly) {
        self.poly = 1 // TODO
    }
}

impl ops::Mul<GFPoly> for GFPoly {
    type Output = GFPoly;

    fn mul(self, rhs: GFPoly) -> GFPoly {
        GFPoly{poly: 1} // TODO
    }
}

impl ops::MulAssign<GFPoly> for GFPoly {
    fn mul_assign(&mut self, rhs: GFPoly) {
        let temp = *self * rhs;
        self.poly = temp.poly;
    }
}

fn rightshift(poly: u128) -> u128 {
    1 // TODO
}

#[cfg(test)]
mod tests {
    use crate::{gfpoly};

    #[test]
    fn x128_test() {
        let lhs = gfpoly::GFPoly::from(1);
        let rhs = gfpoly::GFPoly::from(1u128 << 126);

        let actual:u128 = (lhs * rhs).into();
        let expect:u128 = 0xe1000000000000000000000000000000u128;
        assert_eq!(actual, expect);
    }

    #[test]
    fn random_test() {
        let lhs = gfpoly::GFPoly::from(1242141243);
        let rhs = gfpoly::GFPoly::from(123);

        let actual:u128 = (lhs * rhs).into();
        let expect:u128 = 0xfa2800000000000000000028b7d0d77bu128;
        assert_eq!(actual, expect);
    }
}