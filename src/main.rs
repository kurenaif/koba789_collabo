mod gfpoly;

use aes::{Aes256, Block};
use aes::cipher::{ BlockEncrypt, NewBlockCipher, generic_array::GenericArray, };

fn padding_slice(bytes: &mut Vec<u8>) -> Vec<u128> {
    let cnt = 16 - (bytes.len() % 16);
    if cnt != 16 {
        for _ in 0..cnt {
            bytes.push(0);
        }
    }


    let mut res: Vec<u128> = vec![];
    for slice in bytes.chunks(16) {
        let temp = slice.try_into().unwrap();
        res.push(u128::from_be_bytes(temp));
    }

    res
}

fn ghash(h: u128, a: &[u8], c:&[u8]) -> u128{
    let len_a: u128 = u128::try_from(a.len()).unwrap() * 8u128; // unit: bits
    let len_c: u128 = u128::try_from(c.len()).unwrap() * 8u128; // unit: bits

    let a = padding_slice(&mut a.to_vec());
    let a: Vec<_> = a.iter().map(|x| gfpoly::GFPoly::from(*x)).collect();

    let h = gfpoly::GFPoly::from(h);

    let c = padding_slice(&mut c.to_vec());
    let c: Vec<_> = c.iter().map(|x| gfpoly::GFPoly::from(*x)).collect();

    let m = a.len();
    let n = c.len();
    let mut x = vec![gfpoly::GFPoly::from(0u128); m+n+2];

    for i in 1..(m+1) {
        x[i] = (x[i-1] + a[i-1]) * h;
    }

    for i in 1..(n+1) {
        x[m+i] = (x[m+i-1] + c[i-1]) * h;
    }

    let poly: u128 = ((len_a << 64) + len_c).try_into().unwrap();
    ((x[m+n] + gfpoly::GFPoly::from(poly).into()) * h).into()
}

fn block_encrypt(k: &[u8; 32], msg: &u128) -> u128 {
    let key = GenericArray::from_slice(k);
    let mut block = *Block::from_slice(&msg.to_be_bytes());

    let cipher = Aes256::new(key);

    cipher.encrypt_block(&mut block);
    let a = block;
    let temp: [u8; 16] = a.as_slice().try_into().unwrap();
    u128::from_be_bytes(temp)
}

fn incr(y: u128) -> u128 {
    // split Y[12], Y[4]
    let f = y & (0xffffffffffffffffffffffffffffffffu128 - 0xffffffffu128);
    let mut i = y & 0xffffffffu128; 
    i = (i + 1) & 0xffffffffu128;
    f + i
}

fn bytes_xor<'a>(lhs: &'a[u8], rhs: &'a[u8]) -> Option<Vec<u8>>{
    if lhs.len() != rhs.len() {
        return None;
    }

    let res: Vec<_> = lhs.iter().zip(rhs.iter()).map(|(x,y)| {
        x^y
    }).collect();
    Some(res)
}

fn encrypt<'a>(p: &[u8], k: &[u8; 32], iv: &[u8], a: &[u8]) -> (Vec<u8>, [u8; 16]) {
    let h = block_encrypt(k, &u128::from_be_bytes(*b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00"));
    let p: Vec<_> = p.chunks(16).collect();
    let n = p.len();

    let mut y = vec!(0u128; n+1);
    if iv.len() == 12 {
        let temp = [iv, b"\x00\x00\x00\x01"].concat();
        let temp: [u8; 16] = temp.try_into().unwrap();
        y[0] = u128::from_be_bytes(temp);
    } else {
        y[0] = ghash(h, b"", iv);
    }

    for i in 1..(n+1) {
        y[i] = incr(y[i-1]);
    }

    let mut c = Vec::new();
    for i in 0..(n-1) {
        println!("y[{}] = {:x}",i+1, y[i+1]);
        c.push(
            bytes_xor(p[i], &block_encrypt(k, &y[i+1]).to_be_bytes()).unwrap()
        );
    }
    let u = p[n-1].len();
    c.push(
        bytes_xor(p[n-1], &block_encrypt(k, &y[n]).to_be_bytes()[..u]).unwrap()
    );
    let c = c.concat();

    println!("H: {:x}", &h);
    println!("A: {:?}", a);
    println!("C: {:?}", c);
    println!("ghash: {:x}", &ghash(h, a, c.as_slice()));
    println!("block_encrypt: {:x}", &block_encrypt(k, &y[0]));

    let t: [u8; 16] = bytes_xor(
        &ghash(h, a, c.as_slice()).to_be_bytes(), &block_encrypt(k, &y[0]).to_be_bytes()
    ).unwrap().try_into().unwrap();

    (c, t)
}

fn main() {
//    // 0xfeffe9928665731c6d6a8f9467308308feffe9928665731c6d6a8f9467308308
//    let k = b"\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08";
//    // 0xfeedfacedeadbeeffeedfacedeadbeefabaddad2
//    let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
//    // 0xd9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39
//    let p = b"\xd912%\xf8\x84\x06\xe5\xa5Y\t\xc5\xaf\xf5&\x9a\x86\xa7\xa9S\x154\xf7\xda.L0=\x8a1\x8ar\x1c<\x0c\x95\x95h\tS/\xcf\x0e$I\xa6\xb5%\xb1j\xed\xf5\xaa\r\xe6W\xbac{9";
//    // 0x9313225df88406e555909c5aff5269aa6a7a9538534f7da1e4c303d2a318a728c3c0c95156809539fcf0e2429a6b525416aedbf5a0de6a57a637b39b
//    let iv = b"\x93\x13\x22]\xf8\x84\x06\xe5U\x90\x9cZ\xffRi\xaajz\x958SO}\xa1\xe4\xc3\x03\xd2\xa3\x18\xa7(\xc3\xc0\xc9QV\x80\x959\xfc\xf0\xe2B\x9akRT\x16\xae\xdb\xf5\xa0\xdejW\xa67\xb3\x9b";
//    println!("{:?}", encrypt(p, k, iv, a));


    // let a = gfpoly::GFPoly::from((1u128<<126) + (1u128<<127));
    // let b = gfpoly::GFPoly::from((1u128<<126) + (1u128<<127));
    // println!("{:?}", a * b);

    // let a = gfpoly::GFPoly::from(1);
    // let b = gfpoly::GFPoly::from(1u128<<126);
    // println!("{:?}", a * b);

    // let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
    // let h = b"\xb8;S7\x08\xbfS]\n\xa6\xe5)\x80\xd5;x";
    // let c = b"B\x83\x1e\xc2!wt$Kr!\xb7\x84\xd0\xd4\x9c\xe3\xaa!/,\x02\xa4\xe05\xc1~#)\xac\xa1.!\xd5\x14\xb2Tf\x93\x1c}\x8fjZ\xac\x84\xaa\x05\x1b\xa3\x0b9j\n\xac\x97=X\xe0\x91";
    // println!("{:x}", ghash(u128::from_be_bytes(*h),a,c));
}

#[cfg(test)]
mod tests {
    use crate::{encrypt};
    #[test]
    fn test_case_16() {

        let k = b"\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08";
        let p = b"\xd912%\xf8\x84\x06\xe5\xa5Y\t\xc5\xaf\xf5&\x9a\x86\xa7\xa9S\x154\xf7\xda.L0=\x8a1\x8ar\x1c<\x0c\x95\x95h\tS/\xcf\x0e$I\xa6\xb5%\xb1j\xed\xf5\xaa\r\xe6W\xbac{9";
        let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
        let iv = b"\xca\xfe\xba\xbe\xfa\xce\xdb\xad\xde\xca\xf8\x88";

        let (c, t) = encrypt(p, k, iv, a);

        let c_expect = b"R-\xc1\xf0\x99V}\x07\xf4\x7f7\xa3*\x84B}d:\x8c\xdc\xbf\xe5\xc0\xc9u\x98\xa2\xbd%U\xd1\xaa\x8c\xb0\x8eHY\r\xbb=\xa7\xb0\x8b\x10V\x82\x888\xc5\xf6\x1ec\x93\xbaz\n\xbc\xc9\xf6b";
        let t_expect = b"v\xfcn\xce\x0fN\x17h\xcd\xdf\x88S\xbb-U\x1b";


        println!("{:x}", u128::from_be_bytes(t));
        assert_eq!(c, c_expect);
        assert_eq!(&t, t_expect);
    }

    #[test]
    fn test_case_17() {

        let k = b"\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08";
        let p = b"\xd912%\xf8\x84\x06\xe5\xa5Y\t\xc5\xaf\xf5&\x9a\x86\xa7\xa9S\x154\xf7\xda.L0=\x8a1\x8ar\x1c<\x0c\x95\x95h\tS/\xcf\x0e$I\xa6\xb5%\xb1j\xed\xf5\xaa\r\xe6W\xbac{9";
        let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
        let iv = b"\xca\xfe\xba\xbe\xfa\xce\xdb\xad";

        let (c, t) = encrypt(p, k, iv, a);

        let c_expect = b"\xc3v-\xf1\xcax}2\xaeG\xc1;\xf1\x98D\xcb\xaf\x1a\xe1M\x0b\x97j\xfa\xc5/\xf7\xd7\x9b\xba\x9d\xe0\xfe\xb5\x82\xd394\xa4\xf0\x95L\xc26;\xc7?xb\xacC\x0ed\xab\xe4\x99\xf4|\x9b\x1f";
        let t_expect = b":3}\xbfF\xa7\x92\xc4^EI\x13\xfe.\xa8\xf2";

        println!("{:x}", u128::from_be_bytes(t));
        assert_eq!(c, c_expect);
        assert_eq!(&t, t_expect);
    }

    #[test]
    fn test_case_18() {

        let k = b"\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08";
        let p = b"\xd912%\xf8\x84\x06\xe5\xa5Y\t\xc5\xaf\xf5&\x9a\x86\xa7\xa9S\x154\xf7\xda.L0=\x8a1\x8ar\x1c<\x0c\x95\x95h\tS/\xcf\x0e$I\xa6\xb5%\xb1j\xed\xf5\xaa\r\xe6W\xbac{9";
        let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
        let iv = b"\x93\x13\x22]\xf8\x84\x06\xe5U\x90\x9cZ\xffRi\xaajz\x958SO}\xa1\xe4\xc3\x03\xd2\xa3\x18\xa7(\xc3\xc0\xc9QV\x80\x959\xfc\xf0\xe2B\x9akRT\x16\xae\xdb\xf5\xa0\xdejW\xa67\xb3\x9b";


        let (c, t) = encrypt(p, k, iv, a);

        let c_expect = b"Z\x8d\xef/\x0c\x9eS\xf1\xf7]xSe\x9e* \xee\xb2\xb2*\xaf\xded\x19\xa0X\xabOotk\xf4\x0f\xc0\xc3\xb7\x80\xf2DE-\xa3\xeb\xf1\xc5\xd8,\xde\xa2A\x89\x97 \x0e\xf8.D\xae~?";
        let t_expect = b"\xa4J\x82f\xee\x1c\x8e\xb0\xc8\xb5\xd4\xcfZ\xe9\xf1\x9a";

        println!("{:x}", u128::from_be_bytes(t));
        assert_eq!(c, c_expect);
        assert_eq!(&t, t_expect);
    }
}


