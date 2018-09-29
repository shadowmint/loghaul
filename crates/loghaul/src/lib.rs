mod source;
mod target;
mod streams;
mod errors;
mod keeper;

pub mod mock;

pub use source::Source;
pub use target::Target;

pub use streams::stream::Stream;
pub use streams::stream_entry::StreamEntry;
pub use streams::stream_buffer::StreamBuffer;

pub use keeper::keeper::Keeper;
pub use keeper::keeper_config::KeeperConfig;
pub use keeper::keeper_config::KeeperEofStrategy;
pub use keeper::keeper_log::KeeperLogEntry;
pub use keeper::keeper_log::KeeperLog;

pub use errors::loghaul_error::LoghaulError;
pub use errors::loghaul_error::LoghaulErrorCode;
pub use errors::loghaul_error_aggregate::LoghaulErrorAggregate;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
