use keeper::internal::internal_log_channel::InternalKeeperLogSender;
use keeper::KeeperLogEntry;
use keeper::KeeperLog;
use std::thread;
use std::sync::mpsc::TryRecvError;
use Stream;
use std::sync::mpsc::Receiver;
use KeeperConfig;
use LoghaulError;
use LoghaulErrorCode;
use keeper::internal::internal_source_cooler::InternalSourceCooler;
use Source;
use std::error::Error;

pub struct InternalStreamWorker {
    config: KeeperConfig,
    cooler: InternalSourceCooler,
    logger: InternalKeeperLogSender,
    stream: Stream,
}

impl InternalStreamWorker {
    pub fn new(config: KeeperConfig, logger: InternalKeeperLogSender, stream: Stream) -> InternalStreamWorker {
        return InternalStreamWorker {
            cooler: InternalSourceCooler::new(config.eof_strategy),
            config,
            logger,
            stream,
        };
    }

    pub fn run(&mut self, halt_channel: Receiver<()>) {
        self.logger.log(KeeperLogEntry::KeeperWorkerThreadStarted);
        let mut eof:Vec<Box<Source + Send + 'static>> = Vec::new();

        // Poll each source forever, pushing to each target for every input.
        loop {
            thread::sleep(self.config.interval);
            match self.stream.step(&mut eof) {
                Ok(_) => {}
                Err(err) => {
                    self.logger.log(KeeperLogEntry::KeeperError(LoghaulError::from(LoghaulErrorCode::SourceErr(err.description().to_string()))));
                }
            }

            // If we got an EOF sources, deal with them.
            if eof.len() > 0 {
                println!("Got an EOF source");
                eof.into_iter().for_each(|i| {
                    println!("Item PUSH!");
                    self.cooler.push(i);
                });
                eof = Vec::new();
            }

            // If we have any new resumed streams, load them
            match self.cooler.resume() {
                Some(resumed) => {
                    println!("Reloaded some source");
                    resumed.into_iter().for_each(|i| self.stream.add_boxed_source(i));
                },
                None => {}
            }

            // Check if we received a halt signal
            match halt_channel.try_recv() {
                Ok(_) | Err(TryRecvError::Disconnected) => {
                    self.logger.log(KeeperLogEntry::KeeperWorkerThreadHalted);
                    break;
                }
                Err(TryRecvError::Empty) => {}
            }
        }
    }
}