// Advanced Encryption Standard (AES) (FIPS PUB 197)
// https://csrc.nist.gov/csrc/media/publications/fips/197/final/documents/fips-197.pdf
// 
// For the AES algorithm, the length of the Cipher Key, K, is 128, 192, or 256 bits.
// The key length is represented by Nk = 4, 6, or 8, 
// which reflects the number of 32-bit words (number of columns) in the Cipher Key.
// 
// For the AES algorithm, the number of rounds to be performed during the execution of 
// the algorithm is dependent on the key size. 
// The number of rounds is represented by Nr, where Nr = 10 when Nk = 4, 
// Nr = 12 when Nk = 6, and Nr = 14 when Nk = 8. 
// 
// The only Key-Block-Round combinations that conform to this standard are given in Fig. 4. 
// For implementation issues relating to the key length, block size and number of rounds, 
// see Sec. 6.3. 
// 
// Figure 4. Key-Block-Round Combinations. 
//             Key Length (Nk words)    Block Size (Nb words)    Number of Rounds (Nr) 
// AES-128         4                       4                           10 
// AES-192         6                       4                           12 
// AES-256         8                       4                           14 
// 
// 
// The Rijndael Animation
// http://www.formaestudio.com/rijndaelinspector/
// 

#[cfg(test)]
use hex;

pub const WORD_SIZE: usize      =  4; // Word(u32) size in bytes

pub const AES_BLOCK_LEN: usize  = 16; // Block Size (bytes)
pub const AES_NB: usize         = AES_BLOCK_LEN  / WORD_SIZE; //  4, Block Size (Nb words)

pub const AES128_KEY_LEN: usize = 16; // Key Length (bytes)
pub const AES192_KEY_LEN: usize = 24;
pub const AES256_KEY_LEN: usize = 32;

pub const AES128_NK: usize  = AES128_KEY_LEN / WORD_SIZE; //  4, Key Length (Nk words)
pub const AES128_NR: usize  = 10;                         // 10, Number of Rounds (Nr) 

pub const AES192_NK: usize  = AES192_KEY_LEN / WORD_SIZE; //  6, Key Length (Nk words)
pub const AES192_NR: usize  = 12;                         // 12, Number of Rounds (Nr) 

pub const AES256_NK: usize  = AES256_KEY_LEN / WORD_SIZE; //  8, Key Length (Nk words)
pub const AES256_NR: usize  = 14;                         // 14, Number of Rounds (Nr) 


macro_rules! impl_aes {
    ($name:ident, $nr:ident, $name_s:tt) => {
        #[derive(Clone, Copy)]
        pub struct $name {
            pub ek: [u8; ($nr + 1) * AES_BLOCK_LEN],
        }

        impl $name {
            pub fn new(key: &[u8]) -> Self {
                let mut ek = [0u8; ($nr + 1) * AES_BLOCK_LEN];
                key_expansion(key, &mut ek);
                Self { ek }
            }

            pub fn encrypt(&self, input: &[u8]) -> [u8; 16] {
                let mut state = [0u8; 16];
                state.copy_from_slice(input);

                encrypt(&mut state, &self.ek, $nr);

                state
            }

            pub fn decrypt(&self, input: &[u8]) -> [u8; 16] {
                let mut state = [0u8; 16];
                state.copy_from_slice(input);
                
                decrypt(&mut state, &self.ek, $nr);
                
                state
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let ek = &self.ek[..];
                f.debug_struct($name_s)
                    .field("ek", &ek)
                    .finish()
            }
        }
    }
}

impl_aes!(ExpandedKey128, AES128_NR, "ExpandedKey128");
impl_aes!(ExpandedKey192, AES192_NR, "ExpandedKey192");
impl_aes!(ExpandedKey256, AES256_NR, "ExpandedKey256");


// The round constant word array. 
pub const RCON: [u32; 10] = [0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x1b, 0x36];

// aes sbox and invert-sbox
// Forward S-Box
pub const FORWARD_S_BOX: [u8; 256] = [
    // 0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
    0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab, 0x76,
    0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4, 0x72, 0xc0,
    0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71, 0xd8, 0x31, 0x15,
    0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2, 0xeb, 0x27, 0xb2, 0x75,
    0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6, 0xb3, 0x29, 0xe3, 0x2f, 0x84,
    0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb, 0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf,
    0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45, 0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8,
    0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5, 0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2,
    0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44, 0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73,
    0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a, 0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb,
    0xe0, 0x32, 0x3a, 0x0a, 0x49, 0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79,
    0xe7, 0xc8, 0x37, 0x6d, 0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08,
    0xba, 0x78, 0x25, 0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a,
    0x70, 0x3e, 0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e,
    0xe1, 0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
    0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb, 0x16,
];

// Reverse S-Box
pub const REVERSE_S_BOX: [u8; 256] = [
    // 0     1     2     3     4     5     6     7     8     9     A     B     C     D     E     F
    0x52, 0x09, 0x6a, 0xd5, 0x30, 0x36, 0xa5, 0x38, 0xbf, 0x40, 0xa3, 0x9e, 0x81, 0xf3, 0xd7, 0xfb,
    0x7c, 0xe3, 0x39, 0x82, 0x9b, 0x2f, 0xff, 0x87, 0x34, 0x8e, 0x43, 0x44, 0xc4, 0xde, 0xe9, 0xcb,
    0x54, 0x7b, 0x94, 0x32, 0xa6, 0xc2, 0x23, 0x3d, 0xee, 0x4c, 0x95, 0x0b, 0x42, 0xfa, 0xc3, 0x4e,
    0x08, 0x2e, 0xa1, 0x66, 0x28, 0xd9, 0x24, 0xb2, 0x76, 0x5b, 0xa2, 0x49, 0x6d, 0x8b, 0xd1, 0x25,
    0x72, 0xf8, 0xf6, 0x64, 0x86, 0x68, 0x98, 0x16, 0xd4, 0xa4, 0x5c, 0xcc, 0x5d, 0x65, 0xb6, 0x92,
    0x6c, 0x70, 0x48, 0x50, 0xfd, 0xed, 0xb9, 0xda, 0x5e, 0x15, 0x46, 0x57, 0xa7, 0x8d, 0x9d, 0x84,
    0x90, 0xd8, 0xab, 0x00, 0x8c, 0xbc, 0xd3, 0x0a, 0xf7, 0xe4, 0x58, 0x05, 0xb8, 0xb3, 0x45, 0x06,
    0xd0, 0x2c, 0x1e, 0x8f, 0xca, 0x3f, 0x0f, 0x02, 0xc1, 0xaf, 0xbd, 0x03, 0x01, 0x13, 0x8a, 0x6b,
    0x3a, 0x91, 0x11, 0x41, 0x4f, 0x67, 0xdc, 0xea, 0x97, 0xf2, 0xcf, 0xce, 0xf0, 0xb4, 0xe6, 0x73,
    0x96, 0xac, 0x74, 0x22, 0xe7, 0xad, 0x35, 0x85, 0xe2, 0xf9, 0x37, 0xe8, 0x1c, 0x75, 0xdf, 0x6e,
    0x47, 0xf1, 0x1a, 0x71, 0x1d, 0x29, 0xc5, 0x89, 0x6f, 0xb7, 0x62, 0x0e, 0xaa, 0x18, 0xbe, 0x1b,
    0xfc, 0x56, 0x3e, 0x4b, 0xc6, 0xd2, 0x79, 0x20, 0x9a, 0xdb, 0xc0, 0xfe, 0x78, 0xcd, 0x5a, 0xf4,
    0x1f, 0xdd, 0xa8, 0x33, 0x88, 0x07, 0xc7, 0x31, 0xb1, 0x12, 0x10, 0x59, 0x27, 0x80, 0xec, 0x5f,
    0x60, 0x51, 0x7f, 0xa9, 0x19, 0xb5, 0x4a, 0x0d, 0x2d, 0xe5, 0x7a, 0x9f, 0x93, 0xc9, 0x9c, 0xef,
    0xa0, 0xe0, 0x3b, 0x4d, 0xae, 0x2a, 0xf5, 0xb0, 0xc8, 0xeb, 0xbb, 0x3c, 0x83, 0x53, 0x99, 0x61,
    0x17, 0x2b, 0x04, 0x7e, 0xba, 0x77, 0xd6, 0x26, 0xe1, 0x69, 0x14, 0x63, 0x55, 0x21, 0x0c, 0x7d,
];

pub const GF_MUL2: [u8; 256] = [
    0x00, 0x02, 0x04, 0x06, 0x08, 0x0a, 0x0c, 0x0e, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1a, 0x1c, 0x1e, 
    0x20, 0x22, 0x24, 0x26, 0x28, 0x2a, 0x2c, 0x2e, 0x30, 0x32, 0x34, 0x36, 0x38, 0x3a, 0x3c, 0x3e, 
    0x40, 0x42, 0x44, 0x46, 0x48, 0x4a, 0x4c, 0x4e, 0x50, 0x52, 0x54, 0x56, 0x58, 0x5a, 0x5c, 0x5e, 
    0x60, 0x62, 0x64, 0x66, 0x68, 0x6a, 0x6c, 0x6e, 0x70, 0x72, 0x74, 0x76, 0x78, 0x7a, 0x7c, 0x7e, 
    0x80, 0x82, 0x84, 0x86, 0x88, 0x8a, 0x8c, 0x8e, 0x90, 0x92, 0x94, 0x96, 0x98, 0x9a, 0x9c, 0x9e, 
    0xa0, 0xa2, 0xa4, 0xa6, 0xa8, 0xaa, 0xac, 0xae, 0xb0, 0xb2, 0xb4, 0xb6, 0xb8, 0xba, 0xbc, 0xbe, 
    0xc0, 0xc2, 0xc4, 0xc6, 0xc8, 0xca, 0xcc, 0xce, 0xd0, 0xd2, 0xd4, 0xd6, 0xd8, 0xda, 0xdc, 0xde, 
    0xe0, 0xe2, 0xe4, 0xe6, 0xe8, 0xea, 0xec, 0xee, 0xf0, 0xf2, 0xf4, 0xf6, 0xf8, 0xfa, 0xfc, 0xfe, 
    0x1b, 0x19, 0x1f, 0x1d, 0x13, 0x11, 0x17, 0x15, 0x0b, 0x09, 0x0f, 0x0d, 0x03, 0x01, 0x07, 0x05, 
    0x3b, 0x39, 0x3f, 0x3d, 0x33, 0x31, 0x37, 0x35, 0x2b, 0x29, 0x2f, 0x2d, 0x23, 0x21, 0x27, 0x25, 
    0x5b, 0x59, 0x5f, 0x5d, 0x53, 0x51, 0x57, 0x55, 0x4b, 0x49, 0x4f, 0x4d, 0x43, 0x41, 0x47, 0x45, 
    0x7b, 0x79, 0x7f, 0x7d, 0x73, 0x71, 0x77, 0x75, 0x6b, 0x69, 0x6f, 0x6d, 0x63, 0x61, 0x67, 0x65, 
    0x9b, 0x99, 0x9f, 0x9d, 0x93, 0x91, 0x97, 0x95, 0x8b, 0x89, 0x8f, 0x8d, 0x83, 0x81, 0x87, 0x85, 
    0xbb, 0xb9, 0xbf, 0xbd, 0xb3, 0xb1, 0xb7, 0xb5, 0xab, 0xa9, 0xaf, 0xad, 0xa3, 0xa1, 0xa7, 0xa5, 
    0xdb, 0xd9, 0xdf, 0xdd, 0xd3, 0xd1, 0xd7, 0xd5, 0xcb, 0xc9, 0xcf, 0xcd, 0xc3, 0xc1, 0xc7, 0xc5, 
    0xfb, 0xf9, 0xff, 0xfd, 0xf3, 0xf1, 0xf7, 0xf5, 0xeb, 0xe9, 0xef, 0xed, 0xe3, 0xe1, 0xe7, 0xe5, 
];
pub const GF_MUL3: [u8; 256] = [
    0x00, 0x03, 0x06, 0x05, 0x0c, 0x0f, 0x0a, 0x09, 0x18, 0x1b, 0x1e, 0x1d, 0x14, 0x17, 0x12, 0x11, 
    0x30, 0x33, 0x36, 0x35, 0x3c, 0x3f, 0x3a, 0x39, 0x28, 0x2b, 0x2e, 0x2d, 0x24, 0x27, 0x22, 0x21, 
    0x60, 0x63, 0x66, 0x65, 0x6c, 0x6f, 0x6a, 0x69, 0x78, 0x7b, 0x7e, 0x7d, 0x74, 0x77, 0x72, 0x71, 
    0x50, 0x53, 0x56, 0x55, 0x5c, 0x5f, 0x5a, 0x59, 0x48, 0x4b, 0x4e, 0x4d, 0x44, 0x47, 0x42, 0x41, 
    0xc0, 0xc3, 0xc6, 0xc5, 0xcc, 0xcf, 0xca, 0xc9, 0xd8, 0xdb, 0xde, 0xdd, 0xd4, 0xd7, 0xd2, 0xd1, 
    0xf0, 0xf3, 0xf6, 0xf5, 0xfc, 0xff, 0xfa, 0xf9, 0xe8, 0xeb, 0xee, 0xed, 0xe4, 0xe7, 0xe2, 0xe1, 
    0xa0, 0xa3, 0xa6, 0xa5, 0xac, 0xaf, 0xaa, 0xa9, 0xb8, 0xbb, 0xbe, 0xbd, 0xb4, 0xb7, 0xb2, 0xb1, 
    0x90, 0x93, 0x96, 0x95, 0x9c, 0x9f, 0x9a, 0x99, 0x88, 0x8b, 0x8e, 0x8d, 0x84, 0x87, 0x82, 0x81, 
    0x9b, 0x98, 0x9d, 0x9e, 0x97, 0x94, 0x91, 0x92, 0x83, 0x80, 0x85, 0x86, 0x8f, 0x8c, 0x89, 0x8a, 
    0xab, 0xa8, 0xad, 0xae, 0xa7, 0xa4, 0xa1, 0xa2, 0xb3, 0xb0, 0xb5, 0xb6, 0xbf, 0xbc, 0xb9, 0xba, 
    0xfb, 0xf8, 0xfd, 0xfe, 0xf7, 0xf4, 0xf1, 0xf2, 0xe3, 0xe0, 0xe5, 0xe6, 0xef, 0xec, 0xe9, 0xea, 
    0xcb, 0xc8, 0xcd, 0xce, 0xc7, 0xc4, 0xc1, 0xc2, 0xd3, 0xd0, 0xd5, 0xd6, 0xdf, 0xdc, 0xd9, 0xda, 
    0x5b, 0x58, 0x5d, 0x5e, 0x57, 0x54, 0x51, 0x52, 0x43, 0x40, 0x45, 0x46, 0x4f, 0x4c, 0x49, 0x4a, 
    0x6b, 0x68, 0x6d, 0x6e, 0x67, 0x64, 0x61, 0x62, 0x73, 0x70, 0x75, 0x76, 0x7f, 0x7c, 0x79, 0x7a, 
    0x3b, 0x38, 0x3d, 0x3e, 0x37, 0x34, 0x31, 0x32, 0x23, 0x20, 0x25, 0x26, 0x2f, 0x2c, 0x29, 0x2a, 
    0x0b, 0x08, 0x0d, 0x0e, 0x07, 0x04, 0x01, 0x02, 0x13, 0x10, 0x15, 0x16, 0x1f, 0x1c, 0x19, 0x1a, 
];

pub const GF_MUL9: [u8; 256] = [
    0x00, 0x09, 0x12, 0x1b, 0x24, 0x2d, 0x36, 0x3f, 0x48, 0x41, 0x5a, 0x53, 0x6c, 0x65, 0x7e, 0x77,
    0x90, 0x99, 0x82, 0x8b, 0xb4, 0xbd, 0xa6, 0xaf, 0xd8, 0xd1, 0xca, 0xc3, 0xfc, 0xf5, 0xee, 0xe7,
    0x3b, 0x32, 0x29, 0x20, 0x1f, 0x16, 0x0d, 0x04, 0x73, 0x7a, 0x61, 0x68, 0x57, 0x5e, 0x45, 0x4c,
    0xab, 0xa2, 0xb9, 0xb0, 0x8f, 0x86, 0x9d, 0x94, 0xe3, 0xea, 0xf1, 0xf8, 0xc7, 0xce, 0xd5, 0xdc,
    0x76, 0x7f, 0x64, 0x6d, 0x52, 0x5b, 0x40, 0x49, 0x3e, 0x37, 0x2c, 0x25, 0x1a, 0x13, 0x08, 0x01,
    0xe6, 0xef, 0xf4, 0xfd, 0xc2, 0xcb, 0xd0, 0xd9, 0xae, 0xa7, 0xbc, 0xb5, 0x8a, 0x83, 0x98, 0x91,
    0x4d, 0x44, 0x5f, 0x56, 0x69, 0x60, 0x7b, 0x72, 0x05, 0x0c, 0x17, 0x1e, 0x21, 0x28, 0x33, 0x3a,
    0xdd, 0xd4, 0xcf, 0xc6, 0xf9, 0xf0, 0xeb, 0xe2, 0x95, 0x9c, 0x87, 0x8e, 0xb1, 0xb8, 0xa3, 0xaa,
    0xec, 0xe5, 0xfe, 0xf7, 0xc8, 0xc1, 0xda, 0xd3, 0xa4, 0xad, 0xb6, 0xbf, 0x80, 0x89, 0x92, 0x9b,
    0x7c, 0x75, 0x6e, 0x67, 0x58, 0x51, 0x4a, 0x43, 0x34, 0x3d, 0x26, 0x2f, 0x10, 0x19, 0x02, 0x0b,
    0xd7, 0xde, 0xc5, 0xcc, 0xf3, 0xfa, 0xe1, 0xe8, 0x9f, 0x96, 0x8d, 0x84, 0xbb, 0xb2, 0xa9, 0xa0,
    0x47, 0x4e, 0x55, 0x5c, 0x63, 0x6a, 0x71, 0x78, 0x0f, 0x06, 0x1d, 0x14, 0x2b, 0x22, 0x39, 0x30,
    0x9a, 0x93, 0x88, 0x81, 0xbe, 0xb7, 0xac, 0xa5, 0xd2, 0xdb, 0xc0, 0xc9, 0xf6, 0xff, 0xe4, 0xed,
    0x0a, 0x03, 0x18, 0x11, 0x2e, 0x27, 0x3c, 0x35, 0x42, 0x4b, 0x50, 0x59, 0x66, 0x6f, 0x74, 0x7d,
    0xa1, 0xa8, 0xb3, 0xba, 0x85, 0x8c, 0x97, 0x9e, 0xe9, 0xe0, 0xfb, 0xf2, 0xcd, 0xc4, 0xdf, 0xd6,
    0x31, 0x38, 0x23, 0x2a, 0x15, 0x1c, 0x07, 0x0e, 0x79, 0x70, 0x6b, 0x62, 0x5d, 0x54, 0x4f, 0x46,
];
pub const GF_MUL11: [u8; 256] = [
    0x00, 0x0b, 0x16, 0x1d, 0x2c, 0x27, 0x3a, 0x31, 0x58, 0x53, 0x4e, 0x45, 0x74, 0x7f, 0x62, 0x69,
    0xb0, 0xbb, 0xa6, 0xad, 0x9c, 0x97, 0x8a, 0x81, 0xe8, 0xe3, 0xfe, 0xf5, 0xc4, 0xcf, 0xd2, 0xd9,
    0x7b, 0x70, 0x6d, 0x66, 0x57, 0x5c, 0x41, 0x4a, 0x23, 0x28, 0x35, 0x3e, 0x0f, 0x04, 0x19, 0x12,
    0xcb, 0xc0, 0xdd, 0xd6, 0xe7, 0xec, 0xf1, 0xfa, 0x93, 0x98, 0x85, 0x8e, 0xbf, 0xb4, 0xa9, 0xa2,
    0xf6, 0xfd, 0xe0, 0xeb, 0xda, 0xd1, 0xcc, 0xc7, 0xae, 0xa5, 0xb8, 0xb3, 0x82, 0x89, 0x94, 0x9f,
    0x46, 0x4d, 0x50, 0x5b, 0x6a, 0x61, 0x7c, 0x77, 0x1e, 0x15, 0x08, 0x03, 0x32, 0x39, 0x24, 0x2f,
    0x8d, 0x86, 0x9b, 0x90, 0xa1, 0xaa, 0xb7, 0xbc, 0xd5, 0xde, 0xc3, 0xc8, 0xf9, 0xf2, 0xef, 0xe4,
    0x3d, 0x36, 0x2b, 0x20, 0x11, 0x1a, 0x07, 0x0c, 0x65, 0x6e, 0x73, 0x78, 0x49, 0x42, 0x5f, 0x54,
    0xf7, 0xfc, 0xe1, 0xea, 0xdb, 0xd0, 0xcd, 0xc6, 0xaf, 0xa4, 0xb9, 0xb2, 0x83, 0x88, 0x95, 0x9e,
    0x47, 0x4c, 0x51, 0x5a, 0x6b, 0x60, 0x7d, 0x76, 0x1f, 0x14, 0x09, 0x02, 0x33, 0x38, 0x25, 0x2e,
    0x8c, 0x87, 0x9a, 0x91, 0xa0, 0xab, 0xb6, 0xbd, 0xd4, 0xdf, 0xc2, 0xc9, 0xf8, 0xf3, 0xee, 0xe5,
    0x3c, 0x37, 0x2a, 0x21, 0x10, 0x1b, 0x06, 0x0d, 0x64, 0x6f, 0x72, 0x79, 0x48, 0x43, 0x5e, 0x55,
    0x01, 0x0a, 0x17, 0x1c, 0x2d, 0x26, 0x3b, 0x30, 0x59, 0x52, 0x4f, 0x44, 0x75, 0x7e, 0x63, 0x68,
    0xb1, 0xba, 0xa7, 0xac, 0x9d, 0x96, 0x8b, 0x80, 0xe9, 0xe2, 0xff, 0xf4, 0xc5, 0xce, 0xd3, 0xd8,
    0x7a, 0x71, 0x6c, 0x67, 0x56, 0x5d, 0x40, 0x4b, 0x22, 0x29, 0x34, 0x3f, 0x0e, 0x05, 0x18, 0x13,
    0xca, 0xc1, 0xdc, 0xd7, 0xe6, 0xed, 0xf0, 0xfb, 0x92, 0x99, 0x84, 0x8f, 0xbe, 0xb5, 0xa8, 0xa3,
];
pub const GF_MUL13: [u8; 256] = [
    0x00, 0x0d, 0x1a, 0x17, 0x34, 0x39, 0x2e, 0x23, 0x68, 0x65, 0x72, 0x7f, 0x5c, 0x51, 0x46, 0x4b,
    0xd0, 0xdd, 0xca, 0xc7, 0xe4, 0xe9, 0xfe, 0xf3, 0xb8, 0xb5, 0xa2, 0xaf, 0x8c, 0x81, 0x96, 0x9b,
    0xbb, 0xb6, 0xa1, 0xac, 0x8f, 0x82, 0x95, 0x98, 0xd3, 0xde, 0xc9, 0xc4, 0xe7, 0xea, 0xfd, 0xf0,
    0x6b, 0x66, 0x71, 0x7c, 0x5f, 0x52, 0x45, 0x48, 0x03, 0x0e, 0x19, 0x14, 0x37, 0x3a, 0x2d, 0x20,
    0x6d, 0x60, 0x77, 0x7a, 0x59, 0x54, 0x43, 0x4e, 0x05, 0x08, 0x1f, 0x12, 0x31, 0x3c, 0x2b, 0x26,
    0xbd, 0xb0, 0xa7, 0xaa, 0x89, 0x84, 0x93, 0x9e, 0xd5, 0xd8, 0xcf, 0xc2, 0xe1, 0xec, 0xfb, 0xf6,
    0xd6, 0xdb, 0xcc, 0xc1, 0xe2, 0xef, 0xf8, 0xf5, 0xbe, 0xb3, 0xa4, 0xa9, 0x8a, 0x87, 0x90, 0x9d,
    0x06, 0x0b, 0x1c, 0x11, 0x32, 0x3f, 0x28, 0x25, 0x6e, 0x63, 0x74, 0x79, 0x5a, 0x57, 0x40, 0x4d,
    0xda, 0xd7, 0xc0, 0xcd, 0xee, 0xe3, 0xf4, 0xf9, 0xb2, 0xbf, 0xa8, 0xa5, 0x86, 0x8b, 0x9c, 0x91,
    0x0a, 0x07, 0x10, 0x1d, 0x3e, 0x33, 0x24, 0x29, 0x62, 0x6f, 0x78, 0x75, 0x56, 0x5b, 0x4c, 0x41,
    0x61, 0x6c, 0x7b, 0x76, 0x55, 0x58, 0x4f, 0x42, 0x09, 0x04, 0x13, 0x1e, 0x3d, 0x30, 0x27, 0x2a,
    0xb1, 0xbc, 0xab, 0xa6, 0x85, 0x88, 0x9f, 0x92, 0xd9, 0xd4, 0xc3, 0xce, 0xed, 0xe0, 0xf7, 0xfa,
    0xb7, 0xba, 0xad, 0xa0, 0x83, 0x8e, 0x99, 0x94, 0xdf, 0xd2, 0xc5, 0xc8, 0xeb, 0xe6, 0xf1, 0xfc,
    0x67, 0x6a, 0x7d, 0x70, 0x53, 0x5e, 0x49, 0x44, 0x0f, 0x02, 0x15, 0x18, 0x3b, 0x36, 0x21, 0x2c,
    0x0c, 0x01, 0x16, 0x1b, 0x38, 0x35, 0x22, 0x2f, 0x64, 0x69, 0x7e, 0x73, 0x50, 0x5d, 0x4a, 0x47,
    0xdc, 0xd1, 0xc6, 0xcb, 0xe8, 0xe5, 0xf2, 0xff, 0xb4, 0xb9, 0xae, 0xa3, 0x80, 0x8d, 0x9a, 0x97,
];
pub const GF_MUL14: [u8; 256] = [
    0x00, 0x0e, 0x1c, 0x12, 0x38, 0x36, 0x24, 0x2a, 0x70, 0x7e, 0x6c, 0x62, 0x48, 0x46, 0x54, 0x5a,
    0xe0, 0xee, 0xfc, 0xf2, 0xd8, 0xd6, 0xc4, 0xca, 0x90, 0x9e, 0x8c, 0x82, 0xa8, 0xa6, 0xb4, 0xba,
    0xdb, 0xd5, 0xc7, 0xc9, 0xe3, 0xed, 0xff, 0xf1, 0xab, 0xa5, 0xb7, 0xb9, 0x93, 0x9d, 0x8f, 0x81,
    0x3b, 0x35, 0x27, 0x29, 0x03, 0x0d, 0x1f, 0x11, 0x4b, 0x45, 0x57, 0x59, 0x73, 0x7d, 0x6f, 0x61,
    0xad, 0xa3, 0xb1, 0xbf, 0x95, 0x9b, 0x89, 0x87, 0xdd, 0xd3, 0xc1, 0xcf, 0xe5, 0xeb, 0xf9, 0xf7,
    0x4d, 0x43, 0x51, 0x5f, 0x75, 0x7b, 0x69, 0x67, 0x3d, 0x33, 0x21, 0x2f, 0x05, 0x0b, 0x19, 0x17,
    0x76, 0x78, 0x6a, 0x64, 0x4e, 0x40, 0x52, 0x5c, 0x06, 0x08, 0x1a, 0x14, 0x3e, 0x30, 0x22, 0x2c,
    0x96, 0x98, 0x8a, 0x84, 0xae, 0xa0, 0xb2, 0xbc, 0xe6, 0xe8, 0xfa, 0xf4, 0xde, 0xd0, 0xc2, 0xcc,
    0x41, 0x4f, 0x5d, 0x53, 0x79, 0x77, 0x65, 0x6b, 0x31, 0x3f, 0x2d, 0x23, 0x09, 0x07, 0x15, 0x1b,
    0xa1, 0xaf, 0xbd, 0xb3, 0x99, 0x97, 0x85, 0x8b, 0xd1, 0xdf, 0xcd, 0xc3, 0xe9, 0xe7, 0xf5, 0xfb,
    0x9a, 0x94, 0x86, 0x88, 0xa2, 0xac, 0xbe, 0xb0, 0xea, 0xe4, 0xf6, 0xf8, 0xd2, 0xdc, 0xce, 0xc0,
    0x7a, 0x74, 0x66, 0x68, 0x42, 0x4c, 0x5e, 0x50, 0x0a, 0x04, 0x16, 0x18, 0x32, 0x3c, 0x2e, 0x20,
    0xec, 0xe2, 0xf0, 0xfe, 0xd4, 0xda, 0xc8, 0xc6, 0x9c, 0x92, 0x80, 0x8e, 0xa4, 0xaa, 0xb8, 0xb6,
    0x0c, 0x02, 0x10, 0x1e, 0x34, 0x3a, 0x28, 0x26, 0x7c, 0x72, 0x60, 0x6e, 0x44, 0x4a, 0x58, 0x56,
    0x37, 0x39, 0x2b, 0x25, 0x0f, 0x01, 0x13, 0x1d, 0x47, 0x49, 0x5b, 0x55, 0x7f, 0x71, 0x63, 0x6d,
    0xd7, 0xd9, 0xcb, 0xc5, 0xef, 0xe1, 0xf3, 0xfd, 0xa7, 0xa9, 0xbb, 0xb5, 0x9f, 0x91, 0x83, 0x8d,
];


#[inline]
pub const fn sub_byte(x: u8) -> u8 {
    FORWARD_S_BOX[x as usize]
}

#[inline]
pub const fn rot_word(x: u32) -> u32 {
    // RotWord([b0, b1, b2, b3]) = [b1, b2, b3, b0]
    let [a, b, c, d] = x.to_le_bytes();
    u32::from_le_bytes([b, c, d, a])
}

#[inline]
pub const fn sub_word(x: u32) -> u32 {
    // SubWord([b0, b1, b2, b3]) = [ SubByte(b0), SubByte(b1), SubByte(b2), SubByte(b3) ]
    let mut bytes = x.to_le_bytes();
    bytes[0] = FORWARD_S_BOX[bytes[0] as usize];
    bytes[1] = FORWARD_S_BOX[bytes[1] as usize];
    bytes[2] = FORWARD_S_BOX[bytes[2] as usize];
    bytes[3] = FORWARD_S_BOX[bytes[3] as usize];
    u32::from_le_bytes(bytes)
}

#[inline]
pub fn sub_bytes(state: &mut [u8; 16]) {
    // let s = state.clone();
    state[0]  = FORWARD_S_BOX[state[0] as usize];
    state[1]  = FORWARD_S_BOX[state[1] as usize];
    state[2]  = FORWARD_S_BOX[state[2] as usize];
    state[3]  = FORWARD_S_BOX[state[3] as usize];
    state[4]  = FORWARD_S_BOX[state[4] as usize];
    state[5]  = FORWARD_S_BOX[state[5] as usize];
    state[6]  = FORWARD_S_BOX[state[6] as usize];
    state[7]  = FORWARD_S_BOX[state[7] as usize];
    state[8]  = FORWARD_S_BOX[state[8] as usize];
    state[9]  = FORWARD_S_BOX[state[9] as usize];
    state[10] = FORWARD_S_BOX[state[10] as usize];
    state[11] = FORWARD_S_BOX[state[11] as usize];
    state[12] = FORWARD_S_BOX[state[12] as usize];
    state[13] = FORWARD_S_BOX[state[13] as usize];
    state[14] = FORWARD_S_BOX[state[14] as usize];
    state[15] = FORWARD_S_BOX[state[15] as usize];
}

#[inline]
pub fn shift_rows(state: &mut [u8; 16]) {
    // Example:
    // 00 11 22 33 
    // 44 55 66 77 
    // 88 99 aa bb
    // cc dd ee ff
    // 
    // 00 55 aa ff 
    // 44 99 ee 33 
    // 88 dd 22 77 
    // cc 11 66 bb

    // R0 << 0
    // no change.

    // R1 << 1
    let a = state[ 1];
    let b = state[ 5];
    let c = state[ 9];
    let d = state[13];
    state[ 1] = b;
    state[ 5] = c;
    state[ 9] = d;
    state[13] = a;

    // R2 << 2
    let a = state[ 2];
    let b = state[ 6];
    let c = state[10];
    let d = state[14];
    state[ 2] = c;
    state[ 6] = d;
    state[10] = a;
    state[14] = b;

    // R3 << 3
    let a = state[3];
    let b = state[7];
    let c = state[11];
    let d = state[15];
    state[ 3] = d;
    state[ 7] = a;
    state[11] = b;
    state[15] = c;
}

#[inline]
pub const fn gf_mul2(v: u8) -> u8 {
    GF_MUL2[v as usize]
}
#[inline]
pub const fn gf_mul3(v: u8) -> u8 {
    GF_MUL3[v as usize]
}
#[inline]
pub const fn gf_mul9(v: u8) -> u8 {
    GF_MUL9[v as usize]
}
#[inline]
pub const fn gf_mul11(v: u8) -> u8 {
    GF_MUL11[v as usize]
}
#[inline]
pub const fn gf_mul13(v: u8) -> u8 {
    GF_MUL13[v as usize]
}
#[inline]
pub const fn gf_mul14(v: u8) -> u8 {
    GF_MUL14[v as usize]
}


#[inline]
pub fn mix_columns(state: &mut [u8; 16]) {
    let mut c = [0u8; 16];
    c[0] = gf_mul2(state[0]) ^ gf_mul3(state[1]) ^ state[2]          ^ state[3];
    c[1] = state[0]          ^ gf_mul2(state[1]) ^ gf_mul3(state[2]) ^ state[3];
    c[2] = state[0]          ^ state[1]          ^ gf_mul2(state[2]) ^ gf_mul3(state[3]);
    c[3] = gf_mul3(state[0]) ^ state[1]          ^ state[2]          ^ gf_mul2(state[3]);

    c[4] = gf_mul2(state[4]) ^ gf_mul3(state[5]) ^ state[6]          ^ state[7];
    c[5] = state[4]          ^ gf_mul2(state[5]) ^ gf_mul3(state[6]) ^ state[7];
    c[6] = state[4]          ^ state[5]          ^ gf_mul2(state[6]) ^ gf_mul3(state[7]);
    c[7] = gf_mul3(state[4]) ^ state[5]          ^ state[6]          ^ gf_mul2(state[7]);

    c[ 8] = gf_mul2(state[8]) ^ gf_mul3(state[9]) ^ state[10]          ^ state[11];
    c[ 9] = state[8]          ^ gf_mul2(state[9]) ^ gf_mul3(state[10]) ^ state[11];
    c[10] = state[8]          ^ state[9]          ^ gf_mul2(state[10]) ^ gf_mul3(state[11]);
    c[11] = gf_mul3(state[8]) ^ state[9]          ^ state[10]          ^ gf_mul2(state[11]);

    c[12] = gf_mul2(state[12]) ^ gf_mul3(state[13]) ^ state[14]          ^ state[15];
    c[13] = state[12]          ^ gf_mul2(state[13]) ^ gf_mul3(state[14]) ^ state[15];
    c[14] = state[12]          ^ state[13]          ^ gf_mul2(state[14]) ^ gf_mul3(state[15]);
    c[15] = gf_mul3(state[12]) ^ state[13]          ^ state[14]          ^ gf_mul2(state[15]);

    state.copy_from_slice(&c);
}


#[inline]
pub fn inv_sub_bytes(state: &mut [u8; 16]) {
    // InvSubBytes
    state[0]  = REVERSE_S_BOX[state[0] as usize];
    state[1]  = REVERSE_S_BOX[state[1] as usize];
    state[2]  = REVERSE_S_BOX[state[2] as usize];
    state[3]  = REVERSE_S_BOX[state[3] as usize];
    state[4]  = REVERSE_S_BOX[state[4] as usize];
    state[5]  = REVERSE_S_BOX[state[5] as usize];
    state[6]  = REVERSE_S_BOX[state[6] as usize];
    state[7]  = REVERSE_S_BOX[state[7] as usize];
    state[8]  = REVERSE_S_BOX[state[8] as usize];
    state[9]  = REVERSE_S_BOX[state[9] as usize];
    state[10] = REVERSE_S_BOX[state[10] as usize];
    state[11] = REVERSE_S_BOX[state[11] as usize];
    state[12] = REVERSE_S_BOX[state[12] as usize];
    state[13] = REVERSE_S_BOX[state[13] as usize];
    state[14] = REVERSE_S_BOX[state[14] as usize];
    state[15] = REVERSE_S_BOX[state[15] as usize];
}



#[inline]
pub fn inv_shift_rows(state: &mut [u8; 16]) {
    // Example:
    // 00 11 22 33 
    // 44 55 66 77 
    // 88 99 aa bb 
    // cc dd ee ff
    // 
    // 00 dd aa 77 
    // 44 11 ee bb 
    // 88 55 22 ff 
    // cc 99 66 33

    // R0 >> 0
    // no change.

    // R1 >> 1
    let a = state[1];
    let b = state[5];
    let c = state[9];
    let d = state[13];
    state[1] = d;
    state[5] = a;
    state[9] = b;
    state[13] = c;

    // R2 >> 2
    let a = state[ 2];
    let b = state[ 6];
    let c = state[10];
    let d = state[14];
    state[ 2] = c;
    state[ 6] = d;
    state[10] = a;
    state[14] = b;

    // R3 >> 3
    let a = state[ 3];
    let b = state[ 7];
    let c = state[11];
    let d = state[15];
    state[ 3] = b;
    state[ 7] = c;
    state[11] = d;
    state[15] = a;
}

#[inline]
pub fn inv_mix_columns(state: &mut [u8; 16]) {
    let mut c = [0u8; 16];
    c[0] = gf_mul14(state[0]) ^ gf_mul11(state[1]) ^ gf_mul13(state[2]) ^ gf_mul9(state[3]);
    c[1] = gf_mul9(state[0])  ^ gf_mul14(state[1]) ^ gf_mul11(state[2]) ^ gf_mul13(state[3]);
    c[2] = gf_mul13(state[0]) ^ gf_mul9(state[1])  ^ gf_mul14(state[2]) ^ gf_mul11(state[3]);
    c[3] = gf_mul11(state[0]) ^ gf_mul13(state[1]) ^ gf_mul9(state[2])  ^ gf_mul14(state[3]);

    c[4] = gf_mul14(state[4]) ^ gf_mul11(state[5]) ^ gf_mul13(state[6]) ^ gf_mul9(state[7]);
    c[5] = gf_mul9(state[4])  ^ gf_mul14(state[5]) ^ gf_mul11(state[6]) ^ gf_mul13(state[7]);
    c[6] = gf_mul13(state[4]) ^ gf_mul9(state[5])  ^ gf_mul14(state[6]) ^ gf_mul11(state[7]);
    c[7] = gf_mul11(state[4]) ^ gf_mul13(state[5]) ^ gf_mul9(state[6])  ^ gf_mul14(state[7]);
    
    c[ 8] = gf_mul14(state[8]) ^ gf_mul11(state[9]) ^ gf_mul13(state[10]) ^ gf_mul9(state[11]);
    c[ 9] = gf_mul9(state[8])  ^ gf_mul14(state[9]) ^ gf_mul11(state[10]) ^ gf_mul13(state[11]);
    c[10] = gf_mul13(state[8]) ^ gf_mul9(state[9])  ^ gf_mul14(state[10]) ^ gf_mul11(state[11]);
    c[11] = gf_mul11(state[8]) ^ gf_mul13(state[9]) ^ gf_mul9(state[10])  ^ gf_mul14(state[11]);

    c[12] = gf_mul14(state[12]) ^ gf_mul11(state[13]) ^ gf_mul13(state[14]) ^ gf_mul9(state[15]);
    c[13] = gf_mul9(state[12])  ^ gf_mul14(state[13]) ^ gf_mul11(state[14]) ^ gf_mul13(state[15]);
    c[14] = gf_mul13(state[12]) ^ gf_mul9(state[13])  ^ gf_mul14(state[14]) ^ gf_mul11(state[15]);
    c[15] = gf_mul11(state[12]) ^ gf_mul13(state[13]) ^ gf_mul9(state[14])  ^ gf_mul14(state[15]);

    state.copy_from_slice(&c);
}

#[inline]
pub fn key_expansion(key: &[u8], expanded_key: &mut [u8]) {
    assert!(key.len() == AES128_KEY_LEN || key.len() == AES192_KEY_LEN || key.len() == AES256_KEY_LEN);
    
    // Nk
    let nk = key.len() / WORD_SIZE;
    // Nr
    let nr = match key.len() {
        16 => 10usize,
        24 => 12usize,
        32 => 14usize,
        _  => unreachable!("invalid AES key size"),
    };
    // (Nr + 1) * Nb
    assert_eq!(expanded_key.len(), (nr + 1) * AES_BLOCK_LEN );

    for i in 0..key.len() {
        expanded_key[i] = key[i];
    }

    // 
    // 4..44
    // 6..52
    // 8..60
    for i in nk..(AES_NB * (nr + 1)) {
        // temp = w[i-1]
        let idx = (i - 1) * WORD_SIZE;
        let mut a = expanded_key[idx + 0];
        let mut b = expanded_key[idx + 1];
        let mut c = expanded_key[idx + 2];
        let mut d = expanded_key[idx + 3];

        if i % nk == 0 {
            let a0 = sub_byte(a);
            let b0 = sub_byte(b);
            let c0 = sub_byte(c);
            let d0 = sub_byte(d);

            let rcon = RCON[i/nk - 1].to_le_bytes();
            a = b0 ^ rcon[0];
            b = c0 ^ rcon[1];
            c = d0 ^ rcon[2];
            d = a0 ^ rcon[3];
        } else if nk > 6 && i % nk == 4 {
            a = sub_byte(a);
            b = sub_byte(b);
            c = sub_byte(c);
            d = sub_byte(d);
        }

        let x = (i - nk) * WORD_SIZE; // w[i-Nk]
                    
        let k0 = expanded_key[x + 0];
        let k1 = expanded_key[x + 1];
        let k2 = expanded_key[x + 2];
        let k3 = expanded_key[x + 3];

        // w[i] = w[i-Nk] xor temp
        let y = i * WORD_SIZE;
        expanded_key[y + 0] = k0 ^ a;
        expanded_key[y + 1] = k1 ^ b;
        expanded_key[y + 2] = k2 ^ c;
        expanded_key[y + 3] = k3 ^ d;
    }
}

#[inline]
pub fn add_round_key(state: &mut [u8; 16], rounds_key: &[u8], round: usize) {
    debug_assert!(rounds_key.len() >= round * AES_BLOCK_LEN);

    for i in 0..AES_BLOCK_LEN {
        state[i] ^= rounds_key[i + round * AES_BLOCK_LEN];
    }
}

#[inline]
pub fn encrypt(state: &mut [u8; 16], expanded_key: &[u8], nr: usize) {
    debug_assert!(nr == AES128_NR || nr == AES192_NR || nr == AES256_NR);

    add_round_key(state, expanded_key, 0);

    for i in 1..nr {
        sub_bytes(state);
        shift_rows(state);
        mix_columns(state);
        add_round_key(state, expanded_key, i);
    }

    sub_bytes(state);
    shift_rows(state);
    add_round_key(state, expanded_key, nr);
}

pub fn decrypt(state: &mut [u8; 16], expanded_key: &[u8], nr: usize) {
    debug_assert!(nr == AES128_NR || nr == AES192_NR || nr == AES256_NR);

    add_round_key(state, expanded_key, nr);
    inv_shift_rows(state);
    inv_sub_bytes(state);

    for i in 1..nr {
        add_round_key(state, expanded_key, nr - i);
        inv_mix_columns(state);
        inv_shift_rows(state);
        inv_sub_bytes(state);
    }

    add_round_key(state, expanded_key, 0);
}



// =============================== Bench ==================================
#[cfg(test)]
#[bench]
fn bench_aes128_enc(b: &mut test::Bencher) {
    let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    let key   = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();

    let mut expanded_key = [0u8; (AES128_NR + 1) * AES_BLOCK_LEN ];
    key_expansion(&key, &mut expanded_key);

    b.bytes = AES_BLOCK_LEN as u64;
    b.iter(|| {
        let mut state: [u8; 16] = [1u8; 16];
        encrypt(&mut state, &expanded_key, AES128_NR);
        state
    })
}
#[cfg(test)]
#[bench]
fn bench_aes128_dec(b: &mut test::Bencher) {
    let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
    let key   = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();

    let mut expanded_key = [0u8; (AES128_NR + 1) * AES_BLOCK_LEN ];
    key_expansion(&key, &mut expanded_key);
    
    let mut state: [u8; 16] = [1u8; 16];
    encrypt(&mut state, &expanded_key, AES128_NR);
    
    b.bytes = AES_BLOCK_LEN as u64;
    b.iter(|| {
        let mut state = state.clone();
        decrypt(&mut state, &expanded_key, AES128_NR);
        state
    })
}


// =============================== Test Key Expansion ================================
#[test]
fn test_key_expansion_128() {
    // A.1 Expansion of a 128-bit Cipher Key 
    let key: [u8; 16] = [
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 
        0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c,
    ];

    let mut expanded_key = [0u8; (AES128_NR + 1) * AES_BLOCK_LEN];
    key_expansion(&key, &mut expanded_key);
    assert_eq!(&expanded_key[..], &[
        0x2b, 0x7e, 0x15, 0x16, 0x28, 0xae, 0xd2, 0xa6, 
        0xab, 0xf7, 0x15, 0x88, 0x09, 0xcf, 0x4f, 0x3c, 
        0xa0, 0xfa, 0xfe, 0x17, 0x88, 0x54, 0x2c, 0xb1, 
        0x23, 0xa3, 0x39, 0x39, 0x2a, 0x6c, 0x76, 0x05, 
        0xf2, 0xc2, 0x95, 0xf2, 0x7a, 0x96, 0xb9, 0x43, 
        0x59, 0x35, 0x80, 0x7a, 0x73, 0x59, 0xf6, 0x7f, 
        0x3d, 0x80, 0x47, 0x7d, 0x47, 0x16, 0xfe, 0x3e, 
        0x1e, 0x23, 0x7e, 0x44, 0x6d, 0x7a, 0x88, 0x3b, 
        0xef, 0x44, 0xa5, 0x41, 0xa8, 0x52, 0x5b, 0x7f, 
        0xb6, 0x71, 0x25, 0x3b, 0xdb, 0x0b, 0xad, 0x00, 
        0xd4, 0xd1, 0xc6, 0xf8, 0x7c, 0x83, 0x9d, 0x87, 
        0xca, 0xf2, 0xb8, 0xbc, 0x11, 0xf9, 0x15, 0xbc, 
        0x6d, 0x88, 0xa3, 0x7a, 0x11, 0x0b, 0x3e, 0xfd, 
        0xdb, 0xf9, 0x86, 0x41, 0xca, 0x00, 0x93, 0xfd, 
        0x4e, 0x54, 0xf7, 0x0e, 0x5f, 0x5f, 0xc9, 0xf3, 
        0x84, 0xa6, 0x4f, 0xb2, 0x4e, 0xa6, 0xdc, 0x4f, 
        0xea, 0xd2, 0x73, 0x21, 0xb5, 0x8d, 0xba, 0xd2, 
        0x31, 0x2b, 0xf5, 0x60, 0x7f, 0x8d, 0x29, 0x2f, 
        0xac, 0x77, 0x66, 0xf3, 0x19, 0xfa, 0xdc, 0x21, 
        0x28, 0xd1, 0x29, 0x41, 0x57, 0x5c, 0x00, 0x6e, 
        0xd0, 0x14, 0xf9, 0xa8, 0xc9, 0xee, 0x25, 0x89, 
        0xe1, 0x3f, 0x0c, 0xc8, 0xb6, 0x63, 0x0c, 0xa6, 
    ][..]);
}

#[test]
fn test_key_expansion_192() {
    // A.2 Expansion of a 192-bit Cipher Key
    let key: [u8; 24] = [
        0x8e, 0x73, 0xb0, 0xf7, 0xda, 0x0e, 0x64, 0x52, 
        0xc8, 0x10, 0xf3, 0x2b, 0x80, 0x90, 0x79, 0xe5, 
        0x62, 0xf8, 0xea, 0xd2, 0x52, 0x2c, 0x6b, 0x7b,
    ];

    let mut expanded_key = [0u8; (AES192_NR + 1) * AES_BLOCK_LEN];
    key_expansion(&key, &mut expanded_key);
    assert_eq!(&expanded_key[..], &[
        0x8e, 0x73, 0xb0, 0xf7, 0xda, 0x0e, 0x64, 0x52, 
        0xc8, 0x10, 0xf3, 0x2b, 0x80, 0x90, 0x79, 0xe5, 
        0x62, 0xf8, 0xea, 0xd2, 0x52, 0x2c, 0x6b, 0x7b, 
        0xfe, 0x0c, 0x91, 0xf7, 0x24, 0x02, 0xf5, 0xa5, 
        0xec, 0x12, 0x06, 0x8e, 0x6c, 0x82, 0x7f, 0x6b, 
        0x0e, 0x7a, 0x95, 0xb9, 0x5c, 0x56, 0xfe, 0xc2, 
        0x4d, 0xb7, 0xb4, 0xbd, 0x69, 0xb5, 0x41, 0x18, 
        0x85, 0xa7, 0x47, 0x96, 0xe9, 0x25, 0x38, 0xfd, 
        0xe7, 0x5f, 0xad, 0x44, 0xbb, 0x09, 0x53, 0x86, 
        0x48, 0x5a, 0xf0, 0x57, 0x21, 0xef, 0xb1, 0x4f, 
        0xa4, 0x48, 0xf6, 0xd9, 0x4d, 0x6d, 0xce, 0x24, 
        0xaa, 0x32, 0x63, 0x60, 0x11, 0x3b, 0x30, 0xe6, 
        0xa2, 0x5e, 0x7e, 0xd5, 0x83, 0xb1, 0xcf, 0x9a, 
        0x27, 0xf9, 0x39, 0x43, 0x6a, 0x94, 0xf7, 0x67, 
        0xc0, 0xa6, 0x94, 0x07, 0xd1, 0x9d, 0xa4, 0xe1, 
        0xec, 0x17, 0x86, 0xeb, 0x6f, 0xa6, 0x49, 0x71, 
        0x48, 0x5f, 0x70, 0x32, 0x22, 0xcb, 0x87, 0x55, 
        0xe2, 0x6d, 0x13, 0x52, 0x33, 0xf0, 0xb7, 0xb3, 
        0x40, 0xbe, 0xeb, 0x28, 0x2f, 0x18, 0xa2, 0x59, 
        0x67, 0x47, 0xd2, 0x6b, 0x45, 0x8c, 0x55, 0x3e, 
        0xa7, 0xe1, 0x46, 0x6c, 0x94, 0x11, 0xf1, 0xdf, 
        0x82, 0x1f, 0x75, 0x0a, 0xad, 0x07, 0xd7, 0x53, 
        0xca, 0x40, 0x05, 0x38, 0x8f, 0xcc, 0x50, 0x06, 
        0x28, 0x2d, 0x16, 0x6a, 0xbc, 0x3c, 0xe7, 0xb5, 
        0xe9, 0x8b, 0xa0, 0x6f, 0x44, 0x8c, 0x77, 0x3c, 
        0x8e, 0xcc, 0x72, 0x04, 0x01, 0x00, 0x22, 0x02, 
    ][..]);
}

#[test]
fn test_key_expansion_256() {
    // A.3 Expansion of a 256-bit Cipher Key
    let key: [u8; 32] = [
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 
        0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81,
        0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 
        0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4,
    ];

    let mut expanded_key = [0u8; (AES256_NR + 1) * AES_BLOCK_LEN];
    key_expansion(&key, &mut expanded_key);

    assert_eq!(&expanded_key[..], &[
        0x60, 0x3d, 0xeb, 0x10, 0x15, 0xca, 0x71, 0xbe, 
        0x2b, 0x73, 0xae, 0xf0, 0x85, 0x7d, 0x77, 0x81, 
        0x1f, 0x35, 0x2c, 0x07, 0x3b, 0x61, 0x08, 0xd7, 
        0x2d, 0x98, 0x10, 0xa3, 0x09, 0x14, 0xdf, 0xf4, 
        0x9b, 0xa3, 0x54, 0x11, 0x8e, 0x69, 0x25, 0xaf, 
        0xa5, 0x1a, 0x8b, 0x5f, 0x20, 0x67, 0xfc, 0xde, 
        0xa8, 0xb0, 0x9c, 0x1a, 0x93, 0xd1, 0x94, 0xcd, 
        0xbe, 0x49, 0x84, 0x6e, 0xb7, 0x5d, 0x5b, 0x9a, 
        0xd5, 0x9a, 0xec, 0xb8, 0x5b, 0xf3, 0xc9, 0x17, 
        0xfe, 0xe9, 0x42, 0x48, 0xde, 0x8e, 0xbe, 0x96, 
        0xb5, 0xa9, 0x32, 0x8a, 0x26, 0x78, 0xa6, 0x47, 
        0x98, 0x31, 0x22, 0x29, 0x2f, 0x6c, 0x79, 0xb3, 
        0x81, 0x2c, 0x81, 0xad, 0xda, 0xdf, 0x48, 0xba, 
        0x24, 0x36, 0x0a, 0xf2, 0xfa, 0xb8, 0xb4, 0x64, 
        0x98, 0xc5, 0xbf, 0xc9, 0xbe, 0xbd, 0x19, 0x8e, 
        0x26, 0x8c, 0x3b, 0xa7, 0x09, 0xe0, 0x42, 0x14, 
        0x68, 0x00, 0x7b, 0xac, 0xb2, 0xdf, 0x33, 0x16, 
        0x96, 0xe9, 0x39, 0xe4, 0x6c, 0x51, 0x8d, 0x80, 
        0xc8, 0x14, 0xe2, 0x04, 0x76, 0xa9, 0xfb, 0x8a, 
        0x50, 0x25, 0xc0, 0x2d, 0x59, 0xc5, 0x82, 0x39, 
        0xde, 0x13, 0x69, 0x67, 0x6c, 0xcc, 0x5a, 0x71, 
        0xfa, 0x25, 0x63, 0x95, 0x96, 0x74, 0xee, 0x15, 
        0x58, 0x86, 0xca, 0x5d, 0x2e, 0x2f, 0x31, 0xd7, 
        0x7e, 0x0a, 0xf1, 0xfa, 0x27, 0xcf, 0x73, 0xc3, 
        0x74, 0x9c, 0x47, 0xab, 0x18, 0x50, 0x1d, 0xda, 
        0xe2, 0x75, 0x7e, 0x4f, 0x74, 0x01, 0x90, 0x5a, 
        0xca, 0xfa, 0xaa, 0xe3, 0xe4, 0xd5, 0x9b, 0x34, 
        0x9a, 0xdf, 0x6a, 0xce, 0xbd, 0x10, 0x19, 0x0d, 
        0xfe, 0x48, 0x90, 0xd1, 0xe6, 0x18, 0x8d, 0x0b, 
        0x04, 0x6d, 0xf3, 0x44, 0x70, 0x6c, 0x63, 0x1e, 
    ][..]);
}

// =============================== Test Cipher Example ================================
#[test]
fn test_cipher() {
    // Appendix B – Cipher Example

    // AES 128
    let input = hex::decode("3243f6a8885a308d313198a2e0370734").unwrap();
    let key   = hex::decode("2b7e151628aed2a6abf7158809cf4f3c").unwrap();
    
    let mut state: [u8; 16] = [0u8; 16];
    state.copy_from_slice(&input);

    let mut expanded_key = [0u8; (AES128_NR + 1) * AES_BLOCK_LEN];
    key_expansion(&key, &mut expanded_key);

    encrypt(&mut state, &expanded_key, AES128_NR);
    // 39 25 84 1d 
    // 02 dc 09 fb 
    // dc 11 85 97 
    // 19 6a 0b 32
    assert_eq!(&state[..], &hex::decode("\
3925841d\
02dc09fb\
dc118597\
196a0b32").unwrap()[..]);

    // state to output
    let s = state.clone();
    state[0] = s[0];
    state[1] = s[4];
    state[2] = s[8];
    state[3] = s[12];

    state[4] = s[1];
    state[5] = s[5];
    state[6] = s[9];
    state[7] = s[13];

    state[ 8] = s[2];
    state[ 9] = s[6];
    state[10] = s[10];
    state[11] = s[14];

    state[12] = s[3];
    state[13] = s[7];
    state[14] = s[11];
    state[15] = s[15];
    // 39 02 dc 19 
    // 25 dc 11 6a 
    // 84 09 85 0b 
    // 1d fb 97 32
    assert_eq!(&state[..], &hex::decode("\
3902dc19\
25dc116a\
8409850b\
1dfb9732").unwrap()[..]);
}

#[test]
fn test_example_vectors() {
    // Appendix C – Example Vectors 
    {
        // AES 128
        let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
        let key   = hex::decode("000102030405060708090a0b0c0d0e0f").unwrap();
        
        let mut state: [u8; 16] = [0u8; 16];
        state.copy_from_slice(&input);

        let mut expanded_key = [0u8; (AES128_NR + 1) * AES_BLOCK_LEN];
        key_expansion(&key, &mut expanded_key);

        encrypt(&mut state, &expanded_key, AES128_NR);
        assert_eq!(&state[..],
            &hex::decode("69c4e0d86a7b0430d8cdb78070b4c55a").unwrap()[..]);

        decrypt(&mut state, &expanded_key, AES128_NR);
        assert_eq!(&state[..], &input[..]);
    }
    
    {
        // AES 192
        let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
        let key   = hex::decode("000102030405060708090a0b0c0d0e0f1011121314151617").unwrap();
        
        let mut state: [u8; 16] = [0u8; 16];
        state.copy_from_slice(&input);

        let mut expanded_key = [0u8; (AES192_NR + 1) * AES_BLOCK_LEN];
        key_expansion(&key, &mut expanded_key);

        encrypt(&mut state, &expanded_key, AES192_NR);
        assert_eq!(&state[..], &hex::decode("dda97ca4864cdfe06eaf70a0ec0d7191").unwrap()[..]);

        decrypt(&mut state, &expanded_key, AES192_NR);
        assert_eq!(&state[..], &input[..]);
    }

    {
        // AES 256
        let input = hex::decode("00112233445566778899aabbccddeeff").unwrap();
        let key   = hex::decode("000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f").unwrap();
        
        let mut state: [u8; 16] = [0u8; 16];
        state.copy_from_slice(&input);

        let mut expanded_key = [0u8; (AES256_NR + 1) * AES_BLOCK_LEN];
        key_expansion(&key, &mut expanded_key);

        encrypt(&mut state, &expanded_key, AES256_NR);
        assert_eq!(&state[..], &hex::decode("8ea2b7ca516745bfeafc49904b496089").unwrap()[..]);

        decrypt(&mut state, &expanded_key, AES256_NR);
        assert_eq!(&state[..], &input[..]);
    }
}

