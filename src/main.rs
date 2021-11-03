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
        x[i] = x[i-1] + a[i-1] * h;
    }

    for i in 1..(n+1) {
        x[m+i] = (x[m+i-1] + c[i-1]) * h;
    }

    let res: u128 = ((len_a << 64) + len_c).try_into().unwrap();
    gfpoly::GFPoly::from(res).into()
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
    let f = y & (0xffffffffffffffffffffffffffffffffu128 - 0xffffffffffffffffffffffff00000000u128);
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
        c.push(
            bytes_xor(p[i], &block_encrypt(k, &y[i+1]).to_be_bytes()).unwrap()
        );
    }
    let u = p[n-1].len();
    c.push(
        bytes_xor(p[n-1], &block_encrypt(k, &y[n]).to_be_bytes()[..u]).unwrap()
    );
    let c = c.concat();

    let t: [u8; 16] = bytes_xor(
        &ghash(h, a, c.as_slice()).to_be_bytes(), &block_encrypt(k, &y[0]).to_be_bytes()
    ).unwrap().try_into().unwrap();

    (c, t)
}

fn main() {
    // 0xfeffe9928665731c6d6a8f9467308308feffe9928665731c6d6a8f9467308308
    let k = b"\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08\xfe\xff\xe9\x92\x86es\x1cmj\x8f\x94g0\x83\x08";
    // 0xfeedfacedeadbeeffeedfacedeadbeefabaddad2
    let a = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
    // 0xd9313225f88406e5a55909c5aff5269a86a7a9531534f7da2e4c303d8a318a721c3c0c95956809532fcf0e2449a6b525b16aedf5aa0de657ba637b39
    let p = b"\xd912%\xf8\x84\x06\xe5\xa5Y\t\xc5\xaf\xf5&\x9a\x86\xa7\xa9S\x154\xf7\xda.L0=\x8a1\x8ar\x1c<\x0c\x95\x95h\tS/\xcf\x0e$I\xa6\xb5%\xb1j\xed\xf5\xaa\r\xe6W\xbac{9";
    // 0x9313225df88406e555909c5aff5269aa6a7a9538534f7da1e4c303d2a318a728c3c0c95156809539fcf0e2429a6b525416aedbf5a0de6a57a637b39b
    let iv = b"\x93\x13\x22]\xf8\x84\x06\xe5U\x90\x9cZ\xffRi\xaajz\x958SO}\xa1\xe4\xc3\x03\xd2\xa3\x18\xa7(\xc3\xc0\xc9QV\x80\x959\xfc\xf0\xe2B\x9akRT\x16\xae\xdb\xf5\xa0\xdejW\xa67\xb3\x9b";
    encrypt(p, k, iv, a);
}