use std::path::PathBuf;

use lazy_static::lazy_static;
use log::debug;
use nom_teltonika::AVLFrame;
use rand::{thread_rng, Rng};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::Receiver,
};
use vehicle_management_service::models::Trackable;

use crate::{
    telematics_cache::cache_handler::{CacheHandler, DEFAULT_PURGE_CHUNK_SIZE, PURGE_CHUNK_SIZE_ENV_KEY},
    teltonika::records::TeltonikaRecordsHandler,
    utils::read_env_variable_with_default_value,
};

lazy_static! {
    /// Multi-threaded Tokio runtime for the worker pool
    ///
    /// The worker pool is responsible for processing incoming or cached AVL frames on the background.
    static ref WORKER_RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("worker-pool")
        .enable_all()
        .build()
        .unwrap();
}

/// Message that is sent to the worker pool
pub enum WorkerMessage {
    IncomingFrame {
        frame: AVLFrame,
        trackable: Option<Trackable>,
        base_cache_path: PathBuf,
        imei: String,
    },
}

/// Spawns a future that listens for incoming messages on the receiver channel
///
/// This is called once a new connection is established and we start receiving records from the device.
/// A multi-products single-consumer (MPSC) channel is created, receiver is passed to this function and the sender is used to send messages from the connection handler to the worker pool.
pub fn spawn(mut receiver_channel: Receiver<WorkerMessage>) {
    WORKER_RUNTIME.spawn(async move {
        while let Some(msg) = receiver_channel.recv().await {
            match msg {
                WorkerMessage::IncomingFrame {
                    frame,
                    trackable,
                    base_cache_path,
                    imei,
                } => handle_incoming_frame(frame, trackable, base_cache_path, imei),
            }
        }
    });
}

/// Handles an incoming frame, a callback for [WorkerMessage::IncomingFrame]
///
/// This function spawns a new asynchronous Tokio task that processes the incoming frame and purges the cache if a truck_id is provided.
fn handle_incoming_frame(frame: AVLFrame, trackable: Option<Trackable>, base_cache_path: PathBuf, imei: String) {
    tokio::spawn(async move {
        let identifier: u32 = thread_rng().gen();
        let log_target = imei.clone() + "-" + identifier.to_string().as_str();

        debug!(target: &log_target, "Worker spawned for frame with {} records", frame.records.len());

        TeltonikaRecordsHandler::new(log_target.clone(), trackable.clone(), base_cache_path.clone())
            .handle_records(frame.records)
            .await;

        debug!(target: &log_target, "Worker finished processing incoming frame");

        if let Some(trackable) = trackable {
            let purge_cache_size =
                read_env_variable_with_default_value(PURGE_CHUNK_SIZE_ENV_KEY, DEFAULT_PURGE_CHUNK_SIZE);
            debug!(target: &log_target, "Purging cache for trackable {}", trackable.id.clone());
            CacheHandler::new(log_target.clone(), trackable, base_cache_path)
                .purge_cache(purge_cache_size)
                .await;
            debug!(target: &log_target, "Worker finished purging cache",);
        }

        debug!(target: &log_target, "Worker finished purging cache");
    });
}

#[cfg(test)]
mod tests {
    use nom_teltonika::{AVLEventIO, Priority};
    use vehicle_management_service::models::TruckSpeed;

    use crate::{
        telematics_cache::Cacheable,
        utils::{
            avl_frame_builder::AVLFrameBuilder,
            avl_record_builder::avl_record_builder::AVLRecordBuilder,
            imei::get_random_imei,
            test_utils::{get_temp_dir_path, wait_until},
        },
    };

    #[tokio::test]
    async fn test_worker() {
        let temp_dir = get_temp_dir_path();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
        super::spawn(rx);
        let record = AVLRecordBuilder::new()
            .with_priority(Priority::High)
            .with_io_events(vec![AVLEventIO {
                id: 191,
                value: nom_teltonika::AVLEventIOValue::U16(10),
            }])
            .build();
        let packet = AVLFrameBuilder::new().add_record(record).build();
        tx.send(super::WorkerMessage::IncomingFrame {
            frame: packet,
            trackable: None,
            base_cache_path: temp_dir.clone(),
            imei: "123456789012345".to_string(),
        })
        .await
        .unwrap();

        let cache = wait_until(|| {
            let (cache, cache_size) = TruckSpeed::read_from_file(temp_dir.clone(), 0);
            return (cache_size == 1, cache);
        });
        assert_eq!(cache.first().unwrap().speed, 10_f32);
    }

    #[tokio::test]
    async fn test_worker_under_load() {
        let record_amount = 1000;
        let imei = get_random_imei();
        let temp_dir = get_temp_dir_path();
        let (tx, rx) = tokio::sync::mpsc::channel(1000);
        super::spawn(rx);
        let mut records = Vec::new();
        for i in 0..record_amount {
            let record = AVLRecordBuilder::new()
                .with_priority(Priority::High)
                .with_io_events(vec![AVLEventIO {
                    id: 191,
                    value: nom_teltonika::AVLEventIOValue::U16(i),
                }])
                .build();
            records.push(record);
        }
        let packet = AVLFrameBuilder::new().with_records(records).build();
        tx.send(super::WorkerMessage::IncomingFrame {
            frame: packet,
            trackable: None,
            base_cache_path: temp_dir.clone(),
            imei,
        })
        .await
        .unwrap();

        let cache_size = wait_until(|| {
            let (_, cache_size) = TruckSpeed::read_from_file(temp_dir.clone(), 0);
            return (cache_size == 1000, cache_size);
        });

        assert_eq!(cache_size, 1000);
    }
}
