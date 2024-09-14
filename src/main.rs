mod argument_parser;
mod csvs_processor;
mod top_calculator;
mod weapon_stats;
mod writer;
use argument_parser::ArgumentParser;
use csvs_processor::Processor;
use rayon::ThreadPoolBuilder;
use std::time::Instant;

fn build_thread_pool(num_threads: usize) -> rayon::ThreadPool {
    ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .expect("Failed to build thread pool")
}
fn main() -> std::io::Result<()> {
    let parser = ArgumentParser::new();
    let pool = build_thread_pool(parser.get_num_threads());
    let processor = Processor::new(Instant::now());
    pool.install(|| {
        processor.process_and_write_results(&parser);
    });
    Ok(())
}
