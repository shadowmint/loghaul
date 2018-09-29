use keeper::KeeperLog;
use keeper::KeeperLogEntry;

pub struct InternalNoOpKeeperLog {}

impl KeeperLog for InternalNoOpKeeperLog {
    fn log(&mut self, _record: KeeperLogEntry) {}
}