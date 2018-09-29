use keeper::keeper_config::KeeperEofStrategy;
use Source;
use std::time::Instant;
use std::collections::VecDeque;
use std::time::Duration;

pub struct InternalSourceCooler {
    eof: KeeperEofStrategy,
    cooler: VecDeque<ColdSource>,
}

struct ColdSource {
    hot: bool,
    source: Option<Box<Source + Send + 'static>>,
    last_attempt: Option<Instant>,
}

impl InternalSourceCooler {
    pub fn new(eof: KeeperEofStrategy) -> InternalSourceCooler {
        return InternalSourceCooler {
            eof,
            cooler: VecDeque::new(),
        };
    }

    /// Push a source that has EOF into the cooler
    pub fn push(&mut self, source: Box<Source + Send + 'static>) {
        match self.eof {
            KeeperEofStrategy::DropSource => {}
            KeeperEofStrategy::ResumeSourceAfterCooldown(_) => {
                self.cooler.push_back(
                    ColdSource {
                        hot: false,
                        source: Some(source),
                        last_attempt: None,
                    }
                );
            }
        }
    }

    /// Fetch a set of resumed sources
    pub fn resume(&mut self) -> Option<Vec<Box<Source + Send + 'static>>> {
        if self.reheat_cold_items() {
            return Some(self.remove_hot_items());
        }
        return None;
    }

    fn remove_hot_items(&mut self) -> Vec<Box<Source + Send + 'static>> {
        let rtn = self.cooler.iter_mut().filter(|i| i.hot && i.source.is_some()).map(|i| i.source.take().unwrap()).collect();
        let mut offset = 0;
        let count = self.cooler.len();
        while offset < count {
            offset += 1;
            match self.cooler.pop_front() {
                Some(item) => {
                    if !item.hot {
                        self.cooler.push_back(item);
                    }
                }
                None => {}
            }
        }
        return rtn;
    }

    fn reheat_cold_items(&mut self) -> bool {
        let mut rtn = false;
        for cold_item in self.cooler.iter_mut() {
            if InternalSourceCooler::should_resume(self.eof, cold_item) {
                let mut hot = false;
                match cold_item.source.as_mut() {
                    Some(ref mut source) => {
                        match source.resume() {
                            Ok(_) => {
                                hot = true;
                            }
                            Err(err) => {
                                println!("{:?}", err);
                            }
                        }
                    }
                    None => {}
                }
                cold_item.hot = hot;
                rtn = rtn | hot;
            }
        }
        return rtn;
    }

    fn should_resume(eof: KeeperEofStrategy, source: &mut ColdSource) -> bool {
        if source.source.is_none() {
            return false;
        }

        if source.hot {
            return false;
        }

        match eof {
            KeeperEofStrategy::ResumeSourceAfterCooldown(duration) => {
                if source.elapsed_since_last_attempt(duration) {
                    return InternalSourceCooler::try_resume(source);
                }
            }
            _ => {
                println!("Unsupported eof style");
            }
        }

        return false;
    }

    fn try_resume(source: &mut ColdSource) -> bool {
        source.last_attempt = Some(Instant::now());
        match source.source.as_mut() {
            Some(source_ref) => {
                match source_ref.resume() {
                    Ok(_) => {
                        source.hot = true;
                        return true;
                    }
                    Err(_) => {}
                }
            }
            None => {}
        }
        return false;
    }
}

impl ColdSource {
    fn elapsed_since_last_attempt(&mut self, cooloff: Duration) -> bool {
        return match self.last_attempt {
            Some(instant) => {
                let now = Instant::now();
                let cooloff_expired = (now - instant) > cooloff;
                cooloff_expired
            }
            None => true
        };
    }
}