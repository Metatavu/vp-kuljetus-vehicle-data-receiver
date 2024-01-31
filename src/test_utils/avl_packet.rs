
/// Module containing utility functions for testing AVL packets
use nom_teltonika::{crc16, AVLEventIO, AVLEventIOValue, AVLFrame, AVLRecord, Priority};
const AVL_PACKET_PREAMBLE: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
const AVL_PACKET_CODEC8: [u8; 1] = [0x08];

/// Trait for converting AVL packet to bytes
///
/// Allows for constructing AVL packets from the given data for testing various parsing scenarios.
/// See https://wiki.teltonika-gps.com/view/Codec#Codec_8 for reference of byte order etc.
pub trait AVLPacketToBytes {
    /// Converts the AVL packet to vector of bytes
    fn to_bytes(&self) -> Vec<u8>;
}

impl AVLPacketToBytes for Priority {
  fn to_bytes(&self) -> Vec<u8> {
    match self {
        Priority::Low => vec![0x00],
        Priority::High => vec![0x01],
        Priority::Panic => vec![0x02],
    }
  }
}

impl AVLPacketToBytes for AVLFrame {
  fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();
    let mut bytes_for_crc: Vec<u8> = Vec::new();
    let mut number_of_data: u8 = 0;
    for _ in &self.records {
        number_of_data = number_of_data + 1;
    }

    bytes.append(&mut AVL_PACKET_PREAMBLE.to_vec());
    bytes_for_crc.append(&mut AVL_PACKET_CODEC8.to_vec());
    bytes_for_crc.append(&mut number_of_data.to_be_bytes().to_vec());
    bytes_for_crc.append(&mut self.records.to_bytes());
    bytes_for_crc.append(&mut number_of_data.to_be_bytes().to_vec());
    let crc16 = crc16(&bytes_for_crc) as u32;
    let mut data_field_length = (bytes_for_crc.len() as i32).to_be_bytes().to_vec();
    bytes.append(&mut data_field_length);
    bytes.append(&mut bytes_for_crc);
    bytes.append(&mut crc16.to_be_bytes().to_vec());

    return bytes;
  }
}

impl AVLPacketToBytes for Vec<AVLRecord> {
  fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();

    for record in self {
        bytes.append(&mut record.to_bytes());
    }

    return bytes;
  }
}

impl AVLPacketToBytes for AVLRecord {
  fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();

    gps_element_to_bytes(&mut bytes, self);

    let trigger_event_id = (self.trigger_event_id as i8).to_be_bytes();
    let mut u8_events: Vec<(u8, u8)> = Vec::new();
    let mut u16_events: Vec<(u8, u16)> = Vec::new();
    let mut u32_events: Vec<(u8, u32)> = Vec::new();
    let mut u64_events: Vec<(u8, u64)> = Vec::new();

    for event in &self.io_events {
        match event.value {
            AVLEventIOValue::U8(value) => u8_events.push((event.id as u8, value)),
            AVLEventIOValue::U16(value) => u16_events.push((event.id as u8, value)),
            AVLEventIOValue::U32(value) => u32_events.push((event.id as u8, value)),
            AVLEventIOValue::U64(value) => u64_events.push((event.id as u8, value)),
            AVLEventIOValue::Variable(_) => (),
        }
    }
    bytes.append(&mut trigger_event_id.to_vec());

    bytes.append(&mut (self.io_events.len() as u8).to_be_bytes().to_vec());
    bytes.append(&mut (u8_events.len() as u8).to_be_bytes().to_vec());
    for (id, value) in u8_events {
        bytes.append(&mut id.to_be_bytes().to_vec());
        bytes.append(&mut value.to_be_bytes().to_vec());
    }
    bytes.append(&mut (u16_events.len() as u8).to_be_bytes().to_vec());
    for (id, value) in u16_events {
        bytes.append(&mut id.to_be_bytes().to_vec());
        bytes.append(&mut value.to_be_bytes().to_vec());
    }
    bytes.append(&mut (u32_events.len() as u8).to_be_bytes().to_vec());
    for (id, value) in u32_events {
        bytes.append(&mut id.to_be_bytes().to_vec());
        bytes.append(&mut value.to_be_bytes().to_vec());
    }
    bytes.append(&mut (u64_events.len() as u8).to_be_bytes().to_vec());
    for (id, value) in u64_events {
        bytes.append(&mut id.to_be_bytes().to_vec());
        bytes.append(&mut value.to_be_bytes().to_vec());
    }

    return bytes;
  }
}

impl AVLPacketToBytes for Vec<AVLEventIO> {
  fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();

    for event in self {
        bytes.append(&mut event.to_bytes());
    }

    return bytes;
  }
}

impl AVLPacketToBytes for AVLEventIO {
  fn to_bytes(&self) -> Vec<u8> {
    let mut bytes = Vec::new();

    let id = self.id.to_be_bytes();
    let value: Vec<u8> = match self.value {
        AVLEventIOValue::U8(value) => value.to_be_bytes().to_vec(),
        AVLEventIOValue::U16(value) => value.to_be_bytes().to_vec(),
        AVLEventIOValue::U32(value) => value.to_be_bytes().to_vec(),
        AVLEventIOValue::U64(value) => value.to_be_bytes().to_vec(),
        // Implement this IF needed later in development
        AVLEventIOValue::Variable(_) => vec![0x00],
    };

    bytes.append(&mut id.to_vec());
    bytes.append(&mut value.to_vec());

    return bytes;
  }
}

/// Parses the GPS element of the AVL packet and appends them in correct order to the given bytes vector
///
/// # Arguments
/// * `bytes` - Vector of bytes to which the GPS element will be appended
/// * `record` - AVL record containing the GPS element
fn gps_element_to_bytes(bytes:&mut Vec<u8>, record: &AVLRecord) {
  let timestamp = record.timestamp.timestamp().to_be_bytes().to_vec();
  let priority = record.priority.to_bytes();
  let longitude = (record.longitude as i32).to_be_bytes();
  let latitude = (record.latitude as i32).to_be_bytes();
  let altitude = record.altitude.to_be_bytes();
  let angle = record.angle.to_be_bytes();
  let satellites = record.satellites.to_be_bytes();
  let speed = record.speed.to_be_bytes();

  bytes.append(&mut timestamp.to_vec());
  bytes.append(&mut priority.to_vec());
  bytes.append(&mut longitude.to_vec());
  bytes.append(&mut latitude.to_vec());
  bytes.append(&mut altitude.to_vec());
  bytes.append(&mut angle.to_vec());
  bytes.append(&mut satellites.to_vec());
  bytes.append(&mut speed.to_vec());
}