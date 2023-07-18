use std::{
    fs::File,
    io::Write,
    iter,
    path::Path,
    process::{Command, Stdio},
    vec::Vec,
};

use crate::roc_configuration::RocConfiguration;
use csv::Writer;
use regex::Regex;

pub(crate) fn static_performance(
    benchmarks_path: &Path,
    roc_path: &Path,
    output_path: &Path,
    static_csv_path: &Path,
    dynamic_csv_path: &Path,
) {
    let static_inc_regex = Regex::new(r"inc `.*`;").unwrap();
    let static_dec_regex = Regex::new(r"dec(ref)? `.*`;").unwrap();
    let static_reset_regex = Regex::new(r"Reset(Ref)? \{.*\};").unwrap();
    let static_reuse_regex = Regex::new(r"Reuse `.*`;").unwrap();

    let dynamic_inc_regex = Regex::new(r"inc: (\d*)").unwrap();
    let dynamic_dec_regex = Regex::new(r"dec: (\d*)").unwrap();
    let dynamic_alloc_regex = Regex::new(r"alloc: (\d*)").unwrap();
    let dynamic_dealloc_regex = Regex::new(r"free: (\d*)").unwrap();

    let mut results = [[Result::default(); BENCHMARKS.len()]; CONFIGURATIONS.len()];

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        println!("Configuration:\n{}", configuration.name);

        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            let output = Command::new("sh")
                .current_dir(roc_path)
                .env("RUSTFLAGS", configuration.configuration.flags())
                .env("ROC_PRINT_IR_AFTER_RC", "1")
                .args([
                    "-c",
                    &format!(
                        "ulimit -s unlimited && cargo run -- run --optimize --debug --linker=legacy {}",
                        benchmarks_path.join(benchmark.path).to_str().unwrap()
                    ),
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::inherit())
                .stderr(Stdio::piped())
                .output()
                .expect("failed to execute process");

            let stderr = String::from_utf8(output.stderr).expect("invalid utf8");

            File::create(Path::new(output_path).join(format!(
                "{}-{}.txt",
                configuration.name.replace('/', ""),
                benchmark.name.replace('/', "")
            )))
            .unwrap()
            .write_all(stderr.as_bytes())
            .unwrap();

            let static_incs = static_inc_regex.find_iter(&stderr).count();
            let static_decs = static_dec_regex.find_iter(&stderr).count();
            let static_resets = static_reset_regex.find_iter(&stderr).count();
            let static_reuses = static_reuse_regex.find_iter(&stderr).count();

            let dynamic_incs = dynamic_inc_regex
                .captures(&stderr)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<u64>()
                .unwrap();
            let dynamic_decs = dynamic_dec_regex
                .captures(&stderr)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<u64>()
                .unwrap();
            let dynamic_allocs = dynamic_alloc_regex
                .captures(&stderr)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<u64>()
                .unwrap();
            let dynamic_deallocs = dynamic_dealloc_regex
                .captures(&stderr)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<u64>()
                .unwrap();

            results[ci][bi] = Result {
                static_incs,
                static_decs,
                static_resets,
                static_reuses,
                dynamic_incs,
                dynamic_decs,
                dynamic_allocs,
                dynamic_deallocs,
            };

            println!(
                  "Benchmark: {}\nStatic incs: {} decs: {} resets: {} reuses: {}\nDynamic incs: {} decs: {} allocs: {} deallocs: {}",
                  benchmark.name, static_incs, static_decs, static_resets, static_reuses, dynamic_incs, dynamic_decs, dynamic_allocs , dynamic_deallocs
              );
        }
    }

    let mut wtr = Writer::from_path(static_csv_path).unwrap();
    wtr.write_record(
        iter::once("Static").chain(BENCHMARKS.iter().flat_map(|b| [b.name, "", "", ""])),
    )
    .unwrap();

    wtr.write_record(
        iter::once("").chain(
            BENCHMARKS
                .iter()
                .flat_map(|_| ["dup", "drop", "reset", "reuse"]),
        ),
    )
    .unwrap();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        let iter = BENCHMARKS
            .iter()
            .enumerate()
            .flat_map(|(bi, _)| {
                let result = &results[ci][bi];
                [
                    result.static_incs.to_string(),
                    result.static_decs.to_string(),
                    result.static_resets.to_string(),
                    result.static_reuses.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        wtr.write_record(iter::once(configuration.name).chain(iter.iter().map(|v| v.as_str())))
            .unwrap();
    }

    wtr.flush().unwrap();

    let mut wtr = Writer::from_path(dynamic_csv_path).unwrap();
    wtr.write_record(
        iter::once("Dynamic").chain(BENCHMARKS.iter().flat_map(|b| [b.name, "", "", ""])),
    )
    .unwrap();

    wtr.write_record(
        iter::once("").chain(
            BENCHMARKS
                .iter()
                .flat_map(|_| ["dup", "drop", "alloc", "dealloc"]),
        ),
    )
    .unwrap();
    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        let iter = BENCHMARKS
            .iter()
            .enumerate()
            .flat_map(|(bi, _)| {
                let result = &results[ci][bi];
                [
                    result.dynamic_incs.to_string(),
                    result.dynamic_decs.to_string(),
                    result.dynamic_allocs.to_string(),
                    result.dynamic_deallocs.to_string(),
                ]
            })
            .collect::<Vec<_>>();
        wtr.write_record(iter::once(configuration.name).chain(iter.iter().map(|v| v.as_str())))
            .unwrap();
    }
    wtr.flush().unwrap();
}

const CONFIGURATIONS: [Configuration; 4] = [
    Configuration {
        name: "Beans",
        configuration: RocConfiguration::Beans,
    },
    Configuration {
        name: "Beans Specialized",
        configuration: RocConfiguration::BeansSpecialisation,
    },
    Configuration {
        name: "Perceus",
        configuration: RocConfiguration::Perceus,
    },
    Configuration {
        name: "Perceus Specialized",
        configuration: RocConfiguration::PerceusSpecialisation,
    },
];

struct Configuration<'a> {
    name: &'a str,
    configuration: RocConfiguration,
}

const BENCHMARKS: [Benchmark; 5] = [
    Benchmark {
        name: "Deriv",
        path: "roc/Deriv.roc",
    },
    Benchmark {
        name: "NQueens",
        path: "roc/NQueens.roc",
    },
    Benchmark {
        name: "CFold",
        path: "roc/CFold.roc",
    },
    Benchmark {
        name: "RBTree",
        path: "roc/RBTree.roc",
    },
    Benchmark {
        name: "RBTreeCk",
        path: "roc/RBTreeCk.roc",
    },
];

struct Benchmark<'a> {
    name: &'a str,
    path: &'a str,
}

#[derive(Default, Clone, Copy)]
struct Result {
    static_incs: usize,
    static_decs: usize,
    static_resets: usize,
    static_reuses: usize,
    dynamic_incs: u64,
    dynamic_decs: u64,
    dynamic_allocs: u64,
    dynamic_deallocs: u64,
}
