// RC algorithm
// https://en.wikipedia.org/wiki/RC_algorithm
// 
// The RC algorithms are a set of symmetric-key encryption algorithms invented by Ron Rivest. 
// The "RC" may stand for either Rivest's cipher or, more informally, Ron's code.[1] 
// Despite the similarity in their names, the algorithms are for the most part unrelated. 
// There have been six RC algorithms so far:
// 
//     RC1 was never published.
//     RC2 was a 64-bit block cipher developed in 1987.
//     RC3 was broken before ever being used.
//     RC4 is a stream cipher.
//     RC5 is a 32/64/128-bit block cipher developed in 1994.
//     RC6, a 128-bit block cipher based heavily on RC5, was an AES finalist developed in 1997.
// 
const PI_TABLE: [u8; 256] = [
    0xd9, 0x78, 0xf9, 0xc4, 0x19, 0xdd, 0xb5, 0xed, 
    0x28, 0xe9, 0xfd, 0x79, 0x4a, 0xa0, 0xd8, 0x9d, 
    0xc6, 0x7e, 0x37, 0x83, 0x2b, 0x76, 0x53, 0x8e, 
    0x62, 0x4c, 0x64, 0x88, 0x44, 0x8b, 0xfb, 0xa2, 
    0x17, 0x9a, 0x59, 0xf5, 0x87, 0xb3, 0x4f, 0x13, 
    0x61, 0x45, 0x6d, 0x8d, 0x09, 0x81, 0x7d, 0x32, 
    0xbd, 0x8f, 0x40, 0xeb, 0x86, 0xb7, 0x7b, 0x0b, 
    0xf0, 0x95, 0x21, 0x22, 0x5c, 0x6b, 0x4e, 0x82, 
    0x54, 0xd6, 0x65, 0x93, 0xce, 0x60, 0xb2, 0x1c, 
    0x73, 0x56, 0xc0, 0x14, 0xa7, 0x8c, 0xf1, 0xdc, 
    0x12, 0x75, 0xca, 0x1f, 0x3b, 0xbe, 0xe4, 0xd1, 
    0x42, 0x3d, 0xd4, 0x30, 0xa3, 0x3c, 0xb6, 0x26, 
    0x6f, 0xbf, 0x0e, 0xda, 0x46, 0x69, 0x07, 0x57, 
    0x27, 0xf2, 0x1d, 0x9b, 0xbc, 0x94, 0x43, 0x03, 
    0xf8, 0x11, 0xc7, 0xf6, 0x90, 0xef, 0x3e, 0xe7, 
    0x06, 0xc3, 0xd5, 0x2f, 0xc8, 0x66, 0x1e, 0xd7, 
    0x08, 0xe8, 0xea, 0xde, 0x80, 0x52, 0xee, 0xf7, 
    0x84, 0xaa, 0x72, 0xac, 0x35, 0x4d, 0x6a, 0x2a, 
    0x96, 0x1a, 0xd2, 0x71, 0x5a, 0x15, 0x49, 0x74, 
    0x4b, 0x9f, 0xd0, 0x5e, 0x04, 0x18, 0xa4, 0xec, 
    0xc2, 0xe0, 0x41, 0x6e, 0x0f, 0x51, 0xcb, 0xcc, 
    0x24, 0x91, 0xaf, 0x50, 0xa1, 0xf4, 0x70, 0x39, 
    0x99, 0x7c, 0x3a, 0x85, 0x23, 0xb8, 0xb4, 0x7a, 
    0xfc, 0x02, 0x36, 0x5b, 0x25, 0x55, 0x97, 0x31, 
    0x2d, 0x5d, 0xfa, 0x98, 0xe3, 0x8a, 0x92, 0xae, 
    0x05, 0xdf, 0x29, 0x10, 0x67, 0x6c, 0xba, 0xc9, 
    0xd3, 0x00, 0xe6, 0xcf, 0xe1, 0x9e, 0xa8, 0x2c, 
    0x63, 0x16, 0x01, 0x3f, 0x58, 0xe2, 0x89, 0xa9, 
    0x0d, 0x38, 0x34, 0x1b, 0xab, 0x33, 0xff, 0xb0, 
    0xbb, 0x48, 0x0c, 0x5f, 0xb9, 0xb1, 0xcd, 0x2e, 
    0xc5, 0xf3, 0xdb, 0x47, 0xe5, 0xa5, 0x9c, 0x77, 
    0x0a, 0xa6, 0x20, 0x68, 0xfe, 0x7f, 0xc1, 0xad, 
];

#[inline]
fn key_expansion(key: &[u8]) -> [u16; 64] {
    const MIN_KEY_LEN: usize =   1;
    const MAX_KEY_LEN: usize = 128;

    let key_len = key.len();
    let t1 = key.len() * 8;      // KEY-LEN in bits
    assert!(t1 >= MIN_KEY_LEN && t1 <= MAX_KEY_LEN); // 1 .. 128

    let t8: usize = (t1 + 7) >> 3;
    let tm: usize = (255 % ((2 as u32).pow((8 + t1 - 8 * t8) as u32))) as usize;

    let mut buf: [u8; 128] = [0; 128];
    buf[..key_len].copy_from_slice(&key[..key_len]);

    for i in key_len..128 {
        let pos: u32 = (u32::from(buf[i - 1]) + u32::from(buf[i - key_len])) & 0xff;
        buf[i] = PI_TABLE[pos as usize];
    }

    buf[128 - t8] = PI_TABLE[(buf[128 - t8] & tm as u8) as usize];

    for i in (0..128 - t8).rev() {
        let pos: usize = (buf[i + 1] ^ buf[i + t8]) as usize;
        buf[i] = PI_TABLE[pos];
    }

    let mut ek: [u16; 64] = [0; 64];
    
    for i in 0..64 {
        ek[i] = (u16::from(buf[2 * i + 1]) << 8) + u16::from(buf[2 * i])
    }

    ek
}

#[inline]
fn mix(ek: &[u16; 64], r: &mut [u16; 4], j: &mut usize) {
    r[0] = r[0]
        .wrapping_add(ek[*j])
        .wrapping_add(r[3] & r[2])
        .wrapping_add(!r[3] & r[1]);
    *j += 1;
    r[0] = (r[0] << 1) | (r[0] >> 15);

    r[1] = r[1]
        .wrapping_add(ek[*j])
        .wrapping_add(r[0] & r[3])
        .wrapping_add(!r[0] & r[2]);
    *j += 1;
    r[1] = (r[1] << 2) | (r[1] >> 14);

    r[2] = r[2]
        .wrapping_add(ek[*j])
        .wrapping_add(r[1] & r[0])
        .wrapping_add(!r[1] & r[3]);
    *j += 1;
    r[2] = (r[2] << 3) | (r[2] >> 13);

    r[3] = r[3]
        .wrapping_add(ek[*j])
        .wrapping_add(r[2] & r[1])
        .wrapping_add(!r[2] & r[0]);
    *j += 1;
    r[3] = (r[3] << 5) | (r[3] >> 11);
}

#[inline]
fn reverse_mix(ek: &[u16; 64], r: &mut [u16; 4], j: &mut usize) {
    r[3] = (r[3] << 11) | (r[3] >> 5);
    r[3] = r[3]
        .wrapping_sub(ek[*j])
        .wrapping_sub(r[2] & r[1])
        .wrapping_sub(!r[2] & r[0]);
    *j -= 1;

    r[2] = (r[2] << 13) | (r[2] >> 3);
    r[2] = r[2]
        .wrapping_sub(ek[*j])
        .wrapping_sub(r[1] & r[0])
        .wrapping_sub(!r[1] & r[3]);
    *j -= 1;

    r[1] = (r[1] << 14) | (r[1] >> 2);
    r[1] = r[1]
        .wrapping_sub(ek[*j])
        .wrapping_sub(r[0] & r[3])
        .wrapping_sub(!r[0] & r[2]);
    *j -= 1;

    r[0] = (r[0] << 15) | (r[0] >> 1);
    r[0] = r[0]
        .wrapping_sub(ek[*j])
        .wrapping_sub(r[3] & r[2])
        .wrapping_sub(!r[3] & r[1]);
    *j = j.wrapping_sub(1);
}

#[inline]
fn mash(ek: &[u16; 64], r: &mut [u16; 4]) {
    r[0] = r[0].wrapping_add(ek[(r[3] & 63) as usize]);
    r[1] = r[1].wrapping_add(ek[(r[0] & 63) as usize]);
    r[2] = r[2].wrapping_add(ek[(r[1] & 63) as usize]);
    r[3] = r[3].wrapping_add(ek[(r[2] & 63) as usize]);
}

#[inline]
fn reverse_mash(ek: &[u16; 64], r: &mut [u16; 4]) {
    r[3] = r[3].wrapping_sub(ek[(r[2] & 63) as usize]);
    r[2] = r[2].wrapping_sub(ek[(r[1] & 63) as usize]);
    r[1] = r[1].wrapping_sub(ek[(r[0] & 63) as usize]);
    r[0] = r[0].wrapping_sub(ek[(r[3] & 63) as usize]);
}


// RC2-KEYLEN128-BLOCKLEN128
#[derive(Clone)]
pub struct Rc2K128B128 {
    inner: Rc2,
}

impl Rc2K128B128 {
    pub const KEY_LEN: usize   = 16;
    pub const BLOCK_LEN: usize = 16;

    pub fn new(key: &[u8]) -> Self {
        assert_eq!(key.len(), Self::KEY_LEN);
        
        let inner = Rc2::new(key);

        Self { inner }
    }

    pub fn encrypt(&self, block: &mut [u8]) {
        self.inner.encrypt_two_blocks(block);
    }

    pub fn decrypt(&self, block: &mut [u8]) {
        self.inner.decrypt_two_blocks(block);
    }
}

impl std::fmt::Debug for Rc2K128B128 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ek = &self.inner.ek[..];
        f.debug_struct("Rc2K128B128")
            .field("ek", &ek)
            .finish()
    }
}


// A Description of the RC2(r) Encryption Algorithm (RC2 (also known as ARC2))
// https://tools.ietf.org/html/rfc2268
// https://en.wikipedia.org/wiki/RC2
#[derive(Clone)]
pub struct Rc2 {
    ek: [u16; 64],
}

impl Rc2 {
    pub const BLOCK_LEN: usize   =  8; // In bytes
    pub const MIN_KEY_LEN: usize =  1; // In bytes
    pub const MAX_KEY_LEN: usize = 16; // In bytes

    // Key len: in bytes
    pub fn new(key: &[u8]) -> Self {
        let ek = key_expansion(key);
        Self { ek }
    }

    pub fn encrypt(&self, block: &mut [u8]) {
        debug_assert_eq!(block.len(), Self::BLOCK_LEN);

        let mut r: [u16; 4] = [
            (u16::from(block[1]) << 8) + u16::from(block[0]),
            (u16::from(block[3]) << 8) + u16::from(block[2]),
            (u16::from(block[5]) << 8) + u16::from(block[4]),
            (u16::from(block[7]) << 8) + u16::from(block[6]),
        ];

        let mut j = 0;
        for i in 0..16 {
            mix(&self.ek, &mut r, &mut j);
            if i == 4 || i == 10 {
                mash(&self.ek, &mut r);
            }
        }

        block[0] = (r[0] & 0xff) as u8;
        block[1] = (r[0] >> 8) as u8;
        block[2] = (r[1] & 0xff) as u8;
        block[3] = (r[1] >> 8) as u8;
        block[4] = (r[2] & 0xff) as u8;
        block[5] = (r[2] >> 8) as u8;
        block[6] = (r[3] & 0xff) as u8;
        block[7] = (r[3] >> 8) as u8;
    }

    pub fn decrypt(&self, block: &mut [u8]) {
        debug_assert_eq!(block.len(), Self::BLOCK_LEN);

        let mut r: [u16; 4] = [
            (u16::from(block[1]) << 8) + u16::from(block[0]),
            (u16::from(block[3]) << 8) + u16::from(block[2]),
            (u16::from(block[5]) << 8) + u16::from(block[4]),
            (u16::from(block[7]) << 8) + u16::from(block[6]),
        ];

        let mut j = 63;
        for i in 0..16 {
            reverse_mix(&self.ek, &mut r, &mut j);
            if i == 4 || i == 10 {
                reverse_mash(&self.ek, &mut r);
            }
        }

        block[0] = r[0] as u8;
        block[1] = (r[0] >> 8) as u8;
        block[2] = r[1] as u8;
        block[3] = (r[1] >> 8) as u8;
        block[4] = r[2] as u8;
        block[5] = (r[2] >> 8) as u8;
        block[6] = r[3] as u8;
        block[7] = (r[3] >> 8) as u8;
    }

    // NOTE: 
    //       使块大小变成 16 bytes，跟主流的对称分组密码一样。
    pub fn encrypt_two_blocks(&self, blocks: &mut [u8]) {
        debug_assert_eq!(blocks.len(), Self::BLOCK_LEN * 2);

        self.encrypt(&mut blocks[0.. 8]);
        self.encrypt(&mut blocks[8..16]);
    }

    pub fn decrypt_two_blocks(&self, blocks: &mut [u8]) {
        debug_assert_eq!(blocks.len(), Self::BLOCK_LEN * 2);

        self.decrypt(&mut blocks[0.. 8]);
        self.decrypt(&mut blocks[8..16]);
    }
}

impl std::fmt::Debug for Rc2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let ek = &self.ek[..];
        f.debug_struct("Rc2")
            .field("ek", &ek)
            .finish()
    }
}
