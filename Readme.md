# Coverage Map Benchmark

Benchmark coverage map (Max Reducer) using naive, `std::simd` and `wide`.

CPU is i9-13900K with 64G memory. Coverage map is `2097152` and repeat `32768` rounds, i.e.

```bash
taskset -c 3 ./target/release/libafl_simd_bench -m 2097152 -r 32768
```

Results of total running time (lower is btter):

Complile with `RUSTFLAGS='-C target-cpu=native' cargo build --release`

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|17.089|10.739|9.520|9.625|

## Benchmarks On Other Machines

On another AMD EPYC 7B13:

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|24.965|16.575|15.310|15.688|

On Oracle 4C24G aarch64 machine with `8192` repetitions.

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|12.564|3.141|2.945|2.563|

On m2 pro with `16384` repetitions (potentially inaccurate results due to core binding not available on macOS)

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|21.373|4.617|4.365|3.984|

To check correctness, build it with `cargo build --release --features correctness`

To run miri:

```
MIRIFLAGS="-Zmiri-disable-isolation" cargo miri run -- -m 256 -r 256 -p cov
```