mod support;

use std::hint::black_box;
use std::path::{Path, PathBuf};

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use glob::Pattern;
use luadot::files::{ConflictPolicy, LinkMode};
use luadot::lua::{Config, Matcher, Rule, Track};
use luadot::utils;
use support::{PROBE_COUNT, RULE_COUNTS, managed_name};

fn matching(c: &mut Criterion) {
    let mut group = c.benchmark_group("config");
    let probes = probes();
    group.throughput(Throughput::Elements(probes.len() as u64));

    for count in RULE_COUNTS {
        let config = configured(count);

        group.bench_with_input(
            BenchmarkId::new("link_mode", count),
            &config,
            |b, config| {
                b.iter(|| {
                    for probe in &probes {
                        black_box(config.link_mode(probe));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("conflict_policy", count),
            &config,
            |b, config| {
                b.iter(|| {
                    for probe in &probes {
                        black_box(config.conflict_policy(probe));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("is_ignored", count),
            &config,
            |b, config| {
                b.iter(|| {
                    for probe in &probes {
                        black_box(config.is_ignored(probe));
                    }
                });
            },
        );
    }

    group.finish();
}

fn paths(c: &mut Criterion) {
    let mut group = c.benchmark_group("paths");
    let home = Path::new("/home/user");
    let repo = Path::new("/home/user/.local/share/luadot/repo");
    let outside: Vec<PathBuf> = probes().iter().map(|probe| home.join(probe)).collect();
    let inside: Vec<PathBuf> = probes().iter().map(|probe| repo.join(probe)).collect();
    group.throughput(Throughput::Elements(outside.len() as u64));

    group.bench_function("repo_path", |b| {
        b.iter(|| {
            for path in &outside {
                black_box(utils::repo_path(home, repo, path).expect("a repository path"));
            }
        });
    });

    group.bench_function("system_path", |b| {
        b.iter(|| {
            for path in &inside {
                black_box(utils::system_path(home, repo, path).expect("a system path"));
            }
        });
    });

    group.bench_function("relative", |b| {
        b.iter(|| {
            for path in &inside {
                black_box(utils::relative(repo, path));
            }
        });
    });

    group.finish();
}

fn configured(count: usize) -> Config {
    let mut config = Config::default();
    config.set_link(LinkMode::Hard);
    config.set_conflict(ConflictPolicy::Overwrite);
    config.add_rules(ignored(count));
    config.add_rules(rules(count));

    config
}

fn ignored(count: usize) -> Vec<Rule> {
    (0..count)
        .map(|index| pattern(&format!(".cache/app{index:03}/**")))
        .chain([pattern("*.swp"), pattern(".local/state/**")])
        .map(|pattern| Rule::new(pattern, None, None).with_track(Some(Track::Never)))
        .collect()
}

fn rules(count: usize) -> Vec<Rule> {
    (0..count)
        .map(|index| {
            Rule::new(
                pattern(&format!(".config/app{index:03}/**")),
                (index % 2 == 0).then_some(LinkMode::Symbolic),
                (index % 3 == 0).then_some(ConflictPolicy::Skip),
            )
        })
        .collect()
}

fn pattern(raw: &str) -> Matcher {
    Matcher::Glob(Pattern::new(raw).expect("a valid pattern"))
}

fn probes() -> Vec<PathBuf> {
    (0..PROBE_COUNT).map(managed_name).collect()
}

criterion_group!(benches, matching, paths);
criterion_main!(benches);
