#[cfg(test)]
extern crate rand;
extern crate loghaul;

mod file_source;
mod file_target;
mod internal;
mod errors;

pub use file_source::FileSource;
pub use file_target::FileTarget;

pub use errors::loghaul_file_error::LoghaulFileError;
pub use errors::loghaul_file_error::LoghaulFileErrorCode;

#[cfg(test)]
mod tests {
    use loghaul::Stream;
    use std::sync::{Arc, Mutex};
    use loghaul::KeeperConfig;
    use loghaul::Keeper;
    use std::time::Duration;
    use loghaul::mock::MockKeeperLog;
    use loghaul::KeeperEofStrategy;
    use internal::file_test_helpers::read_entire_file;
    use std::thread::sleep;
    use internal::file_test_helpers::random_test_file;
    use FileSource;
    use FileTarget;
    use std::thread;

    #[test]
    fn test_many_to_many() {
        let input_path_1 = random_test_file();
        let input_path_2 = random_test_file();
        let input_path_3 = random_test_file();

        let output_path_1 = random_test_file();
        let output_path_2 = random_test_file();

        let stream = Stream::new()
            .with_source(FileSource::new(&input_path_1.path))
            .with_source(FileSource::new(&input_path_2.path))
            .with_source(FileSource::new(&input_path_3.path))
            .with_target(FileTarget::new(&output_path_1.path))
            .with_target(FileTarget::new(&output_path_2.path));

        let log = Arc::new(Mutex::new(Vec::new()));
        let mut keeper = Keeper::new(stream, Some(KeeperConfig {
            interval: Duration::from_millis(1),
            eof_strategy: KeeperEofStrategy::DropSource,
            logger: Some(Box::new(MockKeeperLog::new(log))),
        }));

        // Push data into each file from many threads
        let handle1 = thread::spawn(move || {
            for _ in 1..10 {
                input_path_1.write("Random content to file 1\n");
                sleep(Duration::from_millis(10));
            }
        });

        let handle2 = thread::spawn(move || {
            for _ in 1..10 {
                input_path_2.write("Random content to file 2\n");
                sleep(Duration::from_millis(10));
            }
        });

        let handle3 = thread::spawn(move || {
            for _ in 1..10 {
                input_path_3.write("Random content to file 3\n");
                sleep(Duration::from_millis(10));
            }
        });

        handle1.join().unwrap();
        handle2.join().unwrap();
        handle3.join().unwrap();

        // Wait for all producers to finish
        keeper.halt();

        // Read the output files...
        let contents1 = read_entire_file(&output_path_1.path).unwrap();
        let contents2 = read_entire_file(&output_path_2.path).unwrap();

        // Assert
        assert!(contents1.len() > 0);
        assert!(contents2.len() > 0);
    }
}
