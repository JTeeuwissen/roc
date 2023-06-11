use std::{
    fs,
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

use regex::Regex;

use crate::roc_configuration::RocConfiguration;

pub(crate) fn performance(
    benchmarks_path: &Path,
    binaries_path: &Path,
    iterations: u32,
    roc_path: &Path,
    csv_path: &Path,
) {
    let executable_paths = create_paths(binaries_path);

    let _ = fs::create_dir(binaries_path);

    build_benchmarks(benchmarks_path, &executable_paths, roc_path);

    let args = [
        "--warmup",
        "3",
        "--runs",
        &iterations.to_string(),
        "--export-csv",
        csv_path.to_str().unwrap(),
        "--ignore-failure",
    ];
    let commands = BENCHMARKS
        .into_iter()
        .enumerate()
        .flat_map(|(bi, benchmark)| {
            CONFIGURATIONS
                .into_iter()
                .enumerate()
                .flat_map(|(ci, configuration)| {
                    let executable_path = executable_paths[ci][bi.clone()].clone();
                    [
                        "--command-name".to_string(),
                        format!(
                            "{} {}",
                            configuration.name.replace("/", ""),
                            benchmark.name.replace("/", ""),
                        ),
                        format!("ulimit -s unlimited && \"{}\"", executable_path),
                    ]
                })
                .collect::<std::vec::Vec<_>>()
        })
        .collect::<std::vec::Vec<_>>();

    Command::new("hyperfine")
        .args(args.into_iter().chain(commands.iter().map(|s| s.as_str())))
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}

fn create_paths(binaries_path: &Path) -> [[String; BENCHMARKS.len()]; CONFIGURATIONS.len()] {
    let mut executable_paths: [[String; BENCHMARKS.len()]; CONFIGURATIONS.len()] =
        Default::default();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            executable_paths[ci][bi] = binaries_path
                .join(format!(
                    "{} {}",
                    configuration.name.replace("/", ""),
                    benchmark.name.replace("/", ""),
                ))
                .as_path()
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    executable_paths
}

fn build_benchmarks<'a>(
    benchmarks_path: &Path,
    executable_paths: &[[String; BENCHMARKS.len()]; CONFIGURATIONS.len()],
    roc_path: &Path,
) {
    let build_output_regex = Regex::new(r"\n\n    (.*)").unwrap();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            let path = executable_paths[ci][bi].clone();
            match configuration.variant {
                ConfigurationVariant::Roc(roc_configuration) => {
                    let output = Command::new("cargo")
                        .current_dir(Path::new(roc_path))
                        .env("RUSTFLAGS", roc_configuration.flags())
                        .args([
                            "run",
                            "--release",
                            "--",
                            "build",
                            "--optimize",
                            benchmarks_path.join(benchmark.roc_path).to_str().unwrap(),
                        ])
                        .stderr(Stdio::piped())
                        .stdout(Stdio::piped())
                        .output()
                        .expect("failed to execute process");
                    let stdout = std::str::from_utf8(&output.stdout).expect("invalid utf8");
                    let relative_output_path = build_output_regex
                        .captures(&stdout)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str()
                        .to_string();

                    let full_output_path = benchmarks_path
                        .parent()
                        .unwrap()
                        .parent()
                        .unwrap()
                        .join(relative_output_path);

                    fs::copy(full_output_path.clone(), path).unwrap();
                }
                ConfigurationVariant::Koka => {
                    Command::new("koka")
                        .args([
                            benchmarks_path.join(benchmark.koka_path).to_str().unwrap(),
                            "-O2",
                            "--output",
                            path.as_str(),
                        ])
                        .stderr(Stdio::piped())
                        .stdout(Stdio::piped())
                        .output()
                        .expect("failed to execute process");

                    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
                }
            };
        }
    }
}

const CONFIGURATIONS: [Configuration; 5] = [
    Configuration {
        name: "Beans",
        variant: ConfigurationVariant::Roc(RocConfiguration::Beans),
    },
    Configuration {
        name: "Beans w/ specialize",
        variant: ConfigurationVariant::Roc(RocConfiguration::BeansSpecialisation),
    },
    Configuration {
        name: "Perceus",
        variant: ConfigurationVariant::Roc(RocConfiguration::Perceus),
    },
    Configuration {
        name: "Perceus w/ specialize",
        variant: ConfigurationVariant::Roc(RocConfiguration::PerceusSpecialisation),
    },
    Configuration {
        name: "Koka",
        variant: ConfigurationVariant::Koka,
    },
];

struct Configuration<'a> {
    name: &'a str,
    variant: ConfigurationVariant,
}

enum ConfigurationVariant {
    Roc(RocConfiguration),
    Koka,
}

const BENCHMARKS: [Benchmark; 5] = [
    Benchmark {
        name: "Deriv",
        roc_path: "roc/Deriv.roc",
        koka_path: "koka/deriv.kk",
    },
    Benchmark {
        name: "NQueens",
        roc_path: "roc/NQueens.roc",
        koka_path: "koka/nqueens.kk",
    },
    Benchmark {
        name: "CFold",
        roc_path: "roc/CFold.roc",
        koka_path: "koka/cfold.kk",
    },
    Benchmark {
        name: "RBTree",
        roc_path: "roc/RBTree.roc",
        koka_path: "koka/rbtree.kk",
    },
    Benchmark {
        name: "RBTreeCk",
        roc_path: "roc/RBTreeCk.roc",
        koka_path: "koka/rbtree-ck.kk",
    },
];

struct Benchmark<'a> {
    name: &'a str,
    roc_path: &'a str,
    koka_path: &'a str,
}
