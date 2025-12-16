//! ECS Benchmark Runner
//!
//! Provides runtime benchmarking and comparison tools for different ECS backends.

use std::time::{Duration, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

use crate::backends::{EcsBackendType, DynamicWorld};
use crate::traits::EcsWorld;

/// Benchmark results for a specific test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub backend: EcsBackendType,
    pub test_name: String,
    pub duration: Duration,
    pub operations_per_second: f64,
    pub memory_usage_estimate: Option<usize>,
}

/// Collection of benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub results: Vec<BenchmarkResult>,
    pub timestamp: String,
}

impl BenchmarkSuite {
    pub fn new() -> Self {
        Self {
            results: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
    
    /// Add a benchmark result
    pub fn add_result(&mut self, result: BenchmarkResult) {
        self.results.push(result);
    }
    
    /// Get results for a specific backend
    pub fn get_backend_results(&self, backend: EcsBackendType) -> Vec<&BenchmarkResult> {
        self.results.iter()
            .filter(|r| r.backend == backend)
            .collect()
    }
    
    /// Get results for a specific test
    pub fn get_test_results(&self, test_name: &str) -> Vec<&BenchmarkResult> {
        self.results.iter()
            .filter(|r| r.test_name == test_name)
            .collect()
    }
    
    /// Generate a comparison report
    pub fn generate_report(&self) -> String {
        let mut report = String::new();
        report.push_str("ECS Backend Benchmark Report\n");
        report.push_str("============================\n\n");
        
        // Group results by test name
        let mut tests: HashMap<String, Vec<&BenchmarkResult>> = HashMap::new();
        for result in &self.results {
            tests.entry(result.test_name.clone())
                .or_default()
                .push(result);
        }
        
        for (test_name, results) in tests {
            report.push_str(&format!("Test: {}\n", test_name));
            report.push_str("-".repeat(test_name.len() + 6).as_str());
            report.push_str("\n");
            
            // Sort by performance (operations per second)
            let mut sorted_results = results;
            sorted_results.sort_by(|a, b| b.operations_per_second.partial_cmp(&a.operations_per_second).unwrap());
            
            for (i, result) in sorted_results.iter().enumerate() {
                let rank = match i {
                    0 => "🥇",
                    1 => "🥈", 
                    2 => "🥉",
                    _ => "  ",
                };
                
                report.push_str(&format!(
                    "{} {:?}: {:.2} ops/sec ({:.2}ms)\n",
                    rank,
                    result.backend,
                    result.operations_per_second,
                    result.duration.as_secs_f64() * 1000.0
                ));
            }
            report.push_str("\n");
        }
        
        report
    }
    
    /// Save results to JSON file
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
    
    /// Load results from JSON file
    pub fn load_from_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(path)?;
        let suite = serde_json::from_str(&json)?;
        Ok(suite)
    }
}

/// Benchmark runner for ECS backends
pub struct BenchmarkRunner {
    pub iterations: usize,
    pub warmup_iterations: usize,
}

impl Default for BenchmarkRunner {
    fn default() -> Self {
        Self {
            iterations: 1000,
            warmup_iterations: 100,
        }
    }
}

impl BenchmarkRunner {
    pub fn new(iterations: usize, warmup_iterations: usize) -> Self {
        Self {
            iterations,
            warmup_iterations,
        }
    }
    
    /// Run all benchmarks for all available backends
    pub fn run_all_benchmarks(&self) -> BenchmarkSuite {
        let mut suite = BenchmarkSuite::new();
        
        for backend_type in EcsBackendType::available_backends() {
            println!("Benchmarking {:?} backend...", backend_type);
            
            // Entity spawn benchmark
            if let Ok(result) = self.bench_entity_spawn(backend_type, 1000) {
                suite.add_result(result);
            }
            
            // Entity despawn benchmark
            if let Ok(result) = self.bench_entity_despawn(backend_type, 1000) {
                suite.add_result(result);
            }
            
            // World clear benchmark
            if let Ok(result) = self.bench_world_clear(backend_type, 1000) {
                suite.add_result(result);
            }
            
            // Hierarchy operations benchmark
            if let Ok(result) = self.bench_hierarchy_operations(backend_type, 100) {
                suite.add_result(result);
            }
            
            // Mixed operations benchmark
            if let Ok(result) = self.bench_mixed_operations(backend_type) {
                suite.add_result(result);
            }
        }
        
        suite
    }
    
    /// Benchmark entity spawning
    pub fn bench_entity_spawn(&self, backend_type: EcsBackendType, entity_count: usize) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut world = DynamicWorld::new(backend_type)?;
        
        // Warmup
        for _ in 0..self.warmup_iterations {
            world.clear();
            for _ in 0..entity_count {
                world.spawn();
            }
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            world.clear();
            for _ in 0..entity_count {
                world.spawn();
            }
        }
        let duration = start.elapsed();
        
        let total_operations = self.iterations * entity_count;
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            backend: backend_type,
            test_name: format!("entity_spawn_{}", entity_count),
            duration,
            operations_per_second: ops_per_second,
            memory_usage_estimate: None,
        })
    }
    
    /// Benchmark entity despawning
    pub fn bench_entity_despawn(&self, backend_type: EcsBackendType, entity_count: usize) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut world = DynamicWorld::new(backend_type)?;
        
        // Warmup
        for _ in 0..self.warmup_iterations {
            world.clear();
            let entities: Vec<_> = (0..entity_count).map(|_| world.spawn()).collect();
            for entity in entities {
                let _ = world.despawn(entity);
            }
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            world.clear();
            let entities: Vec<_> = (0..entity_count).map(|_| world.spawn()).collect();
            for entity in entities {
                let _ = world.despawn(entity);
            }
        }
        let duration = start.elapsed();
        
        let total_operations = self.iterations * entity_count;
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            backend: backend_type,
            test_name: format!("entity_despawn_{}", entity_count),
            duration,
            operations_per_second: ops_per_second,
            memory_usage_estimate: None,
        })
    }
    
    /// Benchmark world clearing
    pub fn bench_world_clear(&self, backend_type: EcsBackendType, entity_count: usize) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut world = DynamicWorld::new(backend_type)?;
        
        // Warmup
        for _ in 0..self.warmup_iterations {
            for _ in 0..entity_count {
                world.spawn();
            }
            world.clear();
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            for _ in 0..entity_count {
                world.spawn();
            }
            world.clear();
        }
        let duration = start.elapsed();
        
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            backend: backend_type,
            test_name: format!("world_clear_{}", entity_count),
            duration,
            operations_per_second: ops_per_second,
            memory_usage_estimate: None,
        })
    }
    
    /// Benchmark hierarchy operations
    pub fn bench_hierarchy_operations(&self, backend_type: EcsBackendType, child_count: usize) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut world = DynamicWorld::new(backend_type)?;
        
        // Warmup
        for _ in 0..self.warmup_iterations {
            world.clear();
            let parent = world.spawn();
            let children: Vec<_> = (0..child_count).map(|_| world.spawn()).collect();
            for child in children {
                let _ = world.set_parent(child, Some(parent));
            }
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            world.clear();
            let parent = world.spawn();
            let children: Vec<_> = (0..child_count).map(|_| world.spawn()).collect();
            for child in children {
                let _ = world.set_parent(child, Some(parent));
            }
            let _ = world.get_children(parent);
        }
        let duration = start.elapsed();
        
        let total_operations = self.iterations * (child_count + 1); // +1 for get_children
        let ops_per_second = total_operations as f64 / duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            backend: backend_type,
            test_name: format!("hierarchy_operations_{}", child_count),
            duration,
            operations_per_second: ops_per_second,
            memory_usage_estimate: None,
        })
    }
    
    /// Benchmark mixed operations (simulating a game frame)
    pub fn bench_mixed_operations(&self, backend_type: EcsBackendType) -> Result<BenchmarkResult, Box<dyn std::error::Error>> {
        let mut world = DynamicWorld::new(backend_type)?;
        
        // Warmup
        for _ in 0..self.warmup_iterations {
            self.simulate_game_frame(&mut world);
        }
        
        // Actual benchmark
        let start = Instant::now();
        for _ in 0..self.iterations {
            self.simulate_game_frame(&mut world);
        }
        let duration = start.elapsed();
        
        let ops_per_second = self.iterations as f64 / duration.as_secs_f64();
        
        Ok(BenchmarkResult {
            backend: backend_type,
            test_name: "mixed_operations".to_string(),
            duration,
            operations_per_second: ops_per_second,
            memory_usage_estimate: None,
        })
    }
    
    /// Simulate a typical game frame with mixed operations
    fn simulate_game_frame(&self, world: &mut DynamicWorld) {
        // Spawn some entities
        let mut entities = Vec::new();
        for i in 0..20 {
            let entity = world.spawn();
            entities.push(entity);
            
            // Create some hierarchy
            if i > 0 && i % 5 == 0 {
                let _ = world.set_parent(entity, Some(entities[i - 1]));
            }
        }
        
        // Query operations (check if entities are alive)
        for &entity in &entities {
            let _ = world.is_alive(entity);
            let _ = world.get_parent(entity);
        }
        
        // Despawn some entities
        for &entity in entities.iter().take(5) {
            let _ = world.despawn(entity);
        }
        
        // Check entity count
        let _ = world.entity_count();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_benchmark_runner() {
        let runner = BenchmarkRunner::new(10, 5); // Small numbers for testing
        
        // Test individual benchmarks
        for backend_type in EcsBackendType::available_backends() {
            let result = runner.bench_entity_spawn(backend_type, 100);
            assert!(result.is_ok());
            
            let result = result.unwrap();
            assert_eq!(result.backend, backend_type);
            assert!(result.operations_per_second > 0.0);
        }
    }
    
    #[test]
    fn test_benchmark_suite() {
        let mut suite = BenchmarkSuite::new();
        
        let result = BenchmarkResult {
            backend: EcsBackendType::Custom,
            test_name: "test".to_string(),
            duration: Duration::from_millis(100),
            operations_per_second: 1000.0,
            memory_usage_estimate: None,
        };
        
        suite.add_result(result);
        assert_eq!(suite.results.len(), 1);
        
        let report = suite.generate_report();
        assert!(report.contains("ECS Backend Benchmark Report"));
        assert!(report.contains("Custom"));
    }
}