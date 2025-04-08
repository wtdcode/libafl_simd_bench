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
|16.380|10.333|9.470|10.554|

## Benchmarks On Other Machines

On another AMD EPYC 7B13:

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|24.797|16.791|15.205|16.129|

On Oracle 4C24G aarch64 machine with `8192` repetitions.

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|12.540|3.120|2.924|2.821|

On m2 pro with `16384` repetitions (potentially inaccurate results due to core binding not available on macOS)

|Naive|`std::simd::u8x16`|`wide::u8x16`|`wide::u8x32`|
|-|-|-|-|
|21.590|4.410|4.388|4.329|

To check correctness, build it with `cargo build --release --features correctness`

To run miri:

```
MIRIFLAGS="-Zmiri-disable-isolation" cargo miri run -- -m 256 -r 256 -p cov
```