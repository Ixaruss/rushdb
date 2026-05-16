/*
 *  This file is written by ai.
 *  With sole purpose in mind to produce similarly formatted output,
 *  as the redis benchmark program for handy graph generation.
 */

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;

fn write_str(buf: &mut Vec<u8>, s: &str) {
    let b = s.as_bytes();
    buf.extend_from_slice(&(b.len() as u32).to_be_bytes());
    buf.extend_from_slice(b);
}

fn encode_set(key: &str, val: &str) -> Vec<u8> {
    let mut buf = vec![1u8];
    write_str(&mut buf, key);
    write_str(&mut buf, val);
    buf
}

fn encode_get(key: &str) -> Vec<u8> {
    let mut buf = vec![0u8];
    write_str(&mut buf, key);
    buf
}

async fn read_response(s: &mut TcpStream) -> std::io::Result<(u8, Vec<u8>)> {
    let status = s.read_u8().await?;
    let len = s.read_u32().await?;
    let mut val = vec![0u8; len as usize];
    if len > 0 {
        s.read_exact(&mut val).await?;
    }
    Ok((status, val))
}

// ── Config ────────────────────────────────────────────────────────────────────

#[derive(Clone)]
struct Config {
    host: String,
    port: u16,
    requests: usize,
    concurrency: usize,
    value_size: usize,
    threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 6080,
            requests: 1_000_000,
            concurrency: 200,
            value_size: 64,
            threads: 1,
        }
    }
}

// ── Throughput phase ──────────────────────────────────────────────────────────

struct PhaseResult {
    ops: u64,
    errors: u64,
    elapsed: Duration,
    latencies: Vec<Duration>,
}

async fn run_phase(cfg: &Config, phase: &str) -> PhaseResult {
    let addr = format!("{}:{}", cfg.host, cfg.port);
    let value = "x".repeat(cfg.value_size);
    let per_worker = cfg.requests / cfg.concurrency;
    let sem = Arc::new(Semaphore::new(cfg.concurrency));
    let ok_count = Arc::new(AtomicU64::new(0));
    let err_count = Arc::new(AtomicU64::new(0));
    let is_set = phase == "SET";

    // collect per-worker latency samples (first 5 ops per worker)
    let lat_store: Arc<tokio::sync::Mutex<Vec<Duration>>> =
        Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let start = Instant::now();
    let mut handles = Vec::with_capacity(cfg.concurrency);

    for w in 0..cfg.concurrency {
        let permit = sem.clone().acquire_owned().await.unwrap();
        let ok = ok_count.clone();
        let err = err_count.clone();
        let addr = addr.clone();
        let value = value.clone();
        let base = w * per_worker;
        let lats = lat_store.clone();

        handles.push(tokio::spawn(async move {
            let _p = permit;
            match TcpStream::connect(&addr).await {
                Err(e) => {
                    eprintln!("  [worker {w}] connect failed: {e}");
                    err.fetch_add(per_worker as u64, Ordering::Relaxed);
                }
                Ok(mut stream) => {
                    for i in 0..per_worker {
                        let key = format!("bench:{}", base + i);
                        let req = if is_set {
                            encode_set(&key, &value)
                        } else {
                            encode_get(&key)
                        };
                        let t = Instant::now();
                        let success = stream.write_all(&req).await.is_ok()
                            && matches!(read_response(&mut stream).await, Ok((0, _)));
                        let elapsed = t.elapsed();

                        if success {
                            ok.fetch_add(1, Ordering::Relaxed);
                            // sample first 50 ops per worker for latency
                            if i < 50 {
                                lats.lock().await.push(elapsed);
                            }
                        } else {
                            err.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let elapsed = start.elapsed();
    let latencies = lat_store.lock().await.clone();

    PhaseResult {
        ops: ok_count.load(Ordering::Relaxed),
        errors: err_count.load(Ordering::Relaxed),
        elapsed,
        latencies,
    }
}

// ── Latency phase ─────────────────────────────────────────────────────────────

async fn run_latency(cfg: &Config, n: usize) -> Vec<Duration> {
    let addr = format!("{}:{}", cfg.host, cfg.port);
    let value = "x".repeat(cfg.value_size);
    let mut lats = Vec::with_capacity(n * 2);

    let mut stream = match TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("  Latency: connect failed: {e}");
            return lats;
        }
    };

    for i in 0..n {
        let key = format!("lat:{i}");

        let t = Instant::now();
        if stream.write_all(&encode_set(&key, &value)).await.is_err() {
            break;
        }
        if read_response(&mut stream).await.is_err() {
            break;
        }
        lats.push(t.elapsed());

        let t = Instant::now();
        if stream.write_all(&encode_get(&key)).await.is_err() {
            break;
        }
        if read_response(&mut stream).await.is_err() {
            break;
        }
        lats.push(t.elapsed());
    }
    lats
}

// ── Stats helpers ─────────────────────────────────────────────────────────────

fn pct(sorted: &[Duration], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() as f64 * p / 100.0).ceil() as usize)
        .saturating_sub(1)
        .min(sorted.len() - 1);
    sorted[i].as_nanos() as f64 / 1_000_000.0 // return ms like redis
}

fn avg_ms(lats: &[Duration]) -> f64 {
    if lats.is_empty() {
        return 0.0;
    }
    let total: u128 = lats.iter().map(|d| d.as_nanos()).sum();
    total as f64 / lats.len() as f64 / 1_000_000.0
}

fn min_ms(lats: &[Duration]) -> f64 {
    lats.iter().map(|d| d.as_nanos()).min().unwrap_or(0) as f64 / 1_000_000.0
}

fn max_ms(lats: &[Duration]) -> f64 {
    lats.iter().map(|d| d.as_nanos()).max().unwrap_or(0) as f64 / 1_000_000.0
}

// ── Redis-style printer ───────────────────────────────────────────────────────

fn print_redis_style(label: &str, result: &PhaseResult, cfg: &Config) {
    let secs = result.elapsed.as_secs_f64();
    let ops_sec = result.ops as f64 / secs;

    let mut lats = result.latencies.clone();
    lats.sort();

    println!("====== {} ======", label);
    println!("  {} requests completed in {:.2} seconds", result.ops, secs);
    println!("  {} parallel clients", cfg.concurrency);
    println!("  {} bytes payload", cfg.value_size);
    println!("  errors: {}", result.errors);
    println!(
        "  multi-thread: {}",
        if cfg.threads > 1 { "yes" } else { "no" }
    );
    if cfg.threads > 1 {
        println!("  threads: {}", cfg.threads);
    }
    println!();
    println!("Summary:");
    println!("  throughput summary: {:.2} requests per second", ops_sec);
    println!("  latency summary (msec):");
    println!("          avg       min       p50       p95       p99       max");
    println!(
        "        {:<9.3} {:<9.3} {:<9.3} {:<9.3} {:<9.3} {:.3}",
        avg_ms(&lats),
        min_ms(&lats),
        pct(&lats, 50.0),
        pct(&lats, 95.0),
        pct(&lats, 99.0),
        max_ms(&lats),
    );
    println!();
}

// ── main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut cfg = Config::default();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--host" => {
                cfg.host = args[i + 1].clone();
                i += 2;
            }
            "--port" => {
                cfg.port = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--requests" => {
                cfg.requests = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--concurrency" => {
                cfg.concurrency = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--value-size" => {
                cfg.value_size = args[i + 1].parse().unwrap();
                i += 2;
            }
            "--threads" => {
                cfg.threads = args[i + 1].parse().unwrap();
                i += 2;
            }
            _ => {
                i += 1;
            }
        }
    }
    cfg.requests = (cfg.requests / cfg.concurrency) * cfg.concurrency;

    println!("╔══════════════════════════════════════════════════════╗");
    println!("║            RushDB Benchmark  v0.3                   ║");
    println!("╚══════════════════════════════════════════════════════╝");
    println!("  server      : {}:{}", cfg.host, cfg.port);
    println!("  requests    : {}", cfg.requests);
    println!("  concurrency : {} clients", cfg.concurrency);
    println!("  value size  : {} bytes", cfg.value_size);
    println!();

    // SET
    let set_result = run_phase(&cfg, "SET").await;
    print_redis_style("SET", &set_result, &cfg);

    // GET
    let get_result = run_phase(&cfg, "GET").await;
    print_redis_style("GET", &get_result, &cfg);

    // Latency — dedicated sequential phase
    println!("====== LATENCY (sequential, single conn, 1000 SET+GET pairs) ======");
    let mut lats = run_latency(&cfg, 1000).await;
    if lats.is_empty() {
        println!("  (no data — is the server running on port {}?)", cfg.port);
    } else {
        lats.sort();
        println!("  samples: {}", lats.len());
        println!("  latency summary (msec):");
        println!("          avg       min       p50       p95       p99       max");
        println!(
            "        {:<9.3} {:<9.3} {:<9.3} {:<9.3} {:<9.3} {:.3}",
            avg_ms(&lats),
            min_ms(&lats),
            pct(&lats, 50.0),
            pct(&lats, 95.0),
            pct(&lats, 99.0),
            max_ms(&lats),
        );
    }
    println!();
    println!("Done.");
}
