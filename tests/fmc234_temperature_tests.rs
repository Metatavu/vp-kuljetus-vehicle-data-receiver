mod test_utils;

use chrono::{DateTime, Duration};
use log::info;
use tokio::io::AsyncWriteExt;

use uuid::Uuid;
use vp_kuljetus_vehicle_data_receiver::utils::imei::get_random_imei;

use test_utils::tms_services_test_container::TmsServicesTestContainer;

use crate::test_utils::avl_test_utils::create_temperature_frame;
use crate::test_utils::data_receiver_test_container::DataReceiverTestContainer;
use crate::test_utils::mysql_test_container::MySqlTestContainer;

fn setup_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .target(env_logger::Target::Stdout)
        .try_init();
}

/// Test for single temperature reading from FMC 234
/// This test sends a frame with a temperature reading and checks if the reading is correctly processed and stored.
#[tokio::test]
async fn test_fmc234_single_temperature() {
    setup_logging();

    let towable_id = Uuid::new_v4().to_string();
    let imei = get_random_imei();
    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;

    api_services_test_container.mock_create_temperature_reading(200).await;
    api_services_test_container
        .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
        .await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

    let timestamp = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    data_receiver_test_container
        .send_imei_packet(&mut fmc234_tcp_stream, &imei)
        .await;

    let frame_with_temperature = create_temperature_frame(timestamp);

    data_receiver_test_container
        .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
        .await
        .unwrap();

    let reading_count = api_services_test_container.wait_for_temperature_reading(1).await;
    assert_eq!(1, reading_count, "Expected {} temperature readings to be sent", 1);

    fmc234_tcp_stream.shutdown().await.ok();

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;
    mysql_test_container.stop().await;
}

/// Test for multiple temperature readings from FMC 234
/// This test sends multiple frames with temperature readings and checks if all readings are correctly processed and stored.
#[tokio::test]
async fn test_fmc234_multiple_temperatures() {
    setup_logging();

    let imei = get_random_imei();
    let towable_id = Uuid::new_v4().to_string();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading(200).await;
    api_services_test_container
        .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
        .await;

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
        let frame_with_temperature = create_temperature_frame(timestamp);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await
            .unwrap();
    }

    let reading_count = api_services_test_container.wait_for_temperature_reading(100).await;
    assert_eq!(100, reading_count, "Expected {} temperature readings to be sent", 100);

    fmc234_tcp_stream.shutdown().await.ok();

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;
    mysql_test_container.stop().await;
}

/// Test for multiple temperature readings from FMC 234 with poor connection.
/// Poor connection is simulated by disconnecting the TCP stream after sending the frames.
#[tokio::test]
async fn test_fmc234_multiple_temperatures_with_poor_connection() {
    setup_logging();

    let imei = get_random_imei();
    let towable_id = Uuid::new_v4().to_string();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading(200).await;
    api_services_test_container
        .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
        .await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    for i in 0..100 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_temperature_frame(timestamp);

        let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

        data_receiver_test_container
            .send_imei_packet(&mut fmc234_tcp_stream, &imei)
            .await;

        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await
            .unwrap();

        fmc234_tcp_stream.shutdown().await.ok();
    }

    let reading_count = api_services_test_container.wait_for_temperature_reading(100).await;
    assert_eq!(100, reading_count, "Expected {} temperature readings to be sent", 100);

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;
    mysql_test_container.stop().await;
}

/// Tests for sending temperature readings from multiple FMC 234 devices simultaneously.
#[tokio::test]
async fn test_fmc234_multiple_devices_temperature() {
    setup_logging();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading(200).await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let mut streams = Vec::new();

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    for _i in 0..10 {
        let imei = get_random_imei();
        let towable_id = Uuid::new_v4().to_string();
        api_services_test_container
            .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
            .await;
        let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

        data_receiver_test_container
            .send_imei_packet(&mut fmc234_tcp_stream, &imei)
            .await;

        streams.push(fmc234_tcp_stream);
    }

    for i in 0..100 {
        for stream in streams.iter_mut() {
            let timestamp = start_time + Duration::seconds(i);
            let frame_with_temperature = create_temperature_frame(timestamp);

            data_receiver_test_container
                .send_avl_frame(stream, &frame_with_temperature)
                .await
                .unwrap();
        }
    }

    for stream in streams.iter_mut() {
        stream.shutdown().await.ok();
    }

    // Wait for all temperature readings to be processed (10 devices with 100 frames = 1000 readings)
    let reading_count = api_services_test_container.wait_for_temperature_reading(1000).await;
    assert_eq!(1000, reading_count, "Expected {} temperature readings to be sent", 100);

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;
    mysql_test_container.stop().await;
}

/// Tests for sending temperature readings with erroneous data in stream
#[tokio::test]
async fn test_fmc234_temperature_with_error() {
    setup_logging();

    let imei = get_random_imei();
    let towable_id = Uuid::new_v4().to_string();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading(200).await;
    api_services_test_container
        .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
        .await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

    data_receiver_test_container
        .send_imei_packet(&mut fmc234_tcp_stream, &imei)
        .await;

    info!("Sending 10 frames with temperature readings");

    // Send 10 frams with temperature readings

    for i in 0..10 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_temperature_frame(timestamp);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await
            .unwrap();
    }

    info!("Sending garbage data");

    let garbage_data = vec![0x01, 0x02, 0x03, 0x04, 0x05];
    fmc234_tcp_stream.write_all(&garbage_data).await.unwrap();

    let failing_frame = create_temperature_frame(start_time + Duration::seconds(11));

    let failing_frame_result = data_receiver_test_container
        .send_avl_frame(&mut fmc234_tcp_stream, &failing_frame)
        .await;

    assert!(
        failing_frame_result.is_err(),
        "Expected error when sending frame with garbage data"
    );

    for i in 11..20 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_temperature_frame(timestamp);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await
            .unwrap();
    }

    fmc234_tcp_stream.shutdown().await.ok();

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;
    mysql_test_container.stop().await;
}

/// Tests for sending temperature readings with errorneous response from the server
#[tokio::test]
async fn test_fmc234_temperature_with_error_response() {
    setup_logging();

    let imei = get_random_imei();
    let towable_id = Uuid::new_v4().to_string();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;
    api_services_test_container.mock_create_temperature_reading(500).await;
    api_services_test_container
        .mock_get_trackable(imei.as_str(), &towable_id, "TOWABLE")
        .await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    let mut fmc234_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc234().await;

    data_receiver_test_container
        .send_imei_packet(&mut fmc234_tcp_stream, &imei)
        .await;

    info!("Sending 10 frames with temperature readings");

    // Send 10 frames with temperature readings
    for i in 0..10 {
        let timestamp = start_time + Duration::seconds(i);
        let frame_with_temperature = create_temperature_frame(timestamp);
        data_receiver_test_container
            .send_avl_frame(&mut fmc234_tcp_stream, &frame_with_temperature)
            .await
            .unwrap();
    }

    // Wait until all temperature readings are processed
    api_services_test_container.wait_for_temperature_reading(10).await;

    // Assert that all readings were processed as failures
    //assert_eq!(mysql_test_container.count_failed_events().await.unwrap(), 10);

    api_services_test_container.mock_create_temperature_reading(200).await;
    api_services_test_container.reset_counts().await;

    // Wait until new temperature readings and failed events are processed
    api_services_test_container.wait_for_temperature_reading(10).await;

    // Assert that all readings were processed as successes
    //assert_eq!(mysql_test_container.count_failed_events().await.unwrap(), 0);

    fmc234_tcp_stream.shutdown().await.ok();

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;

    mysql_test_container.stop().await;
}
