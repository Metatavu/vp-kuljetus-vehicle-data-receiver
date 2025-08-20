use chrono::{DateTime, Utc};
use nom_teltonika::{AVLEventIO, AVLEventIOValue, AVLFrame, Priority};
use vp_kuljetus_vehicle_data_receiver::utils::avl_frame_builder::AVLFrameBuilder;
use vp_kuljetus_vehicle_data_receiver::utils::avl_record_builder::avl_record_builder::AVLRecordBuilder;

pub fn create_temperature_frame(timestamp: DateTime<Utc>) -> AVLFrame {
    return AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::Low)
            .with_timestamp(timestamp)
            .with_angle(0)
            .with_latitude(61.0)
            .with_longitude(27.0)
            .add_io_event(AVLEventIO {
                id: 240,
                value: AVLEventIOValue::U8(0),
            })
            .add_io_event(AVLEventIO {
                id: 21,
                value: AVLEventIOValue::U8(5),
            })
            .add_io_event(AVLEventIO {
                id: 69,
                value: AVLEventIOValue::U8(2),
            })
            .add_io_event(AVLEventIO {
                id: 66,
                value: AVLEventIOValue::U16(11937),
            })
            .add_io_event(AVLEventIO {
                id: 72,
                value: AVLEventIOValue::U32(251),
            })
            .add_io_event(AVLEventIO {
                id: 73,
                value: AVLEventIOValue::U32(0),
            })
            .add_io_event(AVLEventIO {
                id: 74,
                value: AVLEventIOValue::U32(0),
            })
            .add_io_event(AVLEventIO {
                id: 75,
                value: AVLEventIOValue::U32(0),
            })
            .add_io_event(AVLEventIO {
                id: 76,
                value: AVLEventIOValue::U64(5044040395603323408),
            })
            .add_io_event(AVLEventIO {
                id: 77,
                value: AVLEventIOValue::U64(0),
            })
            .add_io_event(AVLEventIO {
                id: 79,
                value: AVLEventIOValue::U64(0),
            })
            .add_io_event(AVLEventIO {
                id: 71,
                value: AVLEventIOValue::U64(0),
            })
            .with_trigger_event_id(0)
            .build()])
        .build();
}

pub fn create_driver_one_card_present_frame(timestamp: DateTime<Utc>) -> AVLFrame {
    return AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::Low)
            .with_timestamp(timestamp)
            .with_angle(0)
            .with_latitude(61.0)
            .with_longitude(27.0)
            .add_io_event(AVLEventIO {
                id: 187,
                value: AVLEventIOValue::U8(1),
            })
            .add_io_event(AVLEventIO {
                id: 184,
                value: AVLEventIOValue::U8(5),
            })
            .add_io_event(AVLEventIO {
                id: 195,
                value: AVLEventIOValue::U64(3544392526090811699),
            })
            .add_io_event(AVLEventIO {
                id: 196,
                value: AVLEventIOValue::U64(3689908453225017393),
            })
            .with_trigger_event_id(195)
            .build()])
        .build();
}

pub fn create_odometer_reading_frame(timestamp: DateTime<Utc>) -> AVLFrame {
    return AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::Low)
            .with_timestamp(timestamp)
            .with_angle(0)
            .with_latitude(61.0)
            .with_longitude(27.0)
            .add_io_event(AVLEventIO {
                id: 192,
                value: AVLEventIOValue::U32(123456),
            })
            .with_trigger_event_id(0)
            .build()])
        .build();
}

pub fn create_speed_frame(timestamp: DateTime<Utc>) -> AVLFrame {
    return AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::Low)
            .with_timestamp(timestamp)
            .with_angle(0)
            .with_latitude(61.0)
            .with_longitude(27.0)
            .add_io_event(AVLEventIO {
                id: 191,
                value: AVLEventIOValue::U32(4000),
            })
            .with_trigger_event_id(0)
            .build()])
        .build();
}

/// Creates a frame that contains only location data
///
/// # Arguments
///
/// * `timestamp` - The timestamp for the frame
///
/// # Returns
///
/// A new AVL frame containing only location data.
pub fn create_location_frame(timestamp: DateTime<Utc>) -> AVLFrame {
    return AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::Low)
            .with_timestamp(timestamp)
            .with_angle(0)
            .with_latitude(61.0)
            .with_longitude(27.0)
            .with_trigger_event_id(0)
            .build()])
        .build();
}
