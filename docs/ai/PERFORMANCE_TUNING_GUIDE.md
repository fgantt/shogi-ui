# Performance Tuning Guide

Comprehensive guide for optimizing the performance of the automated tuning system.

## Table of Contents

1. [Hardware Optimization](#hardware-optimization)
2. [System Configuration](#system-configuration)
3. [Memory Optimization](#memory-optimization)
4. [CPU Optimization](#cpu-optimization)
5. [Storage Optimization](#storage-optimization)
6. [Network Optimization](#network-optimization)
7. [Algorithm-Specific Tuning](#algorithm-specific-tuning)
8. [Benchmarking and Profiling](#benchmarking-and-profiling)

## Hardware Optimization

### CPU Optimization

#### Multi-Core Systems
```bash
# Optimal thread configuration
# For CPU-bound workloads:
threads = min(CPU_cores, dataset_size / 10000)

# Example for 16-core system with 500K positions:
./target/release/tuner tune --threads 16 --batch-size 1000

# For I/O-bound workloads:
threads = CPU_cores * 2

# Example:
./target/release/tuner tune --threads 32 --batch-size 500
```

#### CPU Affinity
```bash
# Pin to specific cores for NUMA systems
taskset -c 0-15 ./target/release/tuner tune [options]

# Use all cores on first NUMA node
numactl --cpunodebind=0 ./target/release/tuner tune [options]

# Use specific CPU cores
numactl --physcpubind=0,2,4,6,8,10,12,14 ./target/release/tuner tune [options]
```

#### CPU Frequency Scaling
```bash
# Set performance mode (Linux)
echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

# Disable CPU power management
echo 1 | sudo tee /sys/devices/system/cpu/intel_pstate/no_turbo

# Set fixed frequency
echo 3500000 | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_min_freq
echo 3500000 | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_max_freq
```

### Memory Optimization

#### Memory Allocation
```bash
# Optimal memory allocation based on dataset size
# Small dataset (<100K positions):
memory_limit = 4GB

# Medium dataset (100K-1M positions):
memory_limit = 8GB

# Large dataset (>1M positions):
memory_limit = 16GB+

# Example configurations:
./target/release/tuner tune --memory-limit 8192 --dataset medium_games.json
./target/release/tuner tune --memory-limit 16384 --dataset large_games.json
```

#### Memory Pool Configuration
```bash
# Pre-allocate memory pools
./target/release/tuner tune --memory-pools --pool-size 1024

# Use memory-mapped files for large datasets
./target/release/tuner tune --mmap-dataset --mmap-size 2147483648

# Enable memory compression
./target/release/tuner tune --compress-memory --compression-level 6
```

#### NUMA Memory Optimization
```bash
# Bind to specific NUMA node
numactl --membind=0 ./target/release/tuner tune [options]

# Interleave memory across NUMA nodes
numactl --interleave=all ./target/release/tuner tune [options]

# Use local memory allocation
numactl --localalloc ./target/release/tuner tune [options]
```

### Storage Optimization

#### SSD Optimization
```bash
# Use SSD for checkpoints and temporary files
./target/release/tuner tune --checkpoint-dir /ssd/checkpoints --temp-dir /ssd/temp

# Optimize SSD settings (Linux)
echo mq-deadline | sudo tee /sys/block/nvme0n1/queue/scheduler
echo 1 | sudo tee /sys/block/nvme0n1/queue/nomerges
```

#### File System Optimization
```bash
# Use XFS for better performance with large files
sudo mkfs.xfs /dev/sdb
sudo mount -o noatime,nodiratime /dev/sdb /data

# Optimize ext4 for tuning workloads
sudo tune2fs -o journal_data_writeback /dev/sdb
sudo mount -o noatime,nodiratime,data=writeback /dev/sdb /data
```

#### I/O Scheduler Optimization
```bash
# Set optimal I/O scheduler
echo mq-deadline | sudo tee /sys/block/nvme0n1/queue/scheduler  # NVMe
echo bfq | sudo tee /sys/block/sda/queue/scheduler              # SATA SSD
echo mq-deadline | sudo tee /sys/block/sda/queue/scheduler      # SATA HDD
```

## System Configuration

### Operating System Tuning

#### Linux Kernel Parameters
```bash
# Optimize for high-throughput workloads
echo 'vm.swappiness=1' >> /etc/sysctl.conf
echo 'vm.dirty_ratio=15' >> /etc/sysctl.conf
echo 'vm.dirty_background_ratio=5' >> /etc/sysctl.conf
echo 'vm.dirty_writeback_centisecs=1500' >> /etc/sysctl.conf
echo 'vm.dirty_expire_centisecs=3000' >> /etc/sysctl.conf

# Apply changes
sysctl -p
```

#### Process Limits
```bash
# Increase file descriptor limits
echo '* soft nofile 65536' >> /etc/security/limits.conf
echo '* hard nofile 65536' >> /etc/security/limits.conf

# Increase process limits
echo '* soft nproc 32768' >> /etc/security/limits.conf
echo '* hard nproc 32768' >> /etc/security/limits.conf
```

#### Network Tuning
```bash
# Optimize TCP settings for large data transfers
echo 'net.core.rmem_max=134217728' >> /etc/sysctl.conf
echo 'net.core.wmem_max=134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_rmem=4096 87380 134217728' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_wmem=4096 65536 134217728' >> /etc/sysctl.conf
```

### Container Optimization

#### Docker Configuration
```dockerfile
# Optimized Dockerfile
FROM rust:1.70-slim

# Install system dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Set optimal environment variables
ENV RUSTFLAGS="-C target-cpu=native"
ENV CARGO_TARGET_DIR=/tmp/cargo

# Build with optimizations
RUN cargo build --release --bin tuner

# Runtime optimizations
ENV OMP_NUM_THREADS=16
ENV MALLOC_ARENA_MAX=2
ENV RUST_LOG=info
```

#### Resource Limits
```bash
# Run with resource constraints
docker run --cpus=16 --memory=32g --memory-swap=32g \
  --ulimit nofile=65536:65536 \
  tuner:latest tune --dataset games.json --output weights.json
```

## Memory Optimization

### Memory Layout Optimization

#### Data Structure Optimization
```rust
// Use cache-friendly data structures
#[repr(C)]
struct CacheOptimizedPosition {
    features: [f64; 2000],  // Aligned to cache line
    result: GameResult,
    phase: GamePhase,
}

// Use memory pools for frequent allocations
let mut position_pool = MemoryPool::new(10000);
```

#### Memory Access Patterns
```bash
# Use sequential access patterns
./target/release/tuner tune --access-pattern sequential

# Enable memory prefetching
./target/release/tuner tune --prefetch-enabled

# Use memory mapping for large datasets
./target/release/tuner tune --mmap-dataset
```

### Garbage Collection Optimization

#### Rust Memory Management
```bash
# Optimize memory allocation
RUSTFLAGS="-C opt-level=3 -C target-cpu=native" cargo build --release

# Use jemalloc for better memory management
RUSTFLAGS="-C link-arg=-Wl,--no-as-needed" cargo build --release --features jemalloc
```

#### Manual Memory Management
```bash
# Enable manual memory management
./target/release/tuner tune --manual-memory-management

# Use memory pools
./target/release/tuner tune --memory-pools --pool-size 2048

# Enable memory compaction
./target/release/tuner tune --compact-memory --compaction-frequency 1000
```

## CPU Optimization

### Parallel Processing

#### Thread Pool Configuration
```bash
# Optimal thread count calculation
# CPU-bound: threads = CPU_cores
# I/O-bound: threads = CPU_cores * 2
# Mixed: threads = CPU_cores * 1.5

# Examples:
./target/release/tuner tune --threads 16    # 16-core CPU-bound
./target/release/tuner tune --threads 32    # 16-core I/O-bound
./target/release/tuner tune --threads 24    # 16-core mixed workload
```

#### Work Stealing
```bash
# Enable work-stealing scheduler
./target/release/tuner tune --work-stealing

# Use custom thread pool
./target/release/tuner tune --custom-thread-pool --pool-size 16
```

### SIMD Optimization

#### Vectorization
```bash
# Enable SIMD optimizations
RUSTFLAGS="-C target-feature=+avx2,+fma" cargo build --release

# Use specific SIMD instructions
./target/release/tuner tune --simd-enabled --simd-width 256
```

#### CPU Feature Detection
```bash
# Auto-detect CPU features
./target/release/tuner tune --auto-detect-cpu-features

# Force specific CPU features
./target/release/tuner tune --cpu-features avx2,fma,sse4.2
```

## Storage Optimization

### I/O Optimization

#### Asynchronous I/O
```bash
# Use async I/O for better throughput
./target/release/tuner tune --async-io --io-depth 32

# Enable I/O batching
./target/release/tuner tune --io-batching --batch-size 1024
```

#### Buffering
```bash
# Optimize buffer sizes
./target/release/tuner tune --read-buffer-size 65536 --write-buffer-size 65536

# Use memory-mapped I/O
./target/release/tuner tune --mmap-io --mmap-size 1073741824
```

### Checkpoint Optimization

#### Checkpoint Strategy
```bash
# Optimize checkpoint frequency
# For fast storage (NVMe): frequent checkpoints
./target/release/tuner tune --checkpoint-frequency 50

# For slow storage (HDD): infrequent checkpoints
./target/release/tuner tune --checkpoint-frequency 500

# Compress checkpoints for space efficiency
./target/release/tuner tune --compress-checkpoints --compression-level 6
```

#### Checkpoint Storage
```bash
# Use separate storage for checkpoints
./target/release/tuner tune --checkpoint-dir /fast-storage/checkpoints

# Use network storage for checkpoints
./target/release/tuner tune --checkpoint-url nfs://server/checkpoints

# Enable checkpoint deduplication
./target/release/tuner tune --deduplicate-checkpoints
```

## Algorithm-Specific Tuning

### Adam Optimizer Tuning

#### Learning Rate Scheduling
```bash
# Use learning rate decay
./target/release/tuner tune --method adam --lr-decay 0.95 --lr-decay-frequency 1000

# Use cosine annealing
./target/release/tuner tune --method adam --lr-schedule cosine --lr-min 0.001

# Use warm restarts
./target/release/tuner tune --method adam --warm-restarts --restart-frequency 2000
```

#### Adaptive Parameters
```bash
# Tune beta parameters for stability
./target/release/tuner tune --method adam --beta1 0.9 --beta2 0.999 --epsilon 1e-8

# Use AMSGrad variant for better convergence
./target/release/tuner tune --method adam --amsgrad

# Use AdaBound for stability
./target/release/tuner tune --method adam --adabound --final-lr 0.1
```

### LBFGS Tuning

#### Memory Management
```bash
# Optimize memory parameter
./target/release/tuner tune --method lbfgs --memory 20  # Higher memory for better convergence

# Use line search optimization
./target/release/tuner tune --method lbfgs --line-search wolfe --c1 1e-4 --c2 0.9

# Adjust convergence tolerance
./target/release/tuner tune --method lbfgs --tolerance 1e-6
```

### Genetic Algorithm Tuning

#### Population Management
```bash
# Optimize population size
./target/release/tuner tune --method genetic --population-size 200  # Larger for exploration

# Tune mutation and crossover rates
./target/release/tuner tune --method genetic --mutation-rate 0.1 --crossover-rate 0.8

# Use adaptive parameters
./target/release/tuner tune --method genetic --adaptive-parameters
```

## Benchmarking and Profiling

### Performance Profiling

#### CPU Profiling
```bash
# Use perf for CPU profiling
perf record -g ./target/release/tuner tune [options]
perf report

# Use flamegraph for visualization
perf record -g ./target/release/tuner tune [options]
perf script | flamegraph > profile.svg
```

#### Memory Profiling
```bash
# Use valgrind for memory profiling
valgrind --tool=massif ./target/release/tuner tune [options]

# Use heaptrack for memory analysis
heaptrack ./target/release/tuner tune [options]
heaptrack_print heaptrack.tuner.*.gz
```

### Benchmarking

#### System Benchmark
```bash
# Run system benchmark
./target/release/tuner benchmark-system

# Benchmark specific components
./target/release/tuner benchmark-data-processing --dataset games.json
./target/release/tuner benchmark-optimization --method adam --iterations 1000
```

#### Performance Comparison
```bash
# Compare different configurations
./target/release/tuner benchmark-config \
  --config baseline.json \
  --config optimized.json \
  --iterations 1000

# Benchmark memory usage
./target/release/tuner benchmark-memory \
  --dataset games.json \
  --memory-limit 8192
```

## Performance Monitoring

### Real-time Monitoring
```bash
# Monitor performance during tuning
./target/release/tuner tune --performance-monitoring --monitor-interval 10

# Use external monitoring tools
htop -p $(pgrep tuner)
iotop -p $(pgrep tuner)
nethogs -p $(pgrep tuner)
```

### Logging and Metrics
```bash
# Enable detailed performance logging
./target/release/tuner tune --performance-logging --log-metrics

# Export performance metrics
./target/release/tuner tune --export-metrics metrics.json --metrics-interval 60
```

## Best Practices Summary

### Hardware Selection
1. **CPU**: High core count (16+ cores) for parallel processing
2. **Memory**: 32GB+ for large datasets, fast memory (DDR4-3200+)
3. **Storage**: NVMe SSD for checkpoints and temporary files
4. **Network**: High-bandwidth connection for data transfer

### System Configuration
1. **OS**: Use latest stable kernel with performance optimizations
2. **Scheduler**: Use performance CPU governor
3. **Memory**: Disable swap or use fast swap on SSD
4. **I/O**: Use optimal I/O scheduler for storage type

### Application Tuning
1. **Threads**: Match thread count to workload type
2. **Memory**: Use memory pools and efficient data structures
3. **I/O**: Use async I/O and optimal buffer sizes
4. **Algorithms**: Choose algorithm based on problem characteristics

### Monitoring
1. **Profiling**: Regular performance profiling and analysis
2. **Benchmarking**: Continuous benchmarking of configurations
3. **Monitoring**: Real-time monitoring during execution
4. **Optimization**: Iterative optimization based on metrics

## Next Steps

- [User Guide](USER_GUIDE.md) for usage instructions
- [Optimization Examples](OPTIMIZATION_EXAMPLES.md) for configuration examples
- [Troubleshooting Guide](TROUBLESHOOTING_GUIDE.md) for problem resolution
