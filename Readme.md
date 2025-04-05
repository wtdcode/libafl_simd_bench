# Coverage Map Benchmark

Benchmark coverage map (Max Reducer) using naive, `std::simd` and `wide`.

CPU is i9-13900K with 64G memory. Coverage map is `2097152` and repeat `655365` rounds, i.e.

```bash
tasket -c 3 ./target/release/libafl_simd_bench -m 2097152 -r 32768
```

Results of total running time (lower is btter):

Compile with just `cargo build --release`

|Naive|`std::simd`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|16.595|9.461|0.777|0.389|

Complile with `RUSTFLAGS='-C target-cpu=native' cargo build --release`

|Naive|`std::simd`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|16.627|9.465|0.777|0.389|