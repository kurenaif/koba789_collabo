// x^128 + x^7 + x^2 + x + 1
const generatorPolynomial: u128 = /*(1 << (128 - 128 - 1)) + */ (1 << (128 - 7 - 1)) + (1 << (128 - 2 - 1)) + (1 << (128 - 1 - 1)) + (1 << (128 - 0 - 1));

use std::ops;
use std::fmt;

pub struct GFPoly {
    poly: u128
}

// impl GFPoly {
//     fn new(bytes: &[u8]) -> Self {
//         let boost = GFPoly::from(1);
//         const eight: GFPoly = GFPoly::from(8);
//         let res = GFPoly::from(0);
// 
//         for byte in bytes {
//             res += byte * boost;
//         }
//     }
// }

impl fmt::Debug for GFPoly {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut res = "".to_string();

        for i in 0..128 {
            if (self.poly >> i) & 1 == 1 {
                if res != "" {
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

impl ops::Add<GFPoly> for GFPoly {
    type Output = GFPoly;

    fn add(self, rhs: GFPoly) -> GFPoly {
        GFPoly{ poly: self.poly ^ rhs.poly }
    }
}

impl ops::AddAssign<GFPoly> for GFPoly {
    fn add_assign(&mut self, rhs: GFPoly) {
        self.poly ^= rhs.poly
    }
}

impl ops::Mul<GFPoly> for GFPoly {
    type Output = GFPoly;

    fn mul(self, rhs: GFPoly) -> GFPoly {
        let mut rhspoly = rhs.poly;
        let mut res = 0u128;

        for i in (0..128).rev() {
            if (self.poly >> i) & 1 == 1 { 
                res += rhspoly;
            }
            rhspoly = rightshift(rhspoly);
        }
 
        GFPoly{
            poly: res
        }
    }
}

// impl ops::MulAssign<GFPoly> for GFPoly {
//     fn mul_assign(&mut self, rhs: GFPoly) {
//         self = &mut(*self * rhs);
//     }
// }

fn rightshift(poly: u128) -> u128 {
    let add = if poly & 1 == 1 {
        0x91800000000000000000000000000000
    } else {
        0
    };
    (poly >> 1) + add
}