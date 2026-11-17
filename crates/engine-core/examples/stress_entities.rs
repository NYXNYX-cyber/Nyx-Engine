use engine_core::{Player, PlayerInput, WorldState};
use std::collections::HashMap;
use std::time::Instant;

fn main() {
    println!("==========================================================================");
    println!("       NYX ENGINE — CORE ENTITY SIMULATION THROUGHPUT BENCHMARK          ");
    println!("==========================================================================");
    println!("Hardware: Intel Core i5-3470 (4 Cores @ 3.20GHz) | 16GB DDR3 | Debian 13\n");

    let counts = vec![1_000, 5_000, 10_000, 25_000, 50_000, 100_000, 250_000];

    println!("| Active Entities | Tick Time (ms) | Tick Time (µs) | Entities/sec (Throughput) | Frame Margin (60Hz) |");
    println!("|----------------:|---------------:|---------------:|--------------------------:|--------------------:|");

    for &count in &counts {
        let mut world = WorldState::new(1920.0, 1080.0);
        let mut inputs = HashMap::with_capacity(count);

        for i in 0..count {
            world.players.insert(
                i as u32,
                Player::new(i as u32, format!("Player_{}", i), "#00f0ff".into()),
            );
            inputs.insert(
                i as u32,
                PlayerInput {
                    up: i % 2 == 0,
                    down: i % 3 == 0,
                    left: i % 4 == 0,
                    right: i % 5 == 0,
                },
            );
        }

        // Warm up 10 ticks
        for _ in 0..10 {
            world.step(&inputs, 1.0 / 60.0);
        }

        // Measure 100 ticks
        let start = Instant::now();
        for _ in 0..100 {
            world.step(&inputs, 1.0 / 60.0);
        }
        let elapsed = start.elapsed();
        let avg_tick_secs = elapsed.as_secs_f64() / 100.0;
        let avg_tick_ms = avg_tick_secs * 1000.0;
        let avg_tick_us = avg_tick_secs * 1_000_000.0;
        let entities_per_sec = (count as f64) / avg_tick_secs;
        let frame_budget_margin = ((16.666 - avg_tick_ms) / 16.666) * 100.0;

        println!(
            "| {:>15} | {:>13.4}ms | {:>13.1}µs | {:>24.0} | {:>18.1}% |",
            count, avg_tick_ms, avg_tick_us, entities_per_sec, frame_budget_margin
        );
    }
    println!("==========================================================================");
}
