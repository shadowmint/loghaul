use LoghaulError;
use std::time::Duration;
use errors::loghaul_error::LoghaulErrorCode;
use std::sync::mpsc::Sender;
use keeper::KeeperLogEntry;
use std::sync::mpsc::Receiver;
use keeper::KeeperLog;

pub struct InternalKeeperLogSender {
    pub channel: Sender<KeeperLogEntry>
}

pub struct InternalKeeperLogReceiver {
    pub channel: Receiver<KeeperLogEntry>,
    pub handler: Box<KeeperLog>,
}

impl KeeperLog for InternalKeeperLogSender {
    fn log(&mut self, record: KeeperLogEntry) {
        match self.channel.send(record) {
            Ok(_) => {}
            Err(_) => {}
        }
    }
}

impl InternalKeeperLogReceiver {
    /// Blocking operation that processes logs forever
    pub fn wait(&mut self) {
        loop {
            match self.step(true) {
                Ok(_) => {}
                Err(_) => {
                    return;
                }
            }
        }
    }

    /// Perform a single step processing logs
    pub fn step(&mut self, block_for_response: bool) -> Result<(), LoghaulError> {
        let entry = if block_for_response {
            // Block for up to 1 second; this is only done when we're shutting donn the keeper.
            match self.channel.recv_timeout(Duration::from_millis(1000)) {
                Ok(e) => e,
                Err(_) => {
                    return Err(LoghaulError::from(LoghaulErrorCode::NotImplemented));
                }
            }
        } else {
            match self.channel.try_recv() {
                Ok(e) => e,
                Err(_) => {
                    return Ok(());
                }
            }
        };

        self.handler.log(entry);
        Ok(())
    }
}

impl Clone for InternalKeeperLogSender {
    fn clone(&self) -> Self {
        InternalKeeperLogSender {
            channel: self.channel.clone()
        }
    }
}