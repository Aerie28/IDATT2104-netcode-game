use crate::constants::{TEST_DURATION};
use crate::types::NetworkCondition;

use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Represents performance metrics for a network condition
pub struct PerformanceMetrics {
    pub avg_prediction_error: f32,
    pub max_prediction_error: f32,
    pub reconciliation_count: u32,
    pub input_lag_ms: i32,
}

/// Analyzes performance metrics under different network conditions
pub struct PerformanceAnalyzer {
    conditions: Vec<NetworkCondition>,
    results: HashMap<String, PerformanceMetrics>,
    current_condition: Option<NetworkCondition>,
    current_index: usize,
    samples: Vec<f32>,
    start_time: Instant,
}

/// Implementation of the PerformanceAnalyzer
impl PerformanceAnalyzer {
    /// Creates a new PerformanceAnalyzer with predefined network conditions for testing
    pub fn new(_sample_duration: Duration) -> Self {
        Self {
            conditions: vec![
                NetworkCondition { latency_ms: 200, packet_loss_percent: 10, name: "Very Poor".to_string() },
                NetworkCondition { latency_ms: 100, packet_loss_percent: 5, name: "Lossy".to_string() },
                NetworkCondition { latency_ms: 200, packet_loss_percent: 0, name: "Poor".to_string() },
                NetworkCondition { latency_ms: 100, packet_loss_percent: 0, name: "Average".to_string() },
                NetworkCondition { latency_ms: 50, packet_loss_percent: 0, name: "Good".to_string() },
                NetworkCondition { latency_ms: 0, packet_loss_percent: 0, name: "Ideal".to_string() },
            ],
            results: HashMap::new(),
            current_condition: None,
            current_index: 0,
            samples: Vec::new(),
            start_time: Instant::now(),
        }
    }

    /// Creates a new PerformanceAnalyzer with a custom set of network conditions
    pub fn start_next_test(&mut self) -> Option<NetworkCondition> {
        if self.current_index < self.conditions.len() {
            let condition = self.conditions[self.current_index].clone();
            self.current_condition = Some(condition.clone());
            self.samples.clear();
            self.start_time = Instant::now();
            self.current_index += 1;
            Some(condition)
        } else {
            None
        }
    }
    
    /// Records a prediction error for the current network condition
    pub fn record_prediction_error(&mut self, error: f32) {
        if self.current_condition.is_some() {
            self.samples.push(error);
        }
    }

    /// Resets the analyzer to start a new test
    pub fn reset(&mut self) {
        self.current_index = 0;
        self.results.clear();
        self.current_condition = None;
        self.samples.clear();
    }

    /// Checks if the current test is complete based on elapsed time
    pub fn is_test_complete(&self) -> bool {
        if self.current_condition.is_none() {
            return false;
        }

        self.start_time.elapsed() >= TEST_DURATION
    }

    /// Completes the current test and calculates performance metrics
    pub fn complete_current_test(&mut self) {
        if let Some(condition) = &self.current_condition {
            let avg_error = if self.samples.is_empty() {
                0.0
            } else {
                self.samples.iter().sum::<f32>() / self.samples.len() as f32
            };
            
            let max_error = self.samples.iter().fold(0.0_f32, |max, &x| f32::max(max, x));

            self.results.insert(condition.name.clone(), PerformanceMetrics {
                avg_prediction_error: avg_error,
                max_prediction_error: max_error,
                reconciliation_count: self.samples.len() as u32,
                input_lag_ms: condition.latency_ms,
            });
        }
    }

    /// Returns the results of the performance tests
    pub fn generate_report(&self) -> String {
        let mut report = "# Performance Analysis Report\n\n".to_string();
        report.push_str("| Network Condition | Avg Error | Max Error | Input Lag |\n");
        report.push_str("|------------------|-----------|-----------|----------|\n");

        for (condition, metrics) in &self.results {
            report.push_str(&format!("| {:<16} | {:>8.2} | {:>8.2} | {:>8} ms |\n",
                     condition,
                     metrics.avg_prediction_error,
                     metrics.max_prediction_error,
                     metrics.input_lag_ms));
        }
        report
    }
}

/// Tests for the PerformanceAnalyzer
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_new_analyzer() {
        let analyzer = PerformanceAnalyzer::new(Duration::from_secs(5));
        assert_eq!(analyzer.current_index, 0);
        assert!(analyzer.current_condition.is_none());
        assert!(analyzer.samples.is_empty());
        assert_eq!(analyzer.conditions.len(), 6);
    }

    #[test]
    fn test_record_prediction_error() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));

        // No condition selected yet, should not record
        analyzer.record_prediction_error(1.0);
        assert!(analyzer.samples.is_empty());

        // Start a test and record errors
        analyzer.start_next_test();
        analyzer.record_prediction_error(1.0);
        analyzer.record_prediction_error(2.0);
        assert_eq!(analyzer.samples, vec![1.0, 2.0]);
    }

    #[test]
    fn test_reset() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));

        // Run a full test
        analyzer.start_next_test();
        analyzer.record_prediction_error(1.0);
        analyzer.complete_current_test();

        // Now reset
        analyzer.reset();

        // Check everything is back to initial state
        assert_eq!(analyzer.current_index, 0);
        assert!(analyzer.current_condition.is_none());
        assert!(analyzer.samples.is_empty());
        assert!(analyzer.results.is_empty());
    }

    #[test]
    fn test_complete_current_test() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));

        // Start a test and record some errors
        analyzer.start_next_test();
        analyzer.record_prediction_error(1.0);
        analyzer.record_prediction_error(2.0);
        analyzer.record_prediction_error(3.0);

        // Complete the test
        analyzer.complete_current_test();

        // Check metrics
        let metrics = analyzer.results.get("Very Poor").unwrap();
        assert_eq!(metrics.avg_prediction_error, 2.0);
        assert_eq!(metrics.max_prediction_error, 3.0);
        assert_eq!(metrics.reconciliation_count, 3);
        assert_eq!(metrics.input_lag_ms, 200);
    }

    #[test]
    fn test_complete_current_test_with_empty_samples() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));
        analyzer.start_next_test();

        // Complete with no samples recorded
        analyzer.complete_current_test();

        let metrics = analyzer.results.get("Very Poor").unwrap();
        assert_eq!(metrics.avg_prediction_error, 0.0);
        assert_eq!(metrics.max_prediction_error, 0.0);
        assert_eq!(metrics.reconciliation_count, 0);
        assert_eq!(metrics.input_lag_ms, 200);
    }

    #[test]
    fn test_generate_report() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));

        // Run a test cycle
        analyzer.start_next_test();
        analyzer.record_prediction_error(1.5);
        analyzer.complete_current_test();

        // Generate and check report
        let report = analyzer.generate_report();
        assert!(report.contains("Performance Analysis Report"));
        assert!(report.contains("Very Poor"));
        assert!(report.contains("1.50"));
        assert!(report.contains("200 ms"));
    }

    #[test]
    fn test_multiple_conditions() {
        let mut analyzer = PerformanceAnalyzer::new(Duration::from_secs(1));

        // Test first condition
        analyzer.start_next_test();
        analyzer.record_prediction_error(1.5);
        analyzer.complete_current_test();

        // Test second condition
        analyzer.start_next_test();
        analyzer.record_prediction_error(0.8);
        analyzer.complete_current_test();

        // Check both conditions are in results
        assert!(analyzer.results.contains_key("Very Poor"));
        assert!(analyzer.results.contains_key("Lossy"));

        // Check report contains both
        let report = analyzer.generate_report();
        assert!(report.contains("Very Poor"));
        assert!(report.contains("Lossy"));
    }
}