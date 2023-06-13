use std::env;

mod build_performance;
mod roc_configuration;
mod runtime_performance;

fn main() {
    let current_dir = env::current_dir().unwrap();

    let args: Vec<String> = env::args().collect();
    match args.as_slice() {
        [_] => {
            build_performance::inserts(
                current_dir.join("benchmarks/").as_path(),
                current_dir.parent().unwrap(),
                current_dir.join("results/ir/").as_path(),
            );
        }
        [_, iterations] => {
            runtime_performance::performance(
                current_dir.join("benchmarks/").as_path(),
                current_dir.join("binaries/").as_path(),
                iterations.parse::<u32>().unwrap(),
                current_dir.parent().unwrap(),
                current_dir.join("results/time-benchmark.csv").as_path(),
                current_dir.join("results/memory-benchmark.csv").as_path(),
            );
        }
        _ => todo!(),
    }
}
