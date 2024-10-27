use std::{path::PathBuf, time::Duration};

use lazy_static::lazy_static;
use log::debug;
use nom_teltonika::AVLFrame;
use rand::{thread_rng, Rng};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::Receiver,
    time::sleep,
};

use crate::{
    telematics_cache::cache_handler::{
        CacheHandler, DEFAULT_PURGE_CHUNK_SIZE, PURGE_CHUNK_SIZE_ENV_KEY,
    },
    teltonika::records::TeltonikaRecordsHandler,
    utils::read_env_variable_with_default_value,
};

lazy_static! {
    static ref WORKER_RUNTIME: Runtime = Builder::new_multi_thread()
        .thread_name("worker-pool")
        .enable_all()
        .build()
        .unwrap();
}

pub enum WorkerMessage {
    IncomingFrame {
        frame: AVLFrame,
        truck_id: Option<String>,
        base_cache_path: PathBuf,
        imei: String,
    },
}

pub fn spawn(mut receiver_channel: Receiver<WorkerMessage>) {
    WORKER_RUNTIME.spawn(async move {
        while let Some(msg) = receiver_channel.recv().await {
            match msg {
                WorkerMessage::IncomingFrame {
                    frame,
                    truck_id,
                    base_cache_path,
                    imei,
                } => handle_incoming_frame(frame, truck_id, base_cache_path, imei),
            }
        }
    });
}

fn handle_incoming_frame(
    frame: AVLFrame,
    truck_id: Option<String>,
    base_cache_path: PathBuf,
    imei: String,
) {
    tokio::spawn(async move {
        let identifier: u32 = thread_rng().gen();
        let log_target = imei.clone() + "-" + identifier.to_string().as_str();
        debug!(target: &log_target, "Worker spawned for frame with {} records", frame.records.len());
        TeltonikaRecordsHandler::new(
            log_target.clone(),
            truck_id.clone(),
            base_cache_path.clone(),
        )
        .handle_records(frame.records)
        .await;
        debug!(target: &log_target, "Worker finished processing frame");
        if truck_id.is_some() {
            let purge_cache_size = read_env_variable_with_default_value(
                PURGE_CHUNK_SIZE_ENV_KEY,
                DEFAULT_PURGE_CHUNK_SIZE,
            );
            debug!(target: &log_target, "Purging cache for truck {}", truck_id.clone().unwrap());
            CacheHandler::new(log_target.clone(), truck_id.unwrap(), base_cache_path)
                .purge_cache(purge_cache_size)
                .await;
            debug!(target: &log_target, "Worker finished purging cache",);
        }
        sleep(Duration::from_secs(5)).await;
        debug!(target: &log_target, "Worker finished processing frame");
    });
}

#[cfg(test)]
mod tests {
    use std::{env::temp_dir, path::Path, time::Duration};

    use futures::future::join_all;
    use nom_teltonika::{AVLEventIO, Priority};
    use tokio::time::sleep;
    use vehicle_management_service::models::TruckSpeed;

    use crate::{
        telematics_cache::Cacheable,
        utils::{
            avl_frame_builder::AVLFrameBuilder,
            avl_record_builder::avl_record_builder::AVLRecordBuilder,
            test_utils::{get_temp_dir, get_temp_dir_path, wait_until},
        },
    };

    #[tokio::test]
    async fn test_worker() {
        let temp_dir = get_temp_dir();
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
            truck_id: None,
            base_cache_path: temp_dir.path().to_path_buf().clone(),
            imei: "123456789012345".to_string(),
        })
        .await
        .unwrap();

        let cache = wait_until(|| {
            let (cache, cache_size) =
                TruckSpeed::read_from_file(&temp_dir.path().to_str().unwrap(), 0);
            return (cache_size == 1, cache);
        });
        assert_eq!(cache.first().unwrap().speed, 10_f32);
    }

    #[tokio::test]
    async fn test_worker_load() {
        let record_amount = 1000;
        let temp_dir = get_temp_dir();
        let (tx, rx) = tokio::sync::mpsc::channel(1);
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
            truck_id: None,
            base_cache_path: temp_dir.path().to_path_buf().clone(),
            imei: "123456789012345".to_string(),
        })
        .await
        .unwrap();

        let cache_size = wait_until(|| {
            let (_, cache_size) = TruckSpeed::read_from_file(&temp_dir.path().to_str().unwrap(), 0);
            return (cache_size == 1000, cache_size);
        });

        assert_eq!(cache_size, 1000);
    }

    #[tokio::test]
    async fn test_worker_parallel_load() {
        let record_amount = 10;
        let temp_dir = get_temp_dir_path();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
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
        let mut handles = Vec::new();
        for i in 0..10 {
            let tx = tx.clone();
            let packet = packet.clone();
            let temp_dir = temp_dir.clone();
            handles.push(tokio::spawn(async move {
                sleep(Duration::from_secs(i + 1)).await;
                tx.send(super::WorkerMessage::IncomingFrame {
                    frame: packet,
                    truck_id: None,
                    base_cache_path: Path::new(&temp_dir).to_path_buf(),
                    imei: "123456789012345".to_string(),
                })
                .await
                .unwrap();
            }));
        }
        join_all(handles).await;

        let cache_size = wait_until(|| {
            let (_, cache_size) = TruckSpeed::read_from_file(&temp_dir, 0);
            println!("Cache size: {}", cache_size);
            return (cache_size == 100, cache_size);
        });

        assert_eq!(cache_size, 100);
    }
}
