/// Module containing utilities testing building AVL Frames sent by Teltonika Telematics devices for testing purposes
use nom_teltonika::{Codec, AVLRecord, AVLFrame};

/// Builder for [`AVLFrame`]s
///
/// `crc16` is calculated from binary representation of the frame during serialization and therefore it is ignored here.
pub struct AVLFrameBuilder {
codec: Codec,
crc16: u32,
records: Vec<AVLRecord>
}

impl AVLFrameBuilder {
  /// Returns a new instance of [`AVLFrameBuilder`]
  pub fn new() -> AVLFrameBuilder {
    AVLFrameBuilder {
      codec: Codec::C8,
      crc16: 0,
      records: vec![]
    }
  }

  /// Builds the [`AVLFrame`] from the given data
  pub fn build(self) -> AVLFrame {
    AVLFrame {
      codec: self.codec,
      crc16: self.crc16,
      records: self.records
    }
  }

  /// Sets the codec of the [`AVLFrame`]
  pub fn with_codec(mut self, codec: Codec) -> AVLFrameBuilder {
    self.codec = codec;
    return self;
  }

  /// Adds a record to the [`AVLFrame`]
  pub fn add_record(mut self, record: AVLRecord) -> AVLFrameBuilder {
    self.records.push(record);
    return self;
  }

  /// Sets the records of the [`AVLFrame`]
  pub fn with_records(mut self, records: Vec<AVLRecord>) -> AVLFrameBuilder {
    self.records = records;
    return self;
  }
}