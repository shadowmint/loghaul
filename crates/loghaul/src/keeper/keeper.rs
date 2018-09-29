use Stream;
use KeeperConfig;
use std::thread;
use std::sync::mpsc;
use std::sync::mpsc::Sender;
use keeper::keeper_log::create_keeper_log;
use keeper::keeper_log::KeeperLog;
use keeper::KeeperLogEntry;
use LoghaulError;
use std::thread::JoinHandle;
use keeper::internal::internal_log_channel::InternalKeeperLogSender;
use keeper::internal::internal_log_channel::InternalKeeperLogReceiver;
use keeper::internal::internal_noop_log::InternalNoOpKeeperLog;
use keeper::internal::internal_stream_worker::InternalStreamWorker;

/// Keeper looks after a stream, and acts as a managed runtime to dispatch events through the stream.
pub struct Keeper {
    halt: Option<Sender<()>>,
    logger: Option<InternalKeeperLogSender>,
    log_keeper: Option<InternalKeeperLogReceiver>,
    join_handle: Option<JoinHandle<()>>,
}

impl Keeper {
    pub fn new(stream: Stream, config: Option<KeeperConfig>) -> Keeper {
        let mut rtn = Keeper {
            halt: None,
            logger: None,
            log_keeper: None,
            join_handle: None,
        };
        rtn.start(stream, config.unwrap_or(Default::default()));
        return rtn;
    }

    /// Flush the log keeper to track object state
    /// It's not necessary to call this,
    pub fn step(&mut self) -> Result<(), LoghaulError> {
        match self.log_keeper.as_mut() {
            Some(ref mut k) => {
                return k.step(false);
            }
            None => {}
        };
        return Ok(());
    }

    pub fn halt(&mut self) {
        // Stop the worker
        if self.halt.is_some() {
            self.log(KeeperLogEntry::KeeperHaltStarted);
            let halter = self.halt.take().unwrap();
            match halter.send(()) {
                Ok(_) => {}
                Err(_) => {
                    self.log(KeeperLogEntry::KeeperSendWorkerHaltError);
                }
            }
            self.log(KeeperLogEntry::KeeperHalted);
        }

        // Wait for the thread to halt
        match self.join_handle.take() {
            Some(x) => {
                match x.join() {
                    Ok(_) => {}
                    Err(_) => {
                        self.log(KeeperLogEntry::KeeperWaitWorkerError);
                    }
                }
            }
            None => {}
        }

        // Flush the logger
        match self.log_keeper {
            Some(ref mut k) => {
                k.wait();
            }
            None => {}
        };
    }

    fn start(&mut self, stream: Stream, mut config: KeeperConfig) {
        let remote_logger = self.setup_logger(&mut config);
        self.setup_worker(stream, remote_logger, config);
        self.log(KeeperLogEntry::KeeperStarted);
    }

    fn setup_worker(&mut self, stream: Stream, logger: InternalKeeperLogSender, config: KeeperConfig) {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            InternalStreamWorker::new(config, logger, stream).run(rx);
        });

        self.join_handle = Some(handle);
        self.halt = Some(tx);
    }

    fn setup_logger(&mut self, config: &mut KeeperConfig) -> InternalKeeperLogSender {
        let logger = config.logger.take();
        let (sx, rx) = create_keeper_log(logger.unwrap_or(Box::new(InternalNoOpKeeperLog {})));
        let remote_logger = sx.clone();
        self.logger = Some(sx);
        self.log_keeper = Some(rx);
        return remote_logger;
    }

    fn log(&mut self, entry: KeeperLogEntry) {
        match self.logger.as_mut() {
            Some(ref mut l) => { l.log(entry); }
            None => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Keeper;
    use Stream;
    use mock::MockSource;
    use streams::stream_entry::StreamEntry;
    use LoghaulError;
    use mock::MockTarget;
    use KeeperConfig;
    use mock::MockKeeperLog;
    use std::sync::Mutex;
    use std::sync::Arc;
    use keeper::KeeperLogEntry;
    use std::time::Duration;
    use std::str::from_utf8;
    use std::sync::Barrier;
    use keeper::keeper_config::KeeperEofStrategy;

    #[test]
    fn test_keeper() {
        let barrier = Arc::new(Barrier::new(2));
        let barrier_remote = barrier.clone();
        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let results_bucket = results.clone();

        let stream = Stream::new()
            .with_source(MockSource::new(vec!("1", "2", "3", "4", "5")))
            .with_target(MockTarget::new(move |value, buffer| -> Result<(), LoghaulError> {
                match value {
                    StreamEntry::Data => {
                        match from_utf8(buffer) {
                            Ok(svalue) => {
                                if svalue.len() > 0 {
                                    results_bucket.lock().unwrap().push(svalue.to_string());
                                    if svalue == "5" {
                                        barrier_remote.wait();
                                    }
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }));

        let mut keeper_log = Arc::new(Mutex::new((Vec::new())));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::DropSource,
            logger: Some(Box::new(MockKeeperLog::new(keeper_log.clone()))),
        }));

        barrier.wait();
        keeper.halt();

        let logs: Vec<KeeperLogEntry> = MockKeeperLog::convert_to_vec(keeper_log);
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperStarted));
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperHalted));
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperWorkerThreadHalted));
        assert_eq!(results.lock().unwrap().len(), 5);
    }

    #[test]
    fn test_keeper_resume_eof() {
        let barrier = Arc::new(Barrier::new(2));
        let barrier_remote = barrier.clone();
        let results = Arc::new(Mutex::new(Vec::<String>::new()));
        let results_bucket = results.clone();

        let stream = Stream::new()
            .with_source(MockSource::closed(vec!("1", "2", "3", "4", "5")))
            .with_target(MockTarget::new(move |value, buffer| -> Result<(), LoghaulError> {
                println!("GOT {:?}", value);
                match value {
                    StreamEntry::Data => {
                        match from_utf8(buffer) {
                            Ok(svalue) => {
                                println!("GOT {:?}", svalue);
                                match results_bucket.lock() {
                                    Ok(ref mut bucket) => {
                                        bucket.push(svalue.to_string());
                                        if bucket.len() == 15 {
                                            barrier_remote.wait();
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                }
                return Ok(());
            }));

        let keeper_log = Arc::new(Mutex::new((Vec::new())));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::ResumeSourceAfterCooldown(Duration::from_millis(50)),
            logger: Some(Box::new(MockKeeperLog::new(keeper_log.clone()))),
        }));

        barrier.wait();
        keeper.halt();

        let logs: Vec<KeeperLogEntry> = MockKeeperLog::convert_to_vec(keeper_log);
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperStarted));
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperHalted));
        assert!(logs.iter().any(|v| *v == KeeperLogEntry::KeeperWorkerThreadHalted));
        assert_eq!(results.lock().unwrap().iter().filter(|i| *i == "").count(), 0);
        assert_eq!(results.lock().unwrap().iter().filter(|i| *i == "1").count(), 3);
        assert_eq!(results.lock().unwrap().iter().filter(|i| *i == "5").count(), 3);
        assert_eq!(results.lock().unwrap().len(), 15);
    }
}