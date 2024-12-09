/// Module containing utility functions for testing IMEI parsing
use rand::{distributions::Alphanumeric, Rng};

const VALID_TEST_IMEIS: [&str; 10] = [
    "354895074321654",
    "865432107654321",
    "359876541234567",
    "860123456789012",
    "490154203237518",
    "356789045612398",
    "867530912345678",
    "352634509876543",
    "864209765432187",
    "351234567890123",
];

/// Builds a valid IMEI packet from the given IMEI
///
/// The first two bytes denote the length of the IMEI and the rest are the IMEI itself.
///
/// # Arguments
/// * `imei` - The IMEI to build the packet from
///
/// # Returns
/// * `Vec<u8>` - The IMEI packet
pub fn build_valid_imei_packet(imei: &str) -> Vec<u8> {
    let length = imei.len() as i16;
    let mut imei_byte_array = length.to_be_bytes().to_vec();

    imei_byte_array.append(&mut imei.as_bytes().to_vec());

    return imei_byte_array;
}

/// Builds an invalid IMEI packet from the given IMEI
///
/// Otherwise the same as [`build_valid_imei_packet`] expect it doesn't prepend the length of the IMEI.
///
/// # Arguments
/// * `imei` - The IMEI to build the packet from
///
/// # Returns
/// * `Vec<u8>` - The IMEI packet
pub fn build_invalid_imei_packet(imei: &str) -> Vec<u8> {
    return imei.as_bytes().to_vec();
}

/// Generates a random IMEI of the given length
///
/// # Returns
/// * `String` - The generated IMEI
pub fn get_random_imei() -> String {
    VALID_TEST_IMEIS[rand::thread_rng().gen_range(0..VALID_TEST_IMEIS.len())].to_string()
}
