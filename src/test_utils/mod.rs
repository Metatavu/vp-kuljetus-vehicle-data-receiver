#[cfg(test)]
pub mod avl_record_builder;
#[cfg(test)]
pub mod avl_frame_builder;
#[cfg(test)]
pub mod imei;
#[cfg(test)]
pub mod avl_packet;

pub mod utilities {
    /// Converts a hex string to a byte vector
    ///
    /// # Arguments
    /// * `input` - The hex string to convert
    ///
    /// # Returns
    /// * `Vec<u8>` - The byte vector
    pub fn str_to_bytes(input: &str) -> Vec<u8> {
        let mut bytes: Vec<u8> = vec![];
        for (i, char) in input.chars().enumerate() {
            let val = if i % 2 != 0 {
                format!("{}{}", input.chars().nth(i - 1).unwrap(), char)
            } else {
                continue;
            };
            bytes.push(u8::from_str_radix(&val, 16).unwrap())
        }

        return bytes;
    }
}