use std::sync::mpsc;
use LoghaulError;
use keeper::internal::internal_log_channel::InternalKeeperLogReceiver;
use keeper::internal::internal_log_channel::InternalKeeperLogSender;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum KeeperLogEntry {
    KeeperStarted,
    KeeperWorkerThreadStarted,
    KeeperWorkerThreadHalted,
    KeeperHaltStarted,
    KeeperHalted,
    KeeperWaitWorkerError,
    KeeperSendWorkerHaltError,
    KeeperError(LoghaulError),
}

pub trait KeeperLog {
    fn log(&mut self, record: KeeperLogEntry);
}


pub fn create_keeper_log(target: Box<KeeperLog >) -> (InternalKeeperLogSender, InternalKeeperLogReceiver) {
    let (sender, receiver) = mpsc::channel();
    return (
        InternalKeeperLogSender {
            channel: sender
        },
        InternalKeeperLogReceiver {
            channel: receiver,
            handler: target,
        }
    );
}