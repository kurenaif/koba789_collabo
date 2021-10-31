mod gfpoly;

fn slice(bytes: Vec<u8>) -> Vec<[u8;16]> {
    let mut bytes = bytes;
    let cnt = 16 - (bytes.len() % 16);
    if cnt != 16 {
        for _ in 0..cnt {
            bytes.push(0);
        }
    }


    let mut res: Vec<[u8;16]> = vec![];
    for slice in bytes.chunks(16) {
        let temp = slice.try_into();
        res.push(temp.unwrap());
    }

    res
}

fn ghash(H: Vec<u8>, A: Vec<u8>, C: Vec<u8>) {
    println!("{:?}", slice(A));
    println!("{:?}", slice(C));
    println!("{:?}", slice(H));
}

fn main() {
    let poly = gfpoly::GFPoly::from(1u128);
    let poly2 = gfpoly::GFPoly::from(2u128);

    let bytes: Vec<u8> = vec![1, 2, 3];
    let A = b"\xfe\xed\xfa\xce\xde\xad\xbe\xef\xfe\xed\xfa\xce\xde\xad\xbe\xef\xab\xad\xda\xd2";
    let H = b"\xb8;S7\x08\xbfS]\n\xa6\xe5)\x80\xd5;x";
    let C = b"B\x83\x1e\xc2!wt$Kr!\xb7\x84\xd0\xd4\x9c\xe3\xaa!/,\x02\xa4\xe05\xc1~#)\xac\xa1.!\xd5\x14\xb2Tf\x93\x1c}\x8fjZ\xac\x84\xaa\x05\x1b\xa3\x0b9j\n\xac\x97=X\xe0\x91";
    ghash(H.to_vec(), A.to_vec(), C.to_vec());
}