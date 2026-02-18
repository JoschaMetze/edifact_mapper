//! Benchmarks for parser throughput and batch conversion.
//!
//! Run with: `cargo bench -p automapper-core`

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};

use automapper_core::{
    convert_batch, convert_sequential, Coordinator, FormatVersion, UtilmdCoordinator, FV2504,
};

/// A synthetic UTILMD message for benchmarking.
fn synthetic_utilmd() -> Vec<u8> {
    b"UNA:+.? 'UNB+UNOC:3+9900123000002:500+9900456000001:500+251217:1229+REF001'UNH+MSG001+UTILMD:D:11A:UN:S2.1'BGM+E03+DOC001'DTM+137:202506190130:303'NAD+MS+9900123000002::293'NAD+MR+9900456000001::293'IDE+24+TXID001'STS+E01+E01::Z44'DTM+137:202507010000:303'DTM+471:202508010000:303'RFF+Z13:VORGANGS001'RFF+Z49:1'DTM+Z25:202507010000:303'DTM+Z26:202512310000:303'LOC+Z16+DE00014545768S0000000000000003054'LOC+Z17+DE00098765432100000000000000012'LOC+Z18+NELO00000000001'NAD+Z04+9900999000003::293'UNT+18+MSG001'UNZ+1+REF001'".to_vec()
}

fn bench_single_parse(c: &mut Criterion) {
    let input = synthetic_utilmd();

    let mut group = c.benchmark_group("single_parse");
    group.throughput(Throughput::Bytes(input.len() as u64));

    group.bench_function("utilmd_parse", |b| {
        b.iter(|| {
            let mut coord = UtilmdCoordinator::<FV2504>::new();
            let result = coord.parse(black_box(&input)).unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_batch_conversion(c: &mut Criterion) {
    let msg = synthetic_utilmd();

    for batch_size in [10, 100, 1000] {
        let inputs: Vec<&[u8]> = vec![msg.as_slice(); batch_size];
        let total_bytes = msg.len() * batch_size;

        let mut group = c.benchmark_group(format!("batch_{batch_size}"));
        group.throughput(Throughput::Bytes(total_bytes as u64));

        group.bench_function("parallel", |b| {
            b.iter(|| {
                let results = convert_batch(black_box(&inputs), FormatVersion::FV2504);
                black_box(results);
            });
        });

        group.bench_function("sequential", |b| {
            b.iter(|| {
                let results = convert_sequential(black_box(&inputs), FormatVersion::FV2504);
                black_box(results);
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_single_parse, bench_batch_conversion);
criterion_main!(benches);
