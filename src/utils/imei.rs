/// Module containing utility functions for testing IMEI parsing
use rand::Rng;

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

/// Generates a random valid IMEI number
///
/// # Returns
/// * `String` - The generated IMEI
pub fn get_random_imei() -> String {
    let mut rng = rand::thread_rng();
    let mut imei: Vec<u8> = (0..14).map(|_| rng.gen_range(0..=9)).collect();

    // Calculate the checksum for the first 14 digits
    let mut checksum = 0;
    for (i, &digit) in imei.iter().rev().enumerate() {
        if i % 2 == 0 {
            let double = digit * 2;
            checksum += if double > 9 { double - 9 } else { double };
        } else {
            checksum += digit;
        }
    }

    // Calculate the final digit to make it valid
    let final_digit = (10 - (checksum % 10)) % 10;
    imei.push(final_digit);

    // Convert to string
    imei.iter().map(|d| d.to_string()).collect::<String>()
}
