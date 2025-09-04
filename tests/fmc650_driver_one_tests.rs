mod test_utils;

use chrono::DateTime;
use tokio::io::AsyncWriteExt;

use uuid::Uuid;
use vp_kuljetus_vehicle_data_receiver::utils::imei::get_random_imei;

use test_utils::tms_services_test_container::TmsServicesTestContainer;

use crate::test_utils::avl_test_utils::create_driver_one_card_present_frame;
use crate::test_utils::data_receiver_test_container::DataReceiverTestContainer;
use crate::test_utils::mysql_test_container::MySqlTestContainer;

fn setup_logging() {
    let _ = env_logger::builder()
        .is_test(true)
        .target(env_logger::Target::Stdout)
        .try_init();
}

/// Tests for sending driver one with erroneous response from the server
#[tokio::test]
async fn test_fmc650_driver_one_card_present_with_error_response() {
    // TODO: write a new test that works without the database
}
