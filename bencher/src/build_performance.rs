use std::{
    fs::File,
    io::Write,
    path::Path,
    process::{Command, Stdio},
    vec::Vec,
};

use regex::Regex;

use crate::roc_configuration::RocConfiguration;

pub(crate) fn inserts(benchmarks_path: &Path, roc_path: &Path, output_path: &Path) {
    let static_inc_regex = Regex::new(r"inc `.*`;").unwrap();
    let static_dec_regex = Regex::new(r"dec `.*`;").unwrap();
    let static_reset_regex = Regex::new(r"Reset \{.*\};").unwrap();
    let static_reuse_regex = Regex::new(r"Reuse `.*`;").unwrap();

    let dynamic_inc_regex = Regex::new(r"\| increment").unwrap();
    let dynamic_dec_regex = Regex::new(r"\| decrement").unwrap();
    let dynamic_alloc_regex = Regex::new(r"alloc:").unwrap();
    let dynamic_dealloc_regex = Regex::new(r"free:").unwrap();

    let mut results = [[Result::default(); BENCHMARKS.len()]; CONFIGURATIONS.len()];

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        println!("Configuration:\n{}", configuration.name);

        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            let output = Command::new("cargo")
                .current_dir(roc_path)
                .env("RUSTFLAGS", configuration.configuration.flags())
                .env("ROC_PRINT_IR_AFTER_RC", "1")
                .args([
                    "run",
                    "--",
                    "--debug",
                    "--linker=legacy",
                    benchmarks_path.join(benchmark.path).to_str().unwrap(),
                ]) // todo update
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .output()
                .expect("failed to execute process");

            let stdout = String::from_utf8(output.stdout).expect("invalid utf8");
            let stderr = String::from_utf8(output.stderr).expect("invalid utf8");

            File::create(Path::new(output_path).join(format!(
                "{}-{}.txt",
                configuration.name.replace("/", ""),
                benchmark.name.replace("/", "")
            )))
            .unwrap()
            .write_all(stderr.as_bytes())
            .unwrap();

            let static_incs = static_inc_regex.find_iter(&stderr).count();
            let static_decs = static_dec_regex.find_iter(&stderr).count();
            let static_resets = static_reset_regex.find_iter(&stderr).count();
            let static_reuses = static_reuse_regex.find_iter(&stderr).count();

            let dynamic_incs = dynamic_inc_regex.find_iter(&stderr).count();
            let dynamic_decs = dynamic_dec_regex.find_iter(&stderr).count();
            let dynamic_allocs = dynamic_alloc_regex.find_iter(&stdout).count();
            let dynamic_deallocs = dynamic_dealloc_regex.find_iter(&stdout).count();

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

    println!("Results:");
    println!(
        "Static\t{}",
        BENCHMARKS
            .iter()
            .map(|b| { format!("{}\t\t\t\t", b.name) })
            .collect::<Vec<String>>()
            .join("")
    );
    println!(
        "\t{}",
        BENCHMARKS
            .iter()
            .map(|_| { "inc\tdec\treset\treuse\t" })
            .collect::<Vec<_>>()
            .join("")
    );
    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        print!("{}\t", configuration.name);
        for (bi, _) in BENCHMARKS.iter().enumerate() {
            let result = &results[ci][bi];

            print!(
                "{}\t{}\t{}\t{}\t",
                result.static_incs, result.static_decs, result.static_resets, result.static_reuses
            );
        }
        println!("");
    }

    println!("");
    println!(
        "Dynamic\t{}",
        BENCHMARKS
            .iter()
            .map(|b| { format!("{}\t\t\t\t", b.name) })
            .collect::<Vec<String>>()
            .join("")
    );
    println!(
        "\t{}",
        BENCHMARKS
            .iter()
            .map(|_| { "inc\tdec\talloc\tdealloc\t" })
            .collect::<Vec<_>>()
            .join("")
    );
    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        print!("{}\t", configuration.name);
        for (bi, _) in BENCHMARKS.iter().enumerate() {
            let result = &results[ci][bi];

            print!(
                "{}\t{}\t{}\t{}\t",
                result.dynamic_incs,
                result.dynamic_decs,
                result.dynamic_allocs,
                result.dynamic_deallocs
            );
        }
        println!("");
    }
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
    dynamic_incs: usize,
    dynamic_decs: usize,
    dynamic_allocs: usize,
    dynamic_deallocs: usize,
}
