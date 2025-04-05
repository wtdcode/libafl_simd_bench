#![feature(portable_simd)]

use chrono::Utc;
use clap::Parser;
use itertools::Itertools;
use rand::{RngCore, rngs::ThreadRng};
use std::simd::cmp::SimdOrd;

struct MaxReducer {}

trait Reducer<T> {
    fn reduce(first: T, second: T) -> T;
}

impl<T> Reducer<T> for MaxReducer
where
    T: PartialOrd,
{
    #[inline]
    fn reduce(first: T, second: T) -> T {
        if first > second { first } else { second }
    }
}

pub struct DifferentIsNovel {}

trait IsNovel<T> {
    fn is_novel(old: T, new: T) -> bool;
}

impl<T> IsNovel<T> for DifferentIsNovel
where
    T: PartialEq + Default + Copy + 'static,
{
    #[inline]
    fn is_novel(old: T, new: T) -> bool {
        old != new
    }
}

fn afl_nightly_simd<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
    type VectorType = core::simd::u8x16;
    let mut novelties = vec![];
    let mut interesting = false;
    let size = map.len();
    let steps = size / VectorType::LEN;
    let left = size % VectorType::LEN;

    if NV {
        novelties.clear();
        for step in 0..steps {
            let i = step * VectorType::LEN;
            let history = VectorType::from_slice(&hist[i..]);
            let items = VectorType::from_slice(&map[i..]);

            if items.simd_max(history) != history {
                interesting = true;
                unsafe {
                    for j in i..(i + VectorType::LEN) {
                        let item = *map.get_unchecked(j);
                        if item > *hist.get_unchecked(j) {
                            novelties.push(j);
                        }
                    }
                }
            }
        }

        for j in (size - left)..size {
            unsafe {
                let item = *map.get_unchecked(j);
                if item > *hist.get_unchecked(j) {
                    interesting = true;
                    novelties.push(j);
                }
            }
        }
    } else {
        for step in 0..steps {
            let i = step * VectorType::LEN;
            let history = VectorType::from_slice(&hist[i..]);
            let items = VectorType::from_slice(&map[i..]);

            if items.simd_max(history) != history {
                interesting = true;
                break;
            }
        }

        if !interesting {
            for j in (size - left)..size {
                unsafe {
                    let item = *map.get_unchecked(j);
                    if item > *hist.get_unchecked(j) {
                        interesting = true;
                        break;
                    }
                }
            }
        }
    }

    (interesting, novelties)
}

fn afl_stable_wide_128<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
    type VectorType = wide::u8x16;
    let mut novelties = vec![];
    let mut interesting = false;
    let size = map.len();
    let steps = size / VectorType::LANES as usize;
    let left = size % VectorType::LANES as usize;

    if NV {
        novelties.clear();
        for step in 0..steps {
            let i = step * VectorType::LANES as usize;
            let history =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());
            let items =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());

            if items.max(history) != history {
                interesting = true;
                unsafe {
                    for j in i..(i + VectorType::LANES as usize) {
                        let item = *map.get_unchecked(j);
                        if item > *hist.get_unchecked(j) {
                            novelties.push(j);
                        }
                    }
                }
            }
        }

        for j in (size - left)..size {
            unsafe {
                let item = *map.get_unchecked(j);
                if item > *hist.get_unchecked(j) {
                    interesting = true;
                    novelties.push(j);
                }
            }
        }
    } else {
        for step in 0..steps {
            let i = step * VectorType::LANES as usize;
            let history =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());
            let items =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());

            if items.max(history) != history {
                interesting = true;
                break;
            }
        }

        if !interesting {
            for j in (size - left)..size {
                unsafe {
                    let item = *map.get_unchecked(j);
                    if item > *hist.get_unchecked(j) {
                        interesting = true;
                        break;
                    }
                }
            }
        }
    }

    (interesting, novelties)
}

fn afl_stable_wide_256<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
    type VectorType = wide::u8x32;
    let mut novelties = vec![];
    let mut interesting = false;
    let size = map.len();
    let steps = size / VectorType::LANES as usize;
    let left = size % VectorType::LANES as usize;

    if NV {
        novelties.clear();
        for step in 0..steps {
            let i = step * VectorType::LANES as usize;
            let history =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());
            let items =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());

            if items.max(history) != history {
                interesting = true;
                unsafe {
                    for j in i..(i + VectorType::LANES as usize) {
                        let item = *map.get_unchecked(j);
                        if item > *hist.get_unchecked(j) {
                            novelties.push(j);
                        }
                    }
                }
            }
        }

        for j in (size - left)..size {
            unsafe {
                let item = *map.get_unchecked(j);
                if item > *hist.get_unchecked(j) {
                    interesting = true;
                    novelties.push(j);
                }
            }
        }
    } else {
        for step in 0..steps {
            let i = step * VectorType::LANES as usize;
            let history =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());
            let items =
                VectorType::new(hist[i..i + VectorType::LANES as usize].try_into().unwrap());

            if items.max(history) != history {
                interesting = true;
                break;
            }
        }

        if !interesting {
            for j in (size - left)..size {
                unsafe {
                    let item = *map.get_unchecked(j);
                    if item > *hist.get_unchecked(j) {
                        interesting = true;
                        break;
                    }
                }
            }
        }
    }

    (interesting, novelties)
}

fn afl_default_impl<const NV: bool, R, N>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>)
where
    R: Reducer<u8>,
    N: IsNovel<u8>,
{
    let mut novelties = vec![];
    let mut interesting = false;
    let initial = 0;
    if NV {
        for (i, item) in map.iter().enumerate().filter(|(_, item)| **item != initial) {
            let existing = unsafe { *hist.get_unchecked(i) };
            let reduced = R::reduce(existing, *item);
            if N::is_novel(existing, reduced) {
                interesting = true;
                novelties.push(i);
            }
        }
    } else {
        for (i, item) in map.iter().enumerate().filter(|(_, item)| **item != initial) {
            let existing = unsafe { *hist.get_unchecked(i) };
            let reduced = R::reduce(existing, *item);
            if N::is_novel(existing, reduced) {
                interesting = true;
                break;
            }
        }
    }

    (interesting, novelties)
}

#[derive(Parser)]
struct CLI {
    #[arg(short, long)]
    pub map: usize,
    #[arg(short, long)]
    pub rounds: usize,
}

fn measure<F>(f: F, hist: &[u8], map: &[u8]) -> (chrono::TimeDelta, bool, Vec<usize>)
where
    F: FnOnce(&[u8], &[u8]) -> (bool, Vec<usize>),
{
    let before = Utc::now();
    let (interesting, novs) = f(hist, map);
    let after = Utc::now();
    (after - before, interesting, novs)
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

fn measure_one<F>(
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
        let (elp, _, _) = measure(f, hist, map);
        outs.push(elp);
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

    // bring two map into cache
    for _ in 0..16 {
        let _ = afl_default_impl::<false, MaxReducer, DifferentIsNovel>(&hist, &map);
    }

    let default_no_novel = measure_one(
        afl_default_impl::<false, MaxReducer, DifferentIsNovel>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let default_novel = measure_one(
        afl_default_impl::<true, MaxReducer, DifferentIsNovel>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let libafl_simd_no_novel = measure_one(
        afl_nightly_simd::<false>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let libafl_simd_novel = measure_one(
        afl_nightly_simd::<true>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let wide128_no_novel = measure_one(
        afl_stable_wide_128::<false>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let wide128_novel = measure_one(
        afl_stable_wide_128::<true>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let wide256_no_novel = measure_one(
        afl_stable_wide_256::<false>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );
    let wide256_novel = measure_one(
        afl_stable_wide_256::<true>,
        &mut hist,
        &mut map,
        &mut rand,
        args.rounds,
    );

    printout("default_no_novel", default_no_novel);
    printout("default_novel", default_novel);
    printout("libafl_simd_no_novel", libafl_simd_no_novel);
    printout("libafl_simd_novel", libafl_simd_novel);
    printout("wide128_no_novel", wide128_no_novel);
    printout("wide128_novel", wide128_novel);
    printout("wide256_no_novel", wide256_no_novel);
    printout("wide256_novel", wide256_novel);
}
