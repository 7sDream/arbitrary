use bezier::{Poly, SturmSeq};
use criterion::{criterion_group, criterion_main, Criterion};

fn poly_sturm() {
    let poly: Poly = [1.0, -4.0, 2.0, 0.0, -3.0, 7.0].into_iter().collect();
    let _ = poly.real_roots();
}

fn solve_poly(coefficients: [f64; 6]) -> Option<(faer_core::Mat<f64>, faer_core::Mat<f64>)> {
    // find the degree of function, remove leading zeros
    let mut degree: Option<usize> = None;
    for (i, c) in coefficients.iter().enumerate() {
        if *c != 0.0 {
            degree.replace(i);
            break;
        }
    }

    // if all coefficients is zero, or only contains constant item
    // we assume there is no nearest point
    let d = match degree {
        None | Some(5) => return None,
        Some(d) => d,
    };

    // size of companion matrix
    let size = 5 - d;

    // construct the companion matrix of polynomial
    // a_{0..n-1} is **normalized** coefficients, from low to high degree
    //
    // 0.0 0.0 ... 0.0 -a_0
    // 1.0 0.0 ... 0.0 -a_1
    // 0.0 1.0 ... 0.0 -a_2
    // ... ... ... ... ...
    // 0.0 0.0 ... 1.0 -a_{n-1}
    let mat = faer_core::Mat::from_fn(size, size, |i, j| {
        if j + 1 == size {
            -coefficients[5 - i] / coefficients[d]
        } else if i == j + 1 {
            1.0
        } else {
            0.0
        }
    });

    // EVD decomposition to solve the origin polynomial
    let req = faer_evd::compute_evd_req::<f64>(
        size,
        // TODO: figure out why this set to No will cause wasm32 OOM when calculating
        faer_evd::ComputeVectors::Yes,
        faer_core::Parallelism::None,
        faer_evd::EvdParams::default(),
    )
    .ok()?;

    // TODO: make buffer poll for this maybe
    let mut buffer = vec![0u8; req.size_bytes()];
    let mut re = faer_core::Mat::zeros(size, 1);
    let mut im = faer_core::Mat::zeros(size, 1);

    faer_evd::compute_evd_real::<f64>(
        mat.as_ref(),
        re.as_mut(),
        im.as_mut(),
        None, // we do not need eigenvectors
        faer_core::Parallelism::None,
        dyn_stack::PodStack::new(&mut buffer),
        faer_evd::EvdParams::default(),
    );

    Some((re, im))
}

fn poly_evd() {
    let _ = solve_poly([1.0, -4.0, 2.0, 0.0, -3.0, 7.0]);
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("poly_sturm", |b| b.iter(|| poly_sturm()));
    c.bench_function("poly_evd", |b| b.iter(|| poly_evd()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
