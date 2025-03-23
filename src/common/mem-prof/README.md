# Profile memory usage of myapp

This crate provides an easy approach to dump memory profiling info.

## Prerequisites

### jemalloc

```bash
# for macOS
brew install jemalloc

# for Ubuntu
sudo apt install libjemalloc-dev
```

### [flamegraph](https://github.com/brendangregg/FlameGraph) 

```bash
curl https://raw.githubusercontent.com/brendangregg/FlameGraph/master/flamegraph.pl > ./flamegraph.pl 
```

### Build myapp with `mem-prof` feature.

```bash
cargo build --features=mem-prof
```

## Profiling

Start myapp instance with environment variables:

```bash
MALLOC_CONF=prof:true,lg_prof_interval:28 ./target/debug/myapp
```

Dump memory profiling data through HTTP API:

```bash
curl localhost:4000/v1/prof/mem > myapp.hprof
```

You can periodically dump profiling data and compare them to find the delta memory usage.

## Analyze profiling data with flamegraph

To create flamegraph according to dumped profiling data:

```bash
jeprof --svg <path_to_myapp_binary> --base=<baseline_prof> <profile_data> > output.svg
```
