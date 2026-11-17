use engine_core::physics::PhysicsWorld;
use std::time::Instant;

fn main() {
    println!("==========================================================================");
    println!("      NYX ENGINE — RAPIER3D PHYSICS STRESS TEST & BENCHMARK HARNESS      ");
    println!("==========================================================================");
    println!("Hardware: Intel Core i5-3470 (4 Cores @ 3.20GHz) | 16GB DDR3 | Debian 13");
    println!("Frame Budget: 16.666 ms (60.0 FPS fixed tick rate)");
    println!("Simulating 120 ticks per batch (2.0 seconds of continuous 3D collision)\n");

    let entity_counts = vec![100, 250, 500, 1000, 2000, 3500, 5000, 7500];

    println!("| Entity Count | Min Tick | Avg Tick | Max Tick | P95 Tick | P99 Tick | Budget Util | Status (<16.6ms) |");
    println!("|-------------:|---------:|---------:|---------:|---------:|---------:|------------:|:----------------:|");

    for &count in &entity_counts {
        let mut world = PhysicsWorld::default();

        // Add static floor (100x2x100m)
        world.add_static_box(100.0, 1.0, 100.0, 0.0, -1.0, 0.0);
        // Add 4 static boundary walls
        world.add_static_box(1.0, 30.0, 100.0, -100.0, 30.0, 0.0);
        world.add_static_box(1.0, 30.0, 100.0, 100.0, 30.0, 0.0);
        world.add_static_box(100.0, 30.0, 1.0, 0.0, 30.0, -100.0);
        world.add_static_box(100.0, 30.0, 1.0, 0.0, 30.0, 100.0);

        // Spawn dynamic spheres in a layered grid
        let per_dim = (count as f32).cbrt().ceil() as usize;
        let mut spawned = 0;
        'spawn: for x in 0..per_dim {
            for y in 0..per_dim {
                for z in 0..per_dim {
                    if spawned >= count {
                        break 'spawn;
                    }
                    let px = (x as f32 - (per_dim as f32 / 2.0)) * 1.5;
                    let py = 5.0 + (y as f32) * 1.8;
                    let pz = (z as f32 - (per_dim as f32 / 2.0)) * 1.5;
                    world.add_dynamic_sphere(0.4, px, py, pz);
                    spawned += 1;
                }
            }
        }

        // Warm up 10 ticks
        for _ in 0..10 {
            world.step();
        }

        // Measure 120 ticks
        let mut durations = Vec::with_capacity(120);
        for _ in 0..120 {
            let start = Instant::now();
            world.step();
            durations.push(start.elapsed());
        }

        durations.sort();

        let sum: std::time::Duration = durations.iter().sum();
        let avg = sum.as_secs_f64() * 1000.0 / durations.len() as f64;
        let min = durations.first().unwrap().as_secs_f64() * 1000.0;
        let max = durations.last().unwrap().as_secs_f64() * 1000.0;
        let p95_idx = ((durations.len() as f64) * 0.95) as usize;
        let p99_idx = ((durations.len() as f64) * 0.99) as usize;
        let p95 = durations[p95_idx].as_secs_f64() * 1000.0;
        let p99 = durations[p99_idx].as_secs_f64() * 1000.0;
        let budget_pct = (avg / 16.666) * 100.0;
        let status = if avg <= 16.666 { "✔ PASS (60 FPS)" } else { "⚠ OVER BUDGET" };

        println!(
            "| {:>12} | {:>7.2}ms | {:>7.2}ms | {:>7.2}ms | {:>7.2}ms | {:>7.2}ms | {:>10.1}% | {:^16} |",
            count, min, avg, max, p95, p99, budget_pct, status
        );
    }
    println!("==========================================================================");
}
