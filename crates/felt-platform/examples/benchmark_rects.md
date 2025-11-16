# BENCHMARK RECTS Performance Results

Results from running the 200,000 rectangle benchmark on various hardware configurations.

## How to Run

```bash
cargo run --package felt-platform --example benchmark_rects --release
```

Close the window to see final benchmark statistics. The benchmark runs continuously and prints FPS updates every 60 frames.

## Test Environment

### Windows PC (5090)
- **GPU**: NVIDIA GeForce RTX 5090
- **Monitor**: 240Hz
- **OS**: Windows
- **Build**: Release mode
- **VSync**: Enabled (PresentMode::Fifo)

## Results

### Simple Rectangles (3 shapes)
| Metric | Value |
|--------|-------|
| FPS | 240 |
| Frame Time | ~4.17ms |
| Status | ✓ VSync-limited at monitor refresh rate |

### Benchmark: 200,000 Rectangles
| Metric | Value |
|--------|-------|
| Rectangle Count | 200,000 |
| Grid Dimensions | ~447×447 |
| Rectangle Size | ~1.8px × 1.3px |
| Average FPS | 92.6 |
| Average Frame Time | 10.83ms |
| Min Frame Time | 10.73ms |
| Max Frame Time | 10.83ms |
| Throughput | ~18.4M rectangles/second |
| Total Frames Tested | 1,393 |
| Status | ✓ GPU-bound performance |

## Interpretation

- **Simple scenes**: System easily hits monitor refresh rate (240 FPS)
- **Complex scenes**: 200k primitives sustained at 92 FPS demonstrates strong GPU throughput
- **Frame consistency**: Low variance (10.73-10.83ms) shows stable performance
- **Real-world performance**: Typical UI apps (few thousand shapes) will run at full refresh rate

## macOS Results

### macOS M4 Max
- **GPU**: Apple M4 Max
- **Monitor**: Pro Display XDR (60Hz)
- **OS**: macOS
- **Build**: Release mode
- **VSync**: Enabled (PresentMode::Fifo)

#### Benchmark: 200,000 Rectangles
| Metric | Value |
|--------|-------|
| Rectangle Count | 200,000 |
| Grid Dimensions | ~447×447 |
| Rectangle Size | ~1.8px × 1.3px |
| Average FPS | 60.3 |
| Average Frame Time | 16.40ms |
| Min Frame Time | 6.65ms |
| Total Frames Tested | 1,666 |
| Throughput | ~12M rectangles/second |
| Status | ✓ VSync-limited at monitor refresh rate |

**Analysis**: M4 Max maintains full 60 FPS with 200k rectangles. Min frame time of 6.65ms (~150 FPS potential) indicates GPU has headroom and is vsync-capped by the 60Hz display, not GPU-bound.

## Linux Results

### Linux Wayland (RTX 5090)
- **GPU**: NVIDIA GeForce RTX 5090
- **Monitor**: 240Hz (same as Windows test)
- **OS**: Linux with Wayland compositor
- **Build**: Release mode
- **VSync**: Enabled (PresentMode::Fifo)
- **Note**: Same hardware as Windows test (dual-boot)

#### Benchmark: 200,000 Rectangles
| Metric | Value |
|--------|-------|
| Rectangle Count | 200,000 |
| Grid Dimensions | ~447×447 |
| Rectangle Size | ~1.8px × 1.3px |
| Average FPS | 150.6 |
| Average Frame Time | 6.65ms |
| Max Frame Time | 10.17ms |
| Total Frames Tested | 1,241 |
| Throughput | ~30.1M rectangles/second |
| Status | ✓ GPU-bound performance |

**Analysis**: **63% faster than Windows on identical hardware**. Linux/Wayland shows dramatically lower overhead. The 6.65ms average matches M4 Max's minimum, suggesting this is close to the actual GPU throughput limit for this workload.

## Cross-Platform Analysis

### 200k Rectangle Benchmark Comparison

| Platform | GPU | FPS | Frame Time | Throughput | Bottleneck |
|----------|-----|-----|------------|------------|------------|
| Linux/Wayland | RTX 5090 | 150.6 | 6.65ms | 30.1M rects/s | GPU-bound |
| Windows | RTX 5090 | 92.6 | 10.83ms | 18.4M rects/s | OS overhead |
| macOS | M4 Max | 60.3 | 6.65ms min | 12M rects/s | VSync @ 60Hz |

### Key Findings

1. **Platform overhead matters**: Same RTX 5090 shows 63% performance difference between Linux and Windows
2. **M4 Max competitive**: 6.65ms minimum frame time matches Linux/5090 average, suggesting similar GPU throughput
3. **Windows DWM overhead**: ~4ms additional latency per frame compared to Linux/Wayland
4. **VSync working correctly**: All platforms properly sync to display refresh rate when not GPU-bound
