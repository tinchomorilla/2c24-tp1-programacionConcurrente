use rayon::ThreadPoolBuilder;
use std::time::Instant;

use crate::{argument_parser::ArgumentParser, csvs_processor::Processor};
pub struct Runner {}

impl Runner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let parser = ArgumentParser::new();
        let pool = build_thread_pool(parser.get_num_threads());
        let processor = Processor::new(Instant::now());
        pool.install(|| {
            processor.process_and_write_results(&parser);
        });
    }
}

fn build_thread_pool(num_threads: usize) -> rayon::ThreadPool {
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to build thread pool")
}
