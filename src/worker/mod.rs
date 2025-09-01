use lazy_static::lazy_static;
use log::debug;
use nom_teltonika::AVLFrame;
use rand::{thread_rng, Rng};
use sqlx::{MySql, Pool};
use tokio::{
    runtime::{Builder, Runtime},
    sync::mpsc::{error::SendError, Receiver, Sender},
    task::JoinHandle,
};
use vehicle_management_service::models::Trackable;

use crate::{teltonika::records::TeltonikaRecordsHandler, Listener};

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
        trackable: Trackable,
        imei: String,
        listener: Listener,
    },
}

pub struct Worker {
    handle: JoinHandle<()>,
    sender: Sender<WorkerMessage>,
}

impl Worker {
    pub async fn send(&self, msg: WorkerMessage) -> Result<(), SendError<WorkerMessage>> {
        self.sender.send(msg).await
    }
}

pub fn spawn_2(channel: (Sender<WorkerMessage>, Receiver<WorkerMessage>), imei: String) -> Worker {
    debug!(target: &imei, "Spawning worker");
    let (sender, mut receiver) = channel;
    let handle = WORKER_RUNTIME.spawn(async move {
        loop {
            debug!(target: &imei, "Waiting for incoming frame");
            match receiver.recv().await {
                Some(msg) => {
                    debug!(target: &imei, "Received incoming frame");
                    match msg {
                        WorkerMessage::IncomingFrame {
                            frame,
                            trackable,
                            imei,
                            listener,
                        } => handle_incoming_frame(frame, trackable, imei, listener),
                    }
                }
                None => {
                    debug!(target: &imei, "Worker channel closed, exiting worker loop");
                    break;
                }
            }
        }
    });

    Worker { handle, sender }
}

/// Handles an incoming frame, a callback for [WorkerMessage::IncomingFrame]
///
/// This function spawns a new asynchronous Tokio task that processes the incoming frame and purges the cache if a truck_id is provided.
pub fn handle_incoming_frame(frame: AVLFrame, trackable: Trackable, imei: String, listener: Listener) {
    tokio::spawn(async move {
        let identifier: u32 = thread_rng().r#gen();
        let log_target = imei.clone() + "-" + identifier.to_string().as_str();
        let records_handler = TeltonikaRecordsHandler::new(log_target.clone(), trackable.clone(), imei.clone());

        debug!(target: &log_target, "Worker spawned for frame with {} records", frame.records.len());

        records_handler.handle_records(frame.records, &listener).await;

        debug!(target: &log_target, "Worker finished processing incoming frame");

        debug!(target: &log_target, "Processing trackable event for IMEI {}", imei);
    });
}
