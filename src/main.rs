mod argument_parser;
mod csvs_processor;
mod runner;
mod top_calculator;
mod weapon_stats;
mod writer;
use runner::Runner;

fn main() -> std::io::Result<()> {
    let runner = Runner::new();
    runner.run();
    Ok(())
}
