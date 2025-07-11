use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::time::Duration;

// 导入我们的SMA实现
#[derive(Debug, Clone)]
pub struct StackSMA<const N: usize> {
    index: usize,
    count: usize,
    sum: f64,
    data: [f64; N],
}

impl<const N: usize> StackSMA<N> {
    pub fn new() -> Self {
        Self {
            index: 0,
            count: 0,
            sum: 0.0,
            data: [0.0; N],
        }
    }
    
    pub fn next(&mut self, input: f64) -> f64 {
        let old_val = self.data[self.index];
        self.data[self.index] = input;
        self.index = if self.index + 1 < N { self.index + 1 } else { 0 };
        if self.count < N { self.count += 1; }
        self.sum = self.sum - old_val + input;
        self.sum / (self.count as f64)
    }
}

#[derive(Debug, Clone)]
pub struct HeapSMA {
    period: usize,
    index: usize,
    count: usize,
    sum: f64,
    data: Box<[f64]>,
}

impl HeapSMA {
    pub fn new(period: usize) -> Self {
        Self {
            period,
            index: 0,
            count: 0,
            sum: 0.0,
            data: vec![0.0; period].into_boxed_slice(),
        }
    }
    
    pub fn next(&mut self, input: f64) -> f64 {
        let old_val = self.data[self.index];
        self.data[self.index] = input;
        self.index = if self.index + 1 < self.period { self.index + 1 } else { 0 };
        if self.count < self.period { self.count += 1; }
        self.sum = self.sum - old_val + input;
        self.sum / (self.count as f64)
    }
}

// 基准测试：创建性能
fn bench_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA Creation");
    
    // 栈分配创建
    group.bench_function("Stack SMA<20>", |b| {
        b.iter(|| {
            let sma = StackSMA::<20>::new();
            black_box(sma);
        })
    });
    
    // 堆分配创建
    group.bench_function("Heap SMA(20)", |b| {
        b.iter(|| {
            let sma = HeapSMA::new(20);
            black_box(sma);
        })
    });
    
    group.finish();
}

// 基准测试：计算性能
fn bench_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA Computation");
    
    // 不同数据量的测试
    for size in [100, 1000, 10000, 100000].iter() {
        let data: Vec<f64> = (0..*size).map(|i| i as f64).collect();
        
        group.bench_with_input(BenchmarkId::new("Stack", size), size, |b, _| {
            b.iter(|| {
                let mut sma = StackSMA::<20>::new();
                for &value in &data {
                    black_box(sma.next(value));
                }
            })
        });
        
        group.bench_with_input(BenchmarkId::new("Heap", size), size, |b, _| {
            b.iter(|| {
                let mut sma = HeapSMA::new(20);
                for &value in &data {
                    black_box(sma.next(value));
                }
            })
        });
    }
    
    group.finish();
}

// 基准测试：不同窗口大小
fn bench_window_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("SMA Window Sizes");
    
    let data: Vec<f64> = (0..10000).map(|i| i as f64).collect();
    
    for window_size in [5, 10, 20, 50, 100].iter() {
        // 堆分配（所有窗口大小都支持）
        group.bench_with_input(BenchmarkId::new("Heap", window_size), window_size, |b, &size| {
            b.iter(|| {
                let mut sma = HeapSMA::new(size);
                for &value in &data {
                    black_box(sma.next(value));
                }
            })
        });
        
        // 栈分配（只测试小窗口）
        if *window_size <= 50 {
            group.bench_with_input(BenchmarkId::new("Stack", window_size), window_size, |b, &size| {
                b.iter(|| {
                    match size {
                        5 => {
                            let mut sma = StackSMA::<5>::new();
                            for &value in &data {
                                black_box(sma.next(value));
                            }
                        },
                        10 => {
                            let mut sma = StackSMA::<10>::new();
                            for &value in &data {
                                black_box(sma.next(value));
                            }
                        },
                        20 => {
                            let mut sma = StackSMA::<20>::new();
                            for &value in &data {
                                black_box(sma.next(value));
                            }
                        },
                        50 => {
                            let mut sma = StackSMA::<50>::new();
                            for &value in &data {
                                black_box(sma.next(value));
                            }
                        },
                        _ => {}
                    }
                })
            });
        }
    }
    
    group.finish();
}

// 基准测试：内存访问模式
fn bench_memory_pattern(c: &mut Criterion) {
    let mut group = c.benchmark_group("Memory Access Pattern");
    
    // 顺序访问
    group.bench_function("Stack Sequential", |b| {
        let data: Vec<f64> = (0..10000).map(|i| i as f64).collect();
        b.iter(|| {
            let mut sma = StackSMA::<20>::new();
            for &value in &data {
                black_box(sma.next(value));
            }
        })
    });
    
    group.bench_function("Heap Sequential", |b| {
        let data: Vec<f64> = (0..10000).map(|i| i as f64).collect();
        b.iter(|| {
            let mut sma = HeapSMA::new(20);
            for &value in &data {
                black_box(sma.next(value));
            }
        })
    });
    
    // 随机访问（模拟实际交易数据的不规律性）
    group.bench_function("Stack Random", |b| {
        let data: Vec<f64> = (0..10000).map(|i| (i as f64 * 1.618) % 1000.0).collect();
        b.iter(|| {
            let mut sma = StackSMA::<20>::new();
            for &value in &data {
                black_box(sma.next(value));
            }
        })
    });
    
    group.bench_function("Heap Random", |b| {
        let data: Vec<f64> = (0..10000).map(|i| (i as f64 * 1.618) % 1000.0).collect();
        b.iter(|| {
            let mut sma = HeapSMA::new(20);
            for &value in &data {
                black_box(sma.next(value));
            }
        })
    });
    
    group.finish();
}

// 基准测试：批量创建（模拟多股票监控）
fn bench_batch_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Batch Creation");
    
    group.bench_function("Stack Batch 1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let sma = StackSMA::<20>::new();
                black_box(sma);
            }
        })
    });
    
    group.bench_function("Heap Batch 1000", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                let sma = HeapSMA::new(20);
                black_box(sma);
            }
        })
    });
    
    group.finish();
}

// 配置criterion
fn configure_criterion() -> Criterion {
    Criterion::default()
        .measurement_time(Duration::from_secs(10))  // 每个测试运行10秒
        .sample_size(100)                           // 采样100次
        .warm_up_time(Duration::from_secs(2))       // 预热2秒
        .with_plots()                               // 生成图表
}

criterion_group!(
    name = benches;
    config = configure_criterion();
    targets = bench_creation, bench_computation, bench_window_sizes, bench_memory_pattern, bench_batch_creation
);
criterion_main!(benches); 