use rayon::ThreadPoolBuilder;
use std::time::Instant;

use crate::{argument_parser::ArgumentParser, csvs_processor::Processor};
pub struct Runner {}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    /// Runs the application.
    pub fn run(&self) {
        let parser = ArgumentParser::new();
        let pool = build_thread_pool(parser.get_num_threads());
        let processor = Processor::new(Instant::now());
        pool.install(|| {
            processor.process_and_write_results(&parser);
        });
    }
}

/// Builds a thread pool with the given number of threads.
/// It can be seen that as we increase the number of threads 
/// there is an improvement in the processing time, up to a certain thread limit. 
/// After a certain number of threads the program gets worse due to 
/// the fight for the CPU.
/// 
/// # Arguments
///
/// * `num_threads` - The number of threads to use in the thread pool.
///
/// # Returns
///
/// A thread pool with the given number of threads.
/// If the thread pool cannot be built, the function panics.
fn build_thread_pool(num_threads: usize) -> rayon::ThreadPool {
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to build thread pool")
}


