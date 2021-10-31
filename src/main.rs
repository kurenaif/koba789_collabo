mod gfpoly;

fn main() {
    let poly = gfpoly::GFPoly::from(1u128);
    let poly2 = gfpoly::GFPoly::from(2u128);
    println!("{:?}", poly * poly2);
}