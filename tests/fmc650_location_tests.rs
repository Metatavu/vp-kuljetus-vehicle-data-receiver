mod test_utils;

use chrono::DateTime;
use tokio::io::AsyncWriteExt;

use uuid::Uuid;
use vp_kuljetus_vehicle_data_receiver::utils::imei::get_random_imei;

use test_utils::tms_services_test_container::TmsServicesTestContainer;

use crate::test_utils::avl_test_utils::create_location_frame;
use crate::test_utils::data_receiver_test_container::DataReceiverTestContainer;
use crate::test_utils::mysql_test_container::MySqlTestContainer;

fn setup_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .target(env_logger::Target::Stdout)
        .try_init();
}

/// Tests for location with erroneous response from the server
#[tokio::test]
async fn test_fmc650_location_with_error_response() {
    setup_logging();

    let imei = get_random_imei();
    let truck_id = Uuid::new_v4().to_string();

    let mut mysql_test_container = MySqlTestContainer::new();
    mysql_test_container.start().await;

    let mut api_services_test_container = TmsServicesTestContainer::new();
    api_services_test_container.start().await;

    // Add mock for trackable

    api_services_test_container
        .mock_get_trackable(imei.as_str(), &truck_id, "TRUCK")
        .await;

    // Mock the creation of a truck location as failure
    api_services_test_container
        .mock_create_truck_location(truck_id.clone(), 500)
        .await;

    let mut data_receiver_test_container = DataReceiverTestContainer::new();
    data_receiver_test_container.start().await;

    let start_time = DateTime::parse_from_rfc3339("2023-10-01T12:00:00+00:00")
        .unwrap()
        .to_utc();

    let mut fmc650_tcp_stream = data_receiver_test_container.get_tcp_stream_fmc650().await;

    data_receiver_test_container
        .send_imei_packet(&mut fmc650_tcp_stream, &imei)
        .await;

    // Send a driver one frame with card present

    data_receiver_test_container
        .send_avl_frame(&mut fmc650_tcp_stream, &create_location_frame(start_time))
        .await
        .unwrap();

    // Wait until all odometer readings are processed
    api_services_test_container.wait_for_location(1, &truck_id).await;

    // Assert that all events requests were processed as failures
    let failed_events = mysql_test_container.count_failed_events().await.unwrap();
    assert_eq!(failed_events, 1);

    // Change location mapping to respond with 200

    api_services_test_container
        .mock_create_truck_location(truck_id.clone(), 200)
        .await;
    api_services_test_container.reset_counts().await;

    // Wait until failed events are processed
    api_services_test_container.wait_for_location(1, &truck_id).await;

    // Assert that all readings were processed as successes
    assert_eq!(mysql_test_container.count_failed_events().await.unwrap(), 0);

    fmc650_tcp_stream.shutdown().await.ok();

    api_services_test_container.stop().await;
    data_receiver_test_container.stop().await;

    mysql_test_container.stop().await;
}
