use core::slice;

use wide::CmpEq;

/// Hitcounts class lookup
static COUNT_CLASS_LOOKUP: [u8; 256] = [
    0, 1, 2, 4, 8, 8, 8, 8, 16, 16, 16, 16, 16, 16, 16, 16, 32, 32, 32, 32, 32, 32, 32, 32, 32, 32,
    32, 32, 32, 32, 32, 32, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64, 64,
    64, 64, 64, 64, 64, 64, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
    128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128, 128,
];

/// Hitcounts class lookup for 16-byte values
static mut COUNT_CLASS_LOOKUP_16: Vec<u16> = vec![];

/// Initialize the 16-byte hitcounts map
pub fn init_count_class_16() {
    // # Safety
    //
    // Calling this from multiple threads may be racey and hence leak 65k mem or even create a broken lookup vec.
    // We can live with that.
    unsafe {
        let count_class_lookup_16 = &raw mut COUNT_CLASS_LOOKUP_16;
        let count_class_lookup_16 = &mut *count_class_lookup_16;

        if !count_class_lookup_16.is_empty() {
            return;
        }

        *count_class_lookup_16 = vec![0; 65536];
        for i in 0..256 {
            for j in 0..256 {
                count_class_lookup_16[(i << 8) + j] =
                    (u16::from(COUNT_CLASS_LOOKUP[i]) << 8) | u16::from(COUNT_CLASS_LOOKUP[j]);
            }
        }
    }
}

pub fn afl_classify_counts_naive16(map: &mut [u8]) {
    let mut len = map.len();
    let align_offset = map.as_ptr().align_offset(size_of::<u16>());

    // if len == 1, the next branch will already do this lookup
    if len > 1 && align_offset != 0 {
        debug_assert_eq!(
            align_offset, 1,
            "Aligning u8 to u16 should always be offset of 1?"
        );
        unsafe {
            *map.get_unchecked_mut(0) =
                *COUNT_CLASS_LOOKUP.get_unchecked(*map.get_unchecked(0) as usize);
        }
        len -= 1;
    }

    // Fix the last element
    if (len & 1) != 0 {
        unsafe {
            *map.get_unchecked_mut(len - 1) =
                *COUNT_CLASS_LOOKUP.get_unchecked(*map.get_unchecked(len - 1) as usize);
        }
    }

    let cnt = len / 2;

    let map16 =
        unsafe { slice::from_raw_parts_mut(map.as_mut_ptr().add(align_offset) as *mut u16, cnt) };
    let count_class_lookup_16 = &raw mut COUNT_CLASS_LOOKUP_16;

    // 2022-07: Adding `enumerate` here increases execution speed/register allocation on x86_64.
    #[expect(clippy::unused_enumerate_index)]
    for (_i, item) in map16[0..cnt].iter_mut().enumerate() {
        unsafe {
            let count_class_lookup_16 = &mut *count_class_lookup_16;
            *item = *(*count_class_lookup_16).get_unchecked(*item as usize);
        }
    }
}


pub fn afl_simplify_trace_naive(map: &mut [u8]) {
    for it in map.iter_mut() {
        *it = if *it == 0 { 0x1 } else { 0x80 };
    }
}

pub fn afl_simplify_trace_wide128(map: &mut [u8]) {
    type VectorType = wide::u8x16;
    let size = map.len();
    const bs: usize = VectorType::LANES as usize;
    let steps = size / bs;
    let left = size % bs;
    let lhs = VectorType::new([0x1; bs]);
    let rhs = VectorType::new([0x80; bs]);

    for step in 0..steps {
        let i = step * bs;
        let mp = VectorType::new(map[i..(i+bs)].try_into().unwrap());

        let mask = mp.cmp_eq(VectorType::ZERO);
        // let out = lhs.blend(rhs, mask);
        let out = mask.blend(lhs, rhs);
        // println!("out {}, lhs {}, rhs {}, mask {}", out, lhs, rhs, mask);
        map[i..i + bs].copy_from_slice(out.as_array_ref());
    }

    for j in (size - left)..size {
        map[j] = if map[j] == 0 { 0x1 } else { 0x80 }
    }
}



pub fn afl_simplify_trace_wide256(map: &mut [u8]) {
    type VectorType = wide::u64x4;
    let size = map.len();
    const bs: usize = 8 * VectorType::LANES as usize;
    let steps = size / bs;
    let left = size % bs;
    let lhs = VectorType::new([0x01010101010101; 4]);
    let rhs = VectorType::new([0x80808080808080; 4]);

    for step in 0..steps {
        let i = step * bs;
        let buf: [u8; 32] = map[i..i+bs].try_into().unwrap();
        let mp = VectorType::new(unsafe {std::mem::transmute::<[u8; 32], [u64; 4]>(buf)});

        let mask = mp.cmp_eq(VectorType::ZERO);
        // let out = lhs.blend(rhs, mask);
        let out = mask.blend(lhs, rhs);
        // println!("out {}, lhs {}, rhs {}, mask {}", out, lhs, rhs, mask);
        // map[i..i + bs].copy_from_slice(out.as_array_ref());
        unsafe {
            (out.as_array_ref().as_ptr() as *const u8)
                .copy_to_nonoverlapping(map.as_mut_ptr().add(i), bs);
        }
    }

    for j in (size - left)..size {
        map[j] = if map[j] == 0 { 0x1 } else { 0x80 }
    }
}
