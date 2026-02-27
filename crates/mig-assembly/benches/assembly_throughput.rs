//! Benchmarks for MIG-guided assembly throughput.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use mig_assembly::assembler::Assembler;
use mig_assembly::tokenize::parse_to_segments;
use rayon::prelude::*;
use std::path::Path;

fn bench_tokenization(c: &mut Criterion) {
    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping benchmarks");
        return;
    }

    let fixtures: Vec<(String, Vec<u8>)> = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "edi").unwrap_or(false))
        .take(5)
        .map(|e| {
            let name = e.path().file_stem().unwrap().to_string_lossy().to_string();
            let content = std::fs::read(e.path()).unwrap();
            (name, content)
        })
        .collect();

    if fixtures.is_empty() {
        eprintln!("No .edi fixtures found, skipping");
        return;
    }

    let mut group = c.benchmark_group("tokenization");
    for (name, content) in &fixtures {
        group.bench_with_input(BenchmarkId::from_parameter(name), content, |b, content| {
            b.iter(|| parse_to_segments(content).unwrap());
        });
    }
    group.finish();
}

fn bench_assembly(c: &mut Criterion) {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping assembly benchmarks");
        return;
    }

    let mig =
        mig_assembly::parsing::parse_mig(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let fixtures: Vec<(String, Vec<mig_assembly::tokenize::OwnedSegment>)> =
        std::fs::read_dir(fixture_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "edi").unwrap_or(false))
            .take(5)
            .map(|e| {
                let name = e.path().file_stem().unwrap().to_string_lossy().to_string();
                let content = std::fs::read(e.path()).unwrap();
                let segments = parse_to_segments(&content).unwrap();
                (name, segments)
            })
            .collect();

    if fixtures.is_empty() {
        eprintln!("No .edi fixtures found, skipping");
        return;
    }

    let mut group = c.benchmark_group("assembly");
    for (name, segments) in &fixtures {
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            segments,
            |b, segments| {
                let assembler = Assembler::new(&mig);
                b.iter(|| assembler.assemble_generic(segments).unwrap());
            },
        );
    }
    group.finish();
}

fn bench_full_pipeline(c: &mut Criterion) {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping full pipeline benchmarks");
        return;
    }

    let service =
        mig_assembly::ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    let fixtures: Vec<(String, String)> = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "edi").unwrap_or(false))
        .take(5)
        .map(|e| {
            let name = e.path().file_stem().unwrap().to_string_lossy().to_string();
            let content = std::fs::read_to_string(e.path()).unwrap();
            (name, content)
        })
        .collect();

    if fixtures.is_empty() {
        eprintln!("No .edi fixtures found, skipping");
        return;
    }

    let mut group = c.benchmark_group("full_pipeline");
    for (name, content) in &fixtures {
        group.bench_with_input(BenchmarkId::from_parameter(name), content, |b, content| {
            b.iter(|| service.convert_to_tree(content).unwrap());
        });
    }
    group.finish();
}

fn bench_batch_pipeline(c: &mut Criterion) {
    let mig_path = Path::new(
        "../../xml-migs-and-ahbs/FV2504/UTILMD_MIG_Strom_S2_1_Fehlerkorrektur_20250320.xml",
    );
    if !mig_path.exists() {
        eprintln!("MIG file not found, skipping batch pipeline benchmarks");
        return;
    }

    let service =
        mig_assembly::ConversionService::new(mig_path, "UTILMD", Some("Strom"), "FV2504").unwrap();

    let fixture_dir =
        Path::new("../../example_market_communication_bo4e_transactions/UTILMD/FV2504");
    if !fixture_dir.exists() {
        eprintln!("Fixture dir not found, skipping");
        return;
    }

    // Load the first available .edi fixture as our representative message
    let fixture_content = std::fs::read_dir(fixture_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|x| x == "edi").unwrap_or(false))
        .map(|e| std::fs::read_to_string(e.path()).unwrap());

    let content = match fixture_content {
        Some(c) => c,
        None => {
            eprintln!("No .edi fixture found, skipping batch benchmarks");
            return;
        }
    };

    let batch_sizes: &[usize] = &[10, 100, 1000];

    // Sequential batch processing
    let mut group = c.benchmark_group("batch_pipeline_sequential");
    for &n in batch_sizes {
        let batch: Vec<&str> = vec![content.as_str(); n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &batch, |b, batch| {
            b.iter(|| {
                batch
                    .iter()
                    .map(|msg| service.convert_to_tree(msg).unwrap())
                    .collect::<Vec<_>>()
            });
        });
    }
    group.finish();

    // Parallel batch processing with rayon
    let mut group = c.benchmark_group("batch_pipeline_parallel");
    for &n in batch_sizes {
        let batch: Vec<&str> = vec![content.as_str(); n];
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &batch, |b, batch| {
            b.iter(|| {
                batch
                    .par_iter()
                    .map(|msg| service.convert_to_tree(msg).unwrap())
                    .collect::<Vec<_>>()
            });
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_tokenization,
    bench_assembly,
    bench_full_pipeline,
    bench_batch_pipeline
);
criterion_main!(benches);
