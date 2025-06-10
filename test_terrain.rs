#!/usr/bin/env rust-script
//! Quick test to verify terrain generation improvements

use std::f32::consts::PI;

// Simplified versions of the biome height functions to test
fn test_terrain_heights() {
    println!("Testing terrain height ranges:");
    
    // Test a few sample points
    let test_points = vec![
        (0.0, 0.0),
        (100.0, 100.0),
        (500.0, 500.0),
        (1000.0, 1000.0),
        (-500.0, -500.0),
    ];
    
    for (x, z) in test_points {
        let canyon_h = canyon_height(x, z);
        let crystal_h = crystalline_height(x, z);
        let mountain_h = mountain_height(x, z);
        
        println!("\nAt position ({}, {}):", x, z);
        println!("  Canyon: {:.2}", canyon_h);
        println!("  Crystal: {:.2}", crystal_h);
        println!("  Mountain: {:.2}", mountain_h);
    }
}

// Simplified canyon height function
fn canyon_height(x: f32, z: f32) -> f32 {
    let scale1 = 0.003 * 1.5;
    let canyon_main = (x * scale1).sin() + (z * scale1 * 0.7).cos();
    let depth = canyon_main.abs().powf(0.8);
    -150.0 + (1.0 - depth) * 180.0
}

// Simplified crystalline height function
fn crystalline_height(x: f32, z: f32) -> f32 {
    let scale1 = 0.01 * 3.0;
    let spike = ((x * scale1).sin() * (z * scale1).cos()).abs().powf(0.5) * 40.0;
    spike.clamp(-30.0, 120.0)
}

// Simplified mountain height function
fn mountain_height(x: f32, z: f32) -> f32 {
    let scale1 = 0.001 * 0.8;
    let peak = ((x * scale1).sin().powi(2) + (z * scale1).cos().powi(2)).sqrt();
    let h = (1.0 - peak).max(0.0).powf(1.8) * 80.0;
    h.clamp(-50.0, 250.0)
}

fn main() {
    test_terrain_heights();
}