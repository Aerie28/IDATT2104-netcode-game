// src/analysis.rs
use std::collections::HashMap;
use std::time::{Duration, Instant};
use crate::constants::{TEST_DURATION};
use crate::types::NetworkCondition;

pub struct PerformanceMetrics {
    pub avg_prediction_error: f32,
    pub max_prediction_error: f32,
    pub reconciliation_count: u32,
    pub input_lag_ms: i32,
}

pub struct PerformanceAnalyzer {
    conditions: Vec<NetworkCondition>,
    results: HashMap<String, PerformanceMetrics>,
    current_condition: Option<NetworkCondition>,
    current_index: usize,
    samples: Vec<f32>,
    start_time: Instant,
}

impl PerformanceAnalyzer {
    pub fn new(sample_duration: Duration) -> Self {
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

    pub fn record_prediction_error(&mut self, error: f32) {
        if self.current_condition.is_some() {
            self.samples.push(error);
        }
    }

    pub fn reset(&mut self) {
        self.current_index = 0;
        self.results.clear();
        self.current_condition = None;
        self.samples.clear();
    }

    pub fn is_test_complete(&self) -> bool {
        if self.current_condition.is_none() {
            return false;
        }

        self.start_time.elapsed() >= TEST_DURATION
    }

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