use chrono::Utc;
use clap::Parser;
use counts::afl_simplify_trace_naive;
use itertools::Itertools;
use rand::{RngCore, rngs::ThreadRng};

use libafl_simd_bench::counts::*;
use libafl_simd_bench::cov::*;

mod counts;

#[derive(Parser)]
struct CLI {
    #[arg(short, long)]
    pub map: usize,
    #[arg(short, long)]
    pub rounds: usize,
    #[arg(short, long, default_value = "cov")]
    pub program: String,
}

fn measure_cov<F>(f: F, hist: &[u8], map: &[u8]) -> (chrono::TimeDelta, bool, Vec<usize>)
where
    F: FnOnce(&[u8], &[u8]) -> (bool, Vec<usize>),
{
    let before = Utc::now();
    let (interesting, novs) = f(hist, map);
    let after = Utc::now();
    (after - before, interesting, novs)
}

fn measure_simpliy_counts<F>(f: F, map: &mut [u8]) -> chrono::TimeDelta
where
    F: FnOnce(&mut [u8]) -> (),
{
    let before = Utc::now();
    f(map);
    let after = Utc::now();
    after - before
}

fn random_bits(map: &mut [u8], rng: &mut ThreadRng) {
    // randomly set a bit since coverage map is usually sparse enough
    let rng = rng.next_u64() as usize;
    let bytes_idx = (rng / 8) % map.len();
    let bits_idx = rng % 8;
    map[bytes_idx] |= 1 << bits_idx;
}

fn clean_vectors(map: &mut [u8]) {
    for it in map.iter_mut() {
        *it = 0;
    }
}

fn measure_rounds<F>(
    f: F,
    hist: &mut [u8],
    map: &mut [u8],
    rng: &mut ThreadRng,
    rounds: usize,
) -> Vec<chrono::Duration>
where
    F: FnOnce(&[u8], &[u8]) -> (bool, Vec<usize>) + Copy,
{
    let mut outs = Vec::with_capacity(rounds);
    clean_vectors(map);
    clean_vectors(hist);
    for _ in 0..rounds {
        random_bits(map, rng);
        #[cfg(feature = "correctness")]
        {
            let (elp, interesting, nov) = measure_cov(f, hist, map);
            let (_, canonical_interesting, canonical_nov) = measure_cov(
                afl_default_impl::<true, MaxReducer, DifferentIsNovel>,
                hist,
                map,
            );
            if interesting != canonical_interesting || nov != canonical_nov {
                panic!(
                    "Incorrect! {} vs {}, {:?} vs {:?}",
                    interesting, canonical_interesting, nov, canonical_nov
                );
            }
            outs.push(elp);
        }
        #[cfg(not(feature = "correctness"))]
        {
            let (elp, _, _) = measure_cov(f, hist, map);
            outs.push(elp);
        }
    }
    outs
}

fn measure_counts_rounds<F>(
    f: F,
    map: &mut [u8],
    rng: &mut ThreadRng,
    rounds: usize,
) -> Vec<chrono::Duration>
where
    F: FnOnce(&mut [u8]) -> () + Copy,
{
    let mut outs = Vec::with_capacity(rounds);
    clean_vectors(map);

    for _ in 0..rounds {
        random_bits(map, rng);
        #[cfg(feature = "correctness")]
        {
            let mut canonical = map.to_vec();
            let elp = measure_simpliy_counts(f, map);
            afl_simplify_trace_naive(&mut canonical);

            if map != &mut canonical {
                panic!("Incorrect! {:?} vs\n{:?}", map, canonical);
            }
            outs.push(elp);
        }

        #[cfg(not(feature = "correctness"))]
        {
            let elp = measure_simpliy_counts(f, map);
            outs.push(elp);
        }
    }
    outs
}

fn printout(ty: &str, tms: Vec<chrono::TimeDelta>) {
    let tms = tms
        .iter()
        .map(|t| t.to_std().unwrap().as_secs_f64())
        .collect_vec();
    let mean = tms.iter().sum::<f64>() / tms.len() as f64;
    let min = tms.iter().fold(0f64, |acc, x| acc.min(*x));
    let max = tms.iter().fold(0f64, |acc, x| acc.max(*x));
    let std = (tms
        .iter()
        .fold(0f64, |acc, x| acc + (*x - mean) * (*x - mean))
        / (tms.len() - 1) as f64)
        .sqrt();
    let sum: f64 = tms.into_iter().sum();
    println!(
        "{}: avg {:.03}, min {:.03}, max {:.03}, std {:.03}, sum {:.03}",
        ty, mean, min, max, std, sum
    );
}

fn main() {
    let args = CLI::parse();
    let mut map = vec![0; args.map];
    let mut hist = vec![0; args.map];
    let mut rand = rand::rng();

    if args.program == "cov" {
        // bring two map into cache
        for _ in 0..16 {
            let _ = afl_default_impl::<false, MaxReducer, DifferentIsNovel>(&hist, &map);
        }

        println!("Naive implmentation...");
        #[cfg(not(feature = "correctness"))]
        let default_no_novel = measure_rounds(
            afl_default_impl::<false, MaxReducer, DifferentIsNovel>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        let default_novel = measure_rounds(
            afl_default_impl::<true, MaxReducer, DifferentIsNovel>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        println!("std::simd implmentation...");
        #[cfg(not(feature = "correctness"))]
        let libafl_simd_no_novel = measure_rounds(
            afl_nightly_simd::<false>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        let libafl_simd_novel = measure_rounds(
            afl_nightly_simd::<true>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        println!("wide128 implmentation...");
        #[cfg(not(feature = "correctness"))]
        let wide128_no_novel = measure_rounds(
            afl_stable_wide_128::<false>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        let wide128_novel = measure_rounds(
            afl_stable_wide_128::<true>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        println!("wide256 implmentation...");
        #[cfg(not(feature = "correctness"))]
        let wide256_no_novel = measure_rounds(
            afl_stable_wide_256::<false>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );
        let wide256_novel = measure_rounds(
            afl_stable_wide_256::<true>,
            &mut hist,
            &mut map,
            &mut rand,
            args.rounds,
        );

        #[cfg(not(feature = "correctness"))]
        printout("default_no_novel", default_no_novel);
        printout("default_novel", default_novel);
        #[cfg(not(feature = "correctness"))]
        printout("libafl_simd_no_novel", libafl_simd_no_novel);
        printout("libafl_simd_novel", libafl_simd_novel);
        #[cfg(not(feature = "correctness"))]
        printout("wide128_no_novel", wide128_no_novel);
        printout("wide128_novel", wide128_novel);
        #[cfg(not(feature = "correctness"))]
        printout("wide256_no_novel", wide256_no_novel);
        printout("wide256_novel", wide256_novel);
    } else if args.program == "counts" {
        println!("Naive simplify_counts...");
        let simplify_naive =
            measure_counts_rounds(afl_simplify_trace_naive, &mut map, &mut rand, args.rounds);
        println!("wide128 simplify counts...");
        let simplify_wide128 =
            measure_counts_rounds(afl_simplify_trace_wide128, &mut map, &mut rand, args.rounds);
        println!("wide256 simplify counts...");
        let simplify_wide256 =
                measure_counts_rounds(afl_simplify_trace_wide256, &mut map, &mut rand, args.rounds);
    
        printout("simplify_naive", simplify_naive);
        printout("simplify_wide128", simplify_wide128);
        printout("simplify_wide256", simplify_wide256);
    } else {
        panic!("no such bench {}", args.program);
    }
}
