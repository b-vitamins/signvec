# SignVec
![Crates.io](https://img.shields.io/crates/v/signvec)
![docs.rs](https://img.shields.io/docsrs/signvec)
![License](https://img.shields.io/crates/l/signvec)
![GitHub Workflow Status](https://github.com/b-vitamins/signvec/actions/workflows/rust.yml/badge.svg)

`SignVec` extends the capabilities of the traditional `Vec` by providing additional functionalities to efficiently track and manipulate elements based on their sign (positive or negative) using the `Signable` trait.

## Features
- Tracks the sign of elements for optimized sign-specific operations.
- Provides methods for element counting, access, and manipulation based on sign.
- Integrates with user-defined types via the `Signable` trait.

## Usage: Basic operations

```rust
use nanorand::{Rng, WyRand};
use signvec::{svec, Sign, SignVec};

fn main() {
    let mut vector = svec![1, -2, 3, -4];
    assert_eq!(vector.len(), 4);
    assert_eq!(vector[0], 1);
    assert_eq!(vector[1], -2);

    // Count positive and negative elements
    assert_eq!(vector.count(Sign::Plus), 2);
    assert_eq!(vector.count(Sign::Minus), 2);

    // Get indices of positive and negative elements
    assert_eq!(
        vector.indices(Sign::Plus).iter().collect::<Vec<_>>(),
        vec![&0, &2]
    );
    assert_eq!(
        vector.indices(Sign::Minus).iter().collect::<Vec<_>>(),
        vec![&1, &3]
    );

    // Retrieve values based on their sign
    assert_eq!(vector.values(Sign::Plus).collect::<Vec<_>>(), vec![&1, &3]);
    assert_eq!(
        vector.values(Sign::Minus).collect::<Vec<_>>(),
        vec![&-2, &-4]
    );

    // Modify individual elements
    vector.set(1, 5);
    assert_eq!(vector[1], 5);

    // Randomly select an element based on its sign
    let mut rng = WyRand::new();
    if let Some(random_positive) = vector.random(Sign::Plus, &mut rng) {
        println!("Random positive value: {}", random_positive);
    }
}
```

## Usage: Monte Carlo simulations

This demonstrates a simple Monte Carlo simulation where site energies in a `SignVec` are updated based on simulated dynamics and system energy distribution.

```rust
use signvec::{SignVec, svec, Sign, Signable};
use nanorand::{WyRand, Rng};

fn main() {
    let mut energies = svec![1.0, -1.0, 1.5, -1.5, 0.5, -0.5];
    let mut rng = WyRand::new();

    // Simulation loop for multiple Monte Carlo steps
    for _step in 0..100 {
        let site = rng.generate_range(0..energies.len());
        let dE = rng.generate::<f64>() - 0.5; // Change in energy

        let new_energy = energies[site] + dE; // Update site energy
        
        // Make decisions based on system's energy distribution
        if energies.count(Sign::Minus) > energies.count(Sign::Plus) {
            if energies[site].sign() == Sign::Minus && rng.generate::<f64>() < 0.5 {
                energies.set(site, -new_energy); // Flip energy sign
            } else {
                energies.set(site, new_energy);
            }
        } else {
            energies.set(site, new_energy); // Balanced distribution
        }
    }

    println!("Final energy distribution: {:?}", energies);
}
```

## Usage: Portfolio management

Demonstrates how `SignVec` can be used for managing a financial portfolio, simulating market conditions, and making decisions based on the sign-aware characteristics of assets and liabilities.

```rust
use signvec::{SignVec, Sign, svec};
use nanorand::WyRand;

fn main() {
    let mut portfolio = svec![150.0, -200.0, 300.0, -50.0, 400.0];
    let market_conditions = vec![1.05, 0.95, 1.10, 1.00, 1.03];

    // Apply market conditions to adjust portfolio balances
    for (index, &factor) in market_conditions.iter().enumerate() {
        portfolio.set(index, portfolio[index] * factor);
    }

    // Decision making based on portfolio's sign-aware characteristics
    if portfolio.count(Sign::Minus) > 2 {
        println!("Consider rebalancing your portfolio to manage risk.");
    } else {
        println!("Your portfolio is well-balanced and diversified.");
    }

    // Calculate 10% of total liabilities for debt reduction
    let debt_reduction = portfolio.values(Sign::Minus).sum::<f64>() * 0.1;
    println!("Plan for a debt reduction of ${:.2} to strengthen your financial position.", debt_reduction.abs());

    // Identify a high-performing asset for potential investment
    let mut rng = WyRand::new();
    if let Some(lucky_asset) = portfolio.random(Sign::Plus, &mut rng) {
        println!("Consider investing more in an asset valued at ${:.2}.", lucky_asset);
    } else {
        println!("No standout assets for additional investment at the moment.");
    }
}
```

## Benchmarks

The table below is a summary of benchmark results for the specialized functionality of `SignVec`. The times are averaged between operations on positive and negative elements.

| Operation          | `SignVec`    | `Vec`          | Speedup         |
|--------------------|--------------|----------------|-----------------|
| `set`              | 3.85 ns      | -              | -               |
| `count`            | 225.74 ps    | 153.38 ns      | ~679x faster    |
| `random`           | 2.01 ns      | 1.12 µs        | ~556x faster    |
| `indices`          | 86.42 ns     | 1.11 µs        | ~12.8x faster   |
| `values`           | 579.37 ns    | 1.13 µs        | ~1.95x faster   |

Benchmarks were conducted on a machine with the following specifications:
- Processor: AMD Ryzen™ 5 5600G with Radeon™ Graphics x 12
- Memory: 58.8 GiB
- Operating System: Guix System
- OS Type: 64-bit
