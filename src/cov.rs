use chrono::Utc;
use clap::Parser;
use itertools::Itertools;
use rand::{RngCore, rngs::ThreadRng};
use wide::CmpEq;
use std::simd::cmp::SimdOrd;

pub struct MaxReducer {}

pub trait Reducer<T> {
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

pub trait IsNovel<T> {
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

pub fn afl_nightly_simd<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
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

pub fn afl_stable_wide_128<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
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
            let items = VectorType::new(map[i..i + VectorType::LANES as usize].try_into().unwrap());

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
            let items = VectorType::new(map[i..i + VectorType::LANES as usize].try_into().unwrap());

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


pub fn afl_stable_wide_256<const NV: bool>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>) {
    type VectorType = wide::u32x4;
    let mut novelties = vec![];
    let mut interesting = false;
    const bs: usize =  4 * VectorType::LANES as usize;
    let size = map.len();
    let steps = size / bs;
    let left = size % bs;
    
    if NV {
        novelties.clear();
        for step in 0..steps {
            let i = step * bs;
            let buf: [u8; bs] = hist[i..i+bs].try_into().unwrap();
            let history =
                VectorType::new(unsafe {std::mem::transmute(buf)});
            let buf: [u8; bs] = map[i..i+bs].try_into().unwrap();
            let items = VectorType::new(unsafe {std::mem::transmute(buf)});

            if items.max(history) != history {
                interesting = true;
                unsafe {
                    for j in i..(i + bs / 2) {
                        let item = *map.get_unchecked(j);
                        if item > *hist.get_unchecked(j) {
                            novelties.push(j);
                        }
                    }

                    for j in (i + bs / 2)..(i + bs as usize)
                    {
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
            let i = step * bs;
            let buf: [u8; bs] = hist[i..i+bs].try_into().unwrap();
            let history =
                VectorType::new(unsafe {std::mem::transmute(buf)});
            let buf: [u8; bs] = map[i..i+bs].try_into().unwrap();
            let items = VectorType::new(unsafe {std::mem::transmute(buf)});

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

pub fn afl_default_impl<const NV: bool, R, N>(hist: &[u8], map: &[u8]) -> (bool, Vec<usize>)
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
