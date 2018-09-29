use std::default::Default;
use keeper::KeeperLog;
use std::time::Duration;

/// How to deal with sources that EOF
#[derive(Copy, Clone, Debug)]
pub enum KeeperEofStrategy {
    DropSource,
    ResumeSourceAfterCooldown(Duration),
}

pub struct KeeperConfig {
    pub interval: Duration,
    pub logger: Option<Box<KeeperLog + Send>>,
    pub eof_strategy: KeeperEofStrategy,
}

impl Default for KeeperConfig {
    fn default() -> Self {
        return KeeperConfig {
            interval: Duration::from_millis(100),
            logger: None,
            eof_strategy: KeeperEofStrategy::DropSource,
        };
    }
}