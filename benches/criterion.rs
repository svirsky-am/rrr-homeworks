use broken_app::{algo, sum_even,
    sum_even_new_optimized,
    sum_even_with_hot_fix,
    sum_even_by_reference_app,
    normalize, 
    // normalize_by_reference_app,  
    // normalize_faster_new_alt, 
    // average_positive_by_reference_app, 
    average_positive};
use criterion::{criterion_group, criterion_main, BatchSize, Criterion, black_box};

fn bench_sum_even(c: &mut Criterion) {
    let data: Vec<i64> = (0..50_000).collect();
    c.bench_function("sum_even_broken", |b| b.iter(|| sum_even(&data)));
}

fn bench_sum_even_new_optimized(c: &mut Criterion) {
    let data: Vec<i64> = (0..5_000).collect();
    c.bench_function("sum_even_new_optimized", |b| b.iter(|| sum_even_new_optimized(&data)));
}


fn bench_sum_even_with_hot_fix(c: &mut Criterion) {
    let data: Vec<i64> = (0..5_000).collect();
    c.bench_function("sum_even_with_hot_fix", |b| b.iter(|| sum_even_with_hot_fix(&data)));
}


fn bench_sum_even_by_reference_app(c: &mut Criterion) {
    let data: Vec<i64> = (0..5_000).collect();
    c.bench_function("sum_even_by_reference_app", |b| b.iter(|| sum_even_by_reference_app(&data)));
}




fn bench_fib(c: &mut Criterion) {
    c.bench_function("slow_fib_broken", |b| b.iter(|| algo::slow_fib(32)));
}

fn bench_dedup(c: &mut Criterion) {
    let data: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
    c.bench_function("slow_dedup_broken", |b| {
        b.iter_batched(
            || data.clone(),
            |v| {
                let _ = algo::slow_dedup(&v);
            },
            BatchSize::SmallInput,
        )
    });
}

static STRING_TO_NORMALIZE: &str = " Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld  Hel\n\nlo \t\tWor\t\tld ";



fn bench_normalize_with_hot_fix(c: &mut Criterion) {
    // let data: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
    c.bench_function("bench_normalize_with_hot_fix", |b| {

        b.iter(|| normalize(black_box(&STRING_TO_NORMALIZE)));

    });
}


// fn bench_normalize_by_reference_app(c: &mut Criterion) {
//     // let data: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
//     c.bench_function("normalize_by_reference_app", |b| {

//         b.iter(|| normalize_by_reference_app(black_box(&STRING_TO_NORMALIZE)));

//     });
// }

// fn bench_normalize_faster_new_alt(c: &mut Criterion) {
//     // let data: Vec<u64> = (0..5_000).flat_map(|n| [n, n]).collect();
//     c.bench_function("normalize_faster_new_alt", |b| {
//         b.iter(|| normalize_faster_new_alt(black_box(&STRING_TO_NORMALIZE)));
//     });
// }

fn bench_average_positive(c: &mut Criterion) {
    let nums = [-5, 5, 15, -5, 5, 15,-5, 5, 15,-5, 5, 15,-5, 5, 15-5, 5, 15];

    c.bench_function("bench_average_positive", |b| {
        b.iter(|| average_positive(black_box(&nums)));
    });
}


// fn bench_average_positive_by_reference_app(c: &mut Criterion) {
//     let nums = [-5, 5, 15, -5, 5, 15,-5, 5, 15,-5, 5, 15,-5, 5, 15-5, 5, 15];
//     c.bench_function("bench_average_positive_by_reference_app", |b| {
//         b.iter(|| average_positive_by_reference_app(black_box(&nums)));
//     });
// }

criterion_group!(benches_sum_even,
                    bench_sum_even_new_optimized,
                    bench_sum_even_with_hot_fix,
                    bench_sum_even_by_reference_app
);


criterion_group!(benches, bench_fib, bench_dedup, 
    // bench_normalize_by_reference_app,
    // bench_normalize_faster_new_alt,
    bench_average_positive,
    // bench_average_positive_by_reference_app
);
criterion_main!(benches_sum_even, benches);
