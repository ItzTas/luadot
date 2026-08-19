mod support;

use std::hint::black_box;
use std::path::Path;

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use luadot::files::{self, ConflictPolicy, LinkMode};
use support::{FILE_COUNTS, Fixture};
use tempfile::TempDir;

fn walk(c: &mut Criterion) {
    let mut group = c.benchmark_group("walk");

    for count in FILE_COUNTS {
        let fixture = Fixture::with_templates(count);
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("collect_entries", count),
            &fixture,
            |b, fixture| {
                b.iter(|| {
                    files::collect_entries("status", fixture.repo()).expect("a walked repository")
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("collect_files", count),
            &fixture,
            |b, fixture| {
                b.iter(|| files::collect_files("rm", fixture.repo()).expect("a walked repository"));
            },
        );
    }

    group.finish();
}

fn status(c: &mut Criterion) {
    let mut group = c.benchmark_group("status");

    for count in FILE_COUNTS {
        let fixture = Fixture::new(count);
        fixture.spread(LinkMode::Hard);
        let pairs = fixture.pairs();
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("file_status", count),
            &pairs,
            |b, pairs| {
                b.iter(|| {
                    for (source, dest) in pairs {
                        black_box(
                            files::file_status(LinkMode::Hard, source, dest).expect("a status"),
                        );
                    }
                });
            },
        );
    }

    group.finish();
}

fn sync(c: &mut Criterion) {
    let mut group = c.benchmark_group("sync");

    for count in FILE_COUNTS {
        let fixture = Fixture::new(count);
        group.throughput(Throughput::Elements(count as u64));

        for mode in [LinkMode::Hard, LinkMode::Symbolic] {
            group.bench_with_input(
                BenchmarkId::new(mode.name(), count),
                &fixture,
                |b, fixture| {
                    b.iter_batched(
                        || tempfile::tempdir().expect("a temporary destination"),
                        |dest| {
                            place_all(fixture, dest.path(), mode);
                            dest
                        },
                        BatchSize::PerIteration,
                    );
                },
            );
        }
    }

    group.finish();
}

fn place_all(fixture: &Fixture, dest: &Path, mode: LinkMode) {
    for file in fixture.files() {
        let target = dest.join(fixture.relative(file));
        files::sync_file(ConflictPolicy::Overwrite, mode, file, &target).expect("a synced file");
    }
}

fn overwrite(c: &mut Criterion) {
    let mut group = c.benchmark_group("overwrite");

    for count in FILE_COUNTS {
        let fixture = Fixture::new(count);
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("sync_file", count),
            &fixture,
            |b, fixture| {
                b.iter_batched(
                    || diverged(fixture),
                    |dest| {
                        place_all(fixture, dest.path(), LinkMode::Hard);
                        dest
                    },
                    BatchSize::PerIteration,
                );
            },
        );
    }

    group.finish();
}

fn diverged(fixture: &Fixture) -> TempDir {
    let dest = tempfile::tempdir().expect("a temporary destination");
    for file in fixture.files() {
        support::write(&dest.path().join(fixture.relative(file)), "diverged\n");
    }

    dest
}

criterion_group!(benches, walk, status, sync, overwrite);
criterion_main!(benches);
