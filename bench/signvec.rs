use signvec::{SignVec, Sign};
use nanorand::{WyRand, Rng};
use criterion::{black_box, Criterion, BenchmarkId, criterion_group, criterion_main};

fn bench_signvec_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("SignVec_Operations");
    group.noise_threshold(0.05);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.warm_up_time(std::time::Duration::from_secs(5));

    let mut rng = WyRand::new();    
    let data: Vec<i32> = (0..1000).map(|_| rng.generate_range(-5000i32..=5000)).collect();
    let mut sign_vec = SignVec::<i32>::with_capacity(100_000);
    data.iter().for_each(|&val| sign_vec.push(val));

    group.bench_function("set", |b| {
        b.iter(|| {
            let idx = rng.generate_range(0usize..1000);
            sign_vec.set(black_box(idx), black_box(data[idx]));
        })
    });

    group.bench_function("set_unchecked", |b| {
        b.iter(|| {
            let idx = rng.generate_range(0usize..1000);
            sign_vec.set_unchecked(black_box(idx), black_box(data[idx]));
        })
    });

    group.bench_function("random (Sign::Plus)", |b| {
        b.iter(|| {
            sign_vec.random(black_box(Sign::Plus), black_box(&mut rng));
        })
    });

    group.bench_function("random (Sign::Minus)", |b| {
        b.iter(|| {
            sign_vec.random(black_box(Sign::Minus), black_box(&mut rng));
        })
    });

    group.bench_function("random_pos", |b| {
        b.iter(|| {
            sign_vec.random_pos(black_box(&mut rng));
        })
    });

    group.bench_function("random_neg", |b| {
        b.iter(|| {
            sign_vec.random_neg(black_box(&mut rng));
        })
    });

    group.bench_function("count (Sign::Plus)", |b| {
        b.iter(|| {
            black_box(sign_vec.count(black_box(Sign::Plus)));
        })
    });

    group.bench_function("count (Sign::Minus)", |b| {
        b.iter(|| {
            black_box(sign_vec.count(black_box(Sign::Minus)));
        })
    });

    group.bench_function("count_pos", |b| {
        b.iter(|| {
            black_box(sign_vec.count_pos());
        })
    });

    group.bench_function("count_neg", |b| {
        b.iter(|| {
            black_box(sign_vec.count_neg());
        })
    });

    group.bench_function("indices (Sign::Plus)", |b| {
        b.iter(|| {
            black_box(sign_vec.indices(black_box(Sign::Plus)));
        })
    });

    group.bench_function("indices (Sign::Minus)", |b| {
        b.iter(|| {
            black_box(sign_vec.indices(black_box(Sign::Minus)));
        })
    });

    group.bench_function("indices_pos", |b| {
        b.iter(|| {
            black_box(sign_vec.indices_pos());
        })
    });

    group.bench_function("indices_neg", |b| {
        b.iter(|| {
            black_box(sign_vec.indices_neg());
        })
    });

    group.bench_function("sync", |b| {
        b.iter(|| {
            sign_vec.sync();
        })
    });

    group.finish();
}

fn bench_signvec_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("SignVec Vec Comparison");
    group.noise_threshold(0.05);
    group.sampling_mode(criterion::SamplingMode::Flat);
    group.warm_up_time(std::time::Duration::from_secs(5));

    let mut rng = WyRand::new();
    let data: Vec<i32> = (0..1000).map(|_| rng.generate_range(-5000i32..=5000)).collect();
    let mut sign_vec = SignVec::<i32>::with_capacity(100_000);
    let mut vec: Vec<i32> = Vec::with_capacity(100_000);
    data.iter().for_each(|&val| {
        sign_vec.push(val);
        vec.push(val);
    });
    let operations = [Sign::Plus, Sign::Minus];

    // Benchmark for `count`
    for op in operations.iter().cloned() {
        group.bench_with_input(BenchmarkId::new("count_svec", format!("{:?}", op)), &data, |b, _data| {
            b.iter(|| {
                let count = sign_vec.count(op);
                black_box(count);
            });
        });

        group.bench_with_input(BenchmarkId::new("count_vec", format!("{:?}", op)), &data, |b, _data| {
            b.iter(|| {
                let count = vec.iter().filter(|&&x| (op == Sign::Plus && x > 0) || (op == Sign::Minus && x < 0)).count();
                black_box(count);
            });
        });
    }

    // Benchmarks for `indices`
    for op in operations.iter().cloned() {
        group.bench_with_input(BenchmarkId::new("indices_svec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                let indices: Vec<_> = sign_vec.indices(op).iter().collect();
                black_box(indices);
            });
        });

        group.bench_with_input(BenchmarkId::new("indices_vec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                let indices: Vec<_> = vec.iter().enumerate()
                    .filter_map(|(i, &x)| if (op == Sign::Plus && x > 0) || (op == Sign::Minus && x < 0) { Some(i) } else { None })
                    .collect();
                black_box(indices);
            });
        });
    }

    // Benchmarks for `values`
    for op in operations.iter().cloned() {
        group.bench_with_input(BenchmarkId::new("values_svec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                let values: Vec<&i32> = sign_vec.values(op).collect();
                black_box(values);
            });
        });

        group.bench_with_input(BenchmarkId::new("values_vec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                let values: Vec<i32> = vec.iter().cloned()
                    .filter(|&x| (op == Sign::Plus && x > 0) || (op == Sign::Minus && x < 0))
                    .collect();
                black_box(values);
            });
        });
    }

    // Benchmarks for `random`
    for op in operations.iter().cloned() {
        group.bench_with_input(BenchmarkId::new("random_svec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                let random_value = sign_vec.random(op, &mut rng);
                black_box(random_value);
            });
        });

        group.bench_with_input(BenchmarkId::new("random_vec", format!("{:?}", op)), &data, |b, _| {
            b.iter(|| {
                // For Vec, we simulate a similar 'random' operation
                let filtered_values: Vec<i32> = vec.iter().cloned()
                    .filter(|&x| (op == Sign::Plus && x > 0) || (op == Sign::Minus && x < 0))
                    .collect();
                if !filtered_values.is_empty() {
                    let random_index = rng.generate_range(0..filtered_values.len());
                    let random_value = filtered_values[random_index];
                    black_box(random_value);
                }
            });
        });
    }
    group.finish();
}


criterion_group! {
    name = benches;
    config = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(30))
        .without_plots()
        .confidence_level(0.95)
        .significance_level(0.05)
        .configure_from_args();
    targets = bench_signvec_operations, bench_signvec_comparison
}

criterion_main!(benches);