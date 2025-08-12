mod test_utils;

use chrono::{DateTime, Duration, Utc};
use log::info;
use nom_teltonika::{AVLEventIO, AVLEventIOValue, AVLFrame, Priority};
use tokio::io::AsyncWriteExt;

use vp_kuljetus_vehicle_data_receiver::utils::avl_frame_builder::AVLFrameBuilder;
use vp_kuljetus_vehicle_data_receiver::utils::avl_record_builder::avl_record_builder::AVLRecordBuilder;
use vp_kuljetus_vehicle_data_receiver::utils::imei::get_random_imei;

use test_utils::tms_services_test_container::TmsServicesTestContainer;

use crate::test_utils::data_receiver_test_container::DataReceiverTestContainer;

fn create_frame_with_temperature(timestamp: DateTime<Utc>) -> AVLFrame {
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

/// Test for single temperature reading from FMC 234
/// This test sends a frame with a temperature reading and checks if the reading is correctly processed and stored.
#[tokio::test]
async fn test_fmc234_single_temperature() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .try_init();

    let imei = get_random_imei();

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;

    api_services_test_container.mock_create_temperature_reading().await;
    api_services_test_container.mock_get_trackable(imei.as_str()).await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

    let timestamp = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    data_receiver_test_container
        .send_imei_packet(&mut fmc234_tcp_stream, &imei)
        .await;

    let frame_with_temperature = create_frame_with_temperature(timestamp);

    info!("Sending frame with temperature: {:?}", frame_with_temperature);

    data_receiver_test_container
        .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
        .await;

    let reading_count = api_services_test_container.wait_for_temperature_reading(1).await;
    assert_eq!(1, reading_count, "Expected {} temperature readings to be sent", 1);

    fmc234_tcp_stream.shutdown().await.ok();

    info!("Connection to FMC 234 server closed");
}

/// Test for multiple temperature readings from FMC 234
/// This test sends multiple frames with temperature readings and checks if all readings are correctly processed and stored.
#[tokio::test]
async fn test_fmc234_multiple_temperatures() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .try_init();

    let imei = get_random_imei();

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading().await;
    api_services_test_container.mock_get_trackable(imei.as_str()).await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

    data_receiver_test_container
        .send_imei_packet(&mut fmc234_tcp_stream, &imei)
        .await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    for i in 0..100 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_frame_with_temperature(timestamp);
        info!("Sending frame with temperature: {:?}", frame_with_temperature);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await;
    }

    let reading_count = api_services_test_container.wait_for_temperature_reading(100).await;
    assert_eq!(100, reading_count, "Expected {} temperature readings to be sent", 100);

    fmc234_tcp_stream.shutdown().await.ok();

    info!("Connection to FMC 234 server closed");
}

/// Test for multiple temperature readings from FMC 234 with poor connection.
/// Poor connection is simulated by disconnecting the TCP stream after sending the frames.
#[tokio::test]
async fn test_fmc234_multiple_temperatures_with_poor_connection() {
    let _ = env_logger::builder()
        .is_test(true)
        .filter_level(log::LevelFilter::Trace)
        .target(env_logger::Target::Stdout)
        .try_init();

    let imei = get_random_imei();

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading().await;
    api_services_test_container.mock_get_trackable(imei.as_str()).await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    for i in 0..100 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_frame_with_temperature(timestamp);

        let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

        data_receiver_test_container
            .send_imei_packet(&mut fmc234_tcp_stream, &imei)
            .await;

        info!("Sending frame with temperature: {:?}", frame_with_temperature);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await;

        fmc234_tcp_stream.shutdown().await.ok();
    }

    let reading_count = api_services_test_container.wait_for_temperature_reading(100).await;
    assert_eq!(100, reading_count, "Expected {} temperature readings to be sent", 100);

    info!("Connection to FMC 234 server closed");
}
