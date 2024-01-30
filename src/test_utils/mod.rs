/// Module containing utility functions for testing IMEI parsing
#[cfg(test)]
pub mod imei {
    use rand::{distributions::Alphanumeric, Rng};

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
    /// # Arguments
    /// * `length` - The length of the IMEI to generate
    ///
    /// # Returns
    /// * `String` - The generated IMEI
    pub fn get_random_imei_of_length(length: i16) -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(length as usize)
            .map(char::from)
            .collect()
    }
}