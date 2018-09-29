use keeper::KeeperLog;
use keeper::KeeperLogEntry;
use std::sync::Mutex;
use std::sync::Arc;

pub struct MockKeeperLog {
    log: Arc<Mutex<Vec<KeeperLogEntry>>>
}

impl MockKeeperLog {
    pub fn new(container: Arc<Mutex<Vec<KeeperLogEntry>>>) -> MockKeeperLog {
        return MockKeeperLog {
            log: container
        };
    }

    pub fn convert_to_vec(container: Arc<Mutex<Vec<KeeperLogEntry>>>) -> Vec<KeeperLogEntry> {
        container.lock().unwrap().iter().map(|e| e.clone()).collect()
    }
}

impl KeeperLog for MockKeeperLog {
    fn log(&mut self, record: KeeperLogEntry) {
        match self.log.lock().as_mut() {
            Ok(ref mut m) => {
                m.push(record);
            }
            Err(_) => {}
        }
    }
}