use std::{
    env, fs, iter,
    os::unix::prelude::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};

use csv::Writer;
use regex::Regex;

use crate::roc_configuration::RocConfiguration;

pub(crate) fn performance(
    benchmarks_path: &Path,
    roc_binaries_path: &Path,
    benchmark_binaries_path: &Path,
    iterations: u32,
    roc_path: &Path,
    time_csv_path: &Path,
    memory_csv_path: &Path,
) {
    let _ = fs::create_dir(roc_binaries_path);

    let compiler_paths = create_compiler_paths(roc_path, roc_binaries_path);

    let _ = fs::create_dir(benchmark_binaries_path);

    let benchmark_paths = create_benchmark_paths(benchmark_binaries_path);

    build_benchmarks(compiler_paths, benchmarks_path, &benchmark_paths);

    run_time_benchmarks(iterations, time_csv_path, &benchmark_paths);

    run_memory_benchmarks(memory_csv_path, &benchmark_paths);
}

fn create_compiler_paths(
    roc_path: &Path,
    roc_binaries_path: &Path,
) -> [String; CONFIGURATIONS.len()] {
    let mut compiler_paths: [String; CONFIGURATIONS.len()] = Default::default();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        compiler_paths[ci] = match configuration.variant {
            ConfigurationVariant::Roc(roc_configuration) => {
                let output_path_buf = roc_binaries_path.join(configuration.name);
                // Command::new("cargo")
                //     .current_dir(Path::new(roc_path))
                //     .env("RUSTFLAGS", roc_configuration.flags())
                //     .args([
                //         "build",
                //         "--bin=roc",
                //         "--release",
                //         "--target-dir",
                //         output_path_buf.to_str().unwrap(),
                //     ])
                //     .stderr(Stdio::inherit())
                //     .stdout(Stdio::inherit())
                //     .output()
                //     .unwrap();
                output_path_buf
                    .join("release/roc")
                    .to_str()
                    .unwrap()
                    .to_string()
            }
            ConfigurationVariant::Koka => "koka".to_string(),
            ConfigurationVariant::Haskell => "ghc".to_string(),
        }
    }

    compiler_paths
}

fn run_memory_benchmarks(
    memory_csv_path: &Path,
    executable_paths: &[[String; BENCHMARKS.len()]; CONFIGURATIONS.len()],
) {
    let memory_usage_regex = Regex::new(r"Maximum resident set size \(kbytes\): (.*)").unwrap();
    let mut wtr = Writer::from_path(memory_csv_path).unwrap();
    wtr.write_record(iter::once("").chain(BENCHMARKS.iter().map(|benchmark| benchmark.name)))
        .unwrap();
    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        let mut benchmark_results: [String; BENCHMARKS.len()] = Default::default();

        for (bi, _) in BENCHMARKS.iter().enumerate() {
            let executable_path = executable_paths[ci][bi].clone();
            let output = Command::new("/usr/bin/time")
                .args(["-v", executable_path.as_str()])
                .stderr(Stdio::piped())
                .output()
                .expect("failed to execute process");
            let stderr = std::str::from_utf8(&output.stderr).expect("invalid utf8");
            let memory_usage = memory_usage_regex
                .captures(stderr)
                .unwrap()
                .get(1)
                .unwrap()
                .as_str()
                .parse::<i32>()
                .unwrap();
            benchmark_results[bi] = (memory_usage / 1024).to_string();
        }

        wtr.write_record(
            iter::once(configuration.name).chain(benchmark_results.iter().map(|str| str.as_str())),
        )
        .unwrap();
    }

    wtr.flush().unwrap();
}

fn run_time_benchmarks(
    iterations: u32,
    csv_path: &Path,
    executable_paths: &[[String; BENCHMARKS.len()]; CONFIGURATIONS.len()],
) {
    let commands = BENCHMARKS
        .into_iter()
        .enumerate()
        .flat_map(|(bi, benchmark)| {
            CONFIGURATIONS
                .into_iter()
                .enumerate()
                .flat_map(|(ci, configuration)| {
                    let executable_path = executable_paths[ci][bi].clone();
                    [
                        "--command-name".to_string(),
                        format!(
                            "{} {}",
                            configuration.name.replace('/', ""),
                            benchmark.name.replace('/', ""),
                        ),
                        format!("ulimit -s unlimited && \"{}\"", executable_path),
                    ]
                })
                .collect::<std::vec::Vec<_>>()
        })
        .collect::<std::vec::Vec<_>>();

    Command::new("hyperfine")
        .args(
            [
                "--warmup",
                "3",
                "--runs",
                &iterations.to_string(),
                "--export-csv",
                csv_path.to_str().unwrap(),
                "--ignore-failure",
            ]
            .into_iter()
            .chain(commands.iter().map(|s| s.as_str())),
        )
        .stdin(Stdio::inherit())
        .stderr(Stdio::inherit())
        .stdout(Stdio::inherit())
        .output()
        .expect("failed to execute process");
}

fn create_benchmark_paths(
    binaries_path: &Path,
) -> [[String; BENCHMARKS.len()]; CONFIGURATIONS.len()] {
    let mut executable_paths: [[String; BENCHMARKS.len()]; CONFIGURATIONS.len()] =
        Default::default();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            executable_paths[ci][bi] = binaries_path
                .join(format!(
                    "{} {}",
                    configuration.name.replace('/', ""),
                    benchmark.name.replace('/', ""),
                ))
                .as_path()
                .to_str()
                .unwrap()
                .to_string();
        }
    }

    executable_paths
}

fn build_benchmarks(
    compiler_paths: [String; CONFIGURATIONS.len()],
    benchmarks_path: &Path,
    executable_paths: &[[String; BENCHMARKS.len()]; CONFIGURATIONS.len()],
) {
    let build_output_regex = Regex::new(r"\n\n    (.*)").unwrap();

    for (ci, configuration) in CONFIGURATIONS.iter().enumerate() {
        for (bi, benchmark) in BENCHMARKS.iter().enumerate() {
            let path = executable_paths[ci][bi].clone();
            match configuration.variant {
                ConfigurationVariant::Roc(_) => {
                    let output = Command::new(compiler_paths[ci].clone().as_str())
                        .args([
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
                        .captures(stdout)
                        .unwrap()
                        .get(1)
                        .unwrap()
                        .as_str()
                        .to_string();

                    let full_output_path =
                        benchmarks_path.parent().unwrap().join(relative_output_path);

                    fs::rename(full_output_path.clone(), path).unwrap();
                }
                ConfigurationVariant::Koka => {
                    Command::new(compiler_paths[ci].clone())
                        .args([
                            benchmarks_path.join(benchmark.koka_path).to_str().unwrap(),
                            "-O2",
                            "--output",
                            path.as_str(),
                        ])
                        .stderr(Stdio::inherit())
                        .stdout(Stdio::inherit())
                        .output()
                        .expect("failed to execute process");

                    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
                }
                ConfigurationVariant::Haskell => {
                    Command::new(compiler_paths[ci].clone())
                        .env(
                            "PATH",
                            env::var("PATH")
                                .unwrap()
                                .split(':')
                                .filter(|s| !s.starts_with("/nix/"))
                                .collect::<Vec<_>>()
                                .join(":"),
                        )
                        .args([
                            "--make",
                            "-o",
                            path.as_str(),
                            "-O",
                            benchmarks_path
                                .join(benchmark.haskell_path)
                                .to_str()
                                .unwrap(),
                        ])
                        .stderr(Stdio::piped())
                        .stdout(Stdio::piped())
                        .output()
                        .expect("failed to execute process");
                }
            };
        }
    }
}

const CONFIGURATIONS: [Configuration; 6] = [
    Configuration {
        name: "Beans",
        variant: ConfigurationVariant::Roc(RocConfiguration::Beans),
    },
    Configuration {
        name: "Beans Specialized",
        variant: ConfigurationVariant::Roc(RocConfiguration::BeansSpecialisation),
    },
    Configuration {
        name: "Perceus",
        variant: ConfigurationVariant::Roc(RocConfiguration::Perceus),
    },
    Configuration {
        name: "Perceus Specialized",
        variant: ConfigurationVariant::Roc(RocConfiguration::PerceusSpecialisation),
    },
    Configuration {
        name: "Koka",
        variant: ConfigurationVariant::Koka,
    },
    Configuration {
        name: "Haskell",
        variant: ConfigurationVariant::Haskell,
    },
];

struct Configuration<'a> {
    name: &'a str,
    variant: ConfigurationVariant,
}

enum ConfigurationVariant {
    Roc(RocConfiguration),
    Koka,
    Haskell,
}

const BENCHMARKS: [Benchmark; 5] = [
    Benchmark {
        name: "Deriv",
        roc_path: "roc/Deriv.roc",
        koka_path: "koka/deriv.kk",
        haskell_path: "haskell/deriv.hs",
    },
    Benchmark {
        name: "NQueens",
        roc_path: "roc/NQueens.roc",
        koka_path: "koka/nqueens.kk",
        haskell_path: "haskell/nqueens.hs",
    },
    Benchmark {
        name: "CFold",
        roc_path: "roc/CFold.roc",
        koka_path: "koka/cfold.kk",
        haskell_path: "haskell/cfold.hs",
    },
    Benchmark {
        name: "RBTree",
        roc_path: "roc/RBTree.roc",
        koka_path: "koka/rbtree.kk",
        haskell_path: "haskell/rbtree.hs",
    },
    Benchmark {
        name: "RBTreeCk",
        roc_path: "roc/RBTreeCk.roc",
        koka_path: "koka/rbtree-ck.kk",
        haskell_path: "haskell/rbtree-ck.hs",
    },
];

struct Benchmark<'a> {
    name: &'a str,
    roc_path: &'a str,
    koka_path: &'a str,
    haskell_path: &'a str,
}
