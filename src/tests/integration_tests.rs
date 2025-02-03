use std::time::Duration;

use nom_teltonika::{AVLEventIO, AVLEventIOValue, Priority};
use tempfile::tempdir;
use tokio_test::io::Builder;

use crate::{
    teltonika::connection::TeltonikaConnection,
    utils::{
        avl_frame_builder::AVLFrameBuilder,
        avl_packet::AVLPacketToBytes,
        avl_record_builder::avl_record_builder::AVLRecordBuilder,
        imei::{build_valid_imei_packet, get_random_imei},
        test_utils::{driver_card_id_to_two_part_events, mock_server, vin_to_three_part_events, MockServerExt},
    },
};

/// This is not an actual integration test, but mimics the behavior of a Teltonika device sending a packet with a driver card ID and then removing it.
///
/// TODO: Refactor most of the tests to be actual integration tests. See [Rust docs](https://doc.rust-lang.org/rust-by-example/testing/integration_testing.html)
#[tokio::test]
async fn test_driver_one_card_removal() {
    let driver_card_id = String::from("1069619335000001");
    let driver_card_events = driver_card_id_to_two_part_events(driver_card_id.clone()).to_vec();
    let vin_events = vin_to_three_part_events(String::from("W1T96302X10704959")).to_vec();
    let _mocks = mock_server().start_all_mocks();
    let imei = build_valid_imei_packet(&get_random_imei());
    let temp_dir = tempdir().unwrap();
    let frame_with_card = AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .add_io_event(AVLEventIO {
                id: 187,
                value: AVLEventIOValue::U8(1),
            })
            .add_io_events(vin_events)
            .add_io_events(driver_card_events)
            .with_trigger_event_id(187)
            .build()])
        .build();

    let frame_without_card = AVLFrameBuilder::new()
        .with_records(vec![AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![AVLEventIO {
                id: 187,
                value: AVLEventIOValue::U8(0),
            }])
            .with_trigger_event_id(187)
            .build()])
        .build();

    let mock_stream = Builder::new()
        .read(&imei)
        .write(b"\x01")
        .read(&frame_with_card.to_bytes())
        .wait(Duration::from_millis(100))
        .write(&(frame_with_card.records.len() as u32).to_be_bytes())
        .wait(Duration::from_millis(100))
        .read(&frame_without_card.to_bytes())
        .wait(Duration::from_millis(100))
        .write(&(frame_without_card.records.len() as u32).to_be_bytes())
        .wait(Duration::from_millis(3_000))
        .read(&frame_without_card.to_bytes())
        .write(&(frame_without_card.records.len() as u32).to_be_bytes())
        .build();
    let result = TeltonikaConnection::handle_connection(mock_stream, temp_dir.path(), 6500).await;

    assert!(result.is_ok());
}
