use criterion::{criterion_group, criterion_main, Criterion};
use uid_generator_rs::{Snowflake, Ulid, DEFAULT_EPOCH_MS};

fn bench_snowflake(c: &mut Criterion) {
    let sf = Snowflake::new(1, DEFAULT_EPOCH_MS).expect("Snowflake::new failed");
    c.bench_function("snowflake/next_id", |b| {
        b.iter(|| sf.next_id().unwrap());
    });
}

fn bench_ulid(c: &mut Criterion) {
    c.bench_function("ulid/new", |b| {
        b.iter(|| Ulid::new().unwrap());
    });
}

criterion_group!(benches, bench_snowflake, bench_ulid);
criterion_main!(benches);
