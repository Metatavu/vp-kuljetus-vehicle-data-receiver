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

pub struct Worker;

impl Worker {
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
