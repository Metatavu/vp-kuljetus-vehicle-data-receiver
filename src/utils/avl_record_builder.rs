#![allow(dead_code)]
/// Module containing utilities testing building AVL Records sent by Teltonika Telematics devices for testing purposes
#[cfg(test)]
pub mod avl_record_builder {
    use chrono::{DateTime, Utc};
    use nom_teltonika::{AVLEventIO, AVLRecord, Priority};

  /// Builder for [`AVLRecord`]s
  ///
  /// [`nom_teltonika::AVLRecord`] contains some determined field(s) that are not included in the actual packets and
  /// they are added as [`None`] here.
  pub struct AVLRecordBuilder {
      timestamp: Option<DateTime<Utc>>,
      priority: Option<Priority>,
      trigger_event_id: Option<u16>,
      io_events: Vec<AVLEventIO>,
  }

  impl AVLRecordBuilder {
    /// Returns a new instance of [`AVLRecordBuilder`]
    pub fn new() -> AVLRecordBuilder {
        AVLRecordBuilder {
            timestamp: Some(Utc::now()),
            priority: Some(Priority::Low),
            trigger_event_id: None,
            io_events: vec![],
        }
    }

    /// Builds the [`AVLRecord`] from the given data
    pub fn build(self) -> AVLRecord {
        AVLRecord {
            timestamp: self.timestamp.unwrap(),
            priority: self.priority.unwrap(),
            longitude: 0.0,
            latitude: 0.0,
            altitude: 0,
            angle: 0,
            satellites: 0,
            speed: 0,
            trigger_event_id: self.trigger_event_id.unwrap_or(0),
            generation_type: None,
            io_events: self.io_events
        }
    }

    /// Sets the timestamp of the [`AVLRecord`]
    pub fn with_timestamp(mut self, timestamp: DateTime<Utc>) -> AVLRecordBuilder {
        self.timestamp = Some(timestamp);
        return self;
    }

    /// Sets the priority of the [`AVLRecord`]
    pub fn with_priority(mut self, priority: Priority) -> AVLRecordBuilder {
        self.priority = Some(priority);
        return self;
    }

    /// Sets the trigger event id of the [`AVLRecord`]
    pub fn with_trigger_event_id(mut self, trigger_event_id: u16) -> AVLRecordBuilder {
        self.trigger_event_id = Some(trigger_event_id);
        return self;
    }

    /// Adds an [`AVLEventIO`] to the [`AVLRecord`]
    pub fn add_io_event(mut self, io_event: AVLEventIO) -> AVLRecordBuilder {
        self.io_events.push(io_event);
        return self;
    }

    /// Sets the [`AVLEventIO`]s of the [`AVLRecord`]
    pub fn with_io_events(mut self, io_events: Vec<AVLEventIO>) -> AVLRecordBuilder {
        self.io_events = io_events;
        return self;
    }
  }
}