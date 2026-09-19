use criterion::{Criterion, criterion_group, criterion_main};
use readable_uuid::{ReadableUuid, Uuid};
use std::hint::black_box;

fn bench(c: &mut Criterion) {
    let text = "550e8400-e29b-41d4-a716-446655440000";
    let id = Uuid::parse_str(text).unwrap();
    let formatter = ReadableUuid::default();
    c.bench_function("core/uuid", |b| b.iter(|| formatter.format(black_box(&id))));
    c.bench_function("core/string", |b| {
        b.iter(|| formatter.format_str(black_box(text)).unwrap())
    });
    let mut output = String::with_capacity(128);
    c.bench_function("core/reuse", |b| {
        b.iter(|| {
            output.clear();
            formatter.write_into(black_box(&id), &mut output);
            black_box(&output);
        })
    });
    c.bench_function("humanhash/bytes", |b| {
        b.iter(|| humanhash::humanize_bytes(black_box(id.as_bytes())).unwrap())
    });
    c.bench_function("humanhash/string", |b| {
        b.iter(|| humanhash::humanize(black_box(text)).unwrap())
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
