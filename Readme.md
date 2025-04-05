# Coverage Map Benchmark

Benchmark coverage map (Max Reducer) using naive, `std::simd` and `wide`.

CPU is i9-13900K with 64G memory. Coverage map is `2097152` and repeat `32768` rounds, i.e.

```bash
taskset -c 3 ./target/release/libafl_simd_bench -m 2097152 -r 32768
```

Results of total running time (lower is btter):

Compile with just `cargo build --release`

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|16.980|9.939|9.787|13.911|

Complile with `RUSTFLAGS='-C target-cpu=native' cargo build --release`

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|17.235|10.068|10.049|13.635|

On Oracle 4C24G aarch64 machine with `8192` repetitions.

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|12.641|2.620|2.674|2.870|

To check correctness, build it with `cargo build --release --features correctness`