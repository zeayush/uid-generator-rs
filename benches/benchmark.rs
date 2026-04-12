use criterion::{Criterion, criterion_group, criterion_main};
use uid_generator_rs::{DEFAULT_EPOCH_MS, Snowflake, Ulid};

fn bench_snowflake_single_thread(c: &mut Criterion) {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).expect("failed to create snowflake");
    c.bench_function("snowflake/next_id", |b| {
        b.iter(|| sf.next_id().expect("next_id failed"));
    });
}

fn bench_ulid_new(c: &mut Criterion) {
    c.bench_function("ulid/new", |b| {
        b.iter(|| Ulid::new().expect("ulid generation failed"));
    });
}

criterion_group!(benches, bench_snowflake_single_thread, bench_ulid_new);
criterion_main!(benches);
