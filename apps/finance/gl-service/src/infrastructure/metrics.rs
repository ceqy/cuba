//! Metrics Module for GL Service
//!
//! Prometheus 指标实现

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tracing::info;

// ============================================================================
// Metrics Registry
// ============================================================================

/// GL Service 指标注册表
pub struct GlServiceMetrics {
    // 凭证计数器
    pub journal_entries_created: AtomicU64,
    pub journal_entries_posted: AtomicU64,
    pub journal_entries_reversed: AtomicU64,
    
    // 错误计数器
    pub validation_errors: AtomicU64,
    pub posting_errors: AtomicU64,
    
    // 业务指标
    pub open_items_count: AtomicU64,
    pub pending_approvals: AtomicU64,
}

impl GlServiceMetrics {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            journal_entries_created: AtomicU64::new(0),
            journal_entries_posted: AtomicU64::new(0),
            journal_entries_reversed: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            posting_errors: AtomicU64::new(0),
            open_items_count: AtomicU64::new(0),
            pending_approvals: AtomicU64::new(0),
        })
    }

    // ========================================================================
    // Counter Methods
    // ========================================================================

    pub fn inc_created(&self) {
        self.journal_entries_created.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_posted(&self) {
        self.journal_entries_posted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_reversed(&self) {
        self.journal_entries_reversed.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_validation_error(&self) {
        self.validation_errors.fetch_add(1, Ordering::Relaxed);
    }

    pub fn inc_posting_error(&self) {
        self.posting_errors.fetch_add(1, Ordering::Relaxed);
    }

    // ========================================================================
    // Gauge Methods
    // ========================================================================

    pub fn set_open_items(&self, count: u64) {
        self.open_items_count.store(count, Ordering::Relaxed);
    }

    pub fn set_pending_approvals(&self, count: u64) {
        self.pending_approvals.store(count, Ordering::Relaxed);
    }

    // ========================================================================
    // Export to Prometheus Format
    // ========================================================================

    pub fn to_prometheus(&self) -> String {
        let mut output = String::new();

        // 凭证计数器
        output.push_str("# HELP gl_journal_entries_created_total Total journal entries created\n");
        output.push_str("# TYPE gl_journal_entries_created_total counter\n");
        output.push_str(&format!(
            "gl_journal_entries_created_total {}\n",
            self.journal_entries_created.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP gl_journal_entries_posted_total Total journal entries posted\n");
        output.push_str("# TYPE gl_journal_entries_posted_total counter\n");
        output.push_str(&format!(
            "gl_journal_entries_posted_total {}\n",
            self.journal_entries_posted.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP gl_journal_entries_reversed_total Total journal entries reversed\n");
        output.push_str("# TYPE gl_journal_entries_reversed_total counter\n");
        output.push_str(&format!(
            "gl_journal_entries_reversed_total {}\n",
            self.journal_entries_reversed.load(Ordering::Relaxed)
        ));

        // 错误计数器
        output.push_str("# HELP gl_validation_errors_total Total validation errors\n");
        output.push_str("# TYPE gl_validation_errors_total counter\n");
        output.push_str(&format!(
            "gl_validation_errors_total {}\n",
            self.validation_errors.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP gl_posting_errors_total Total posting errors\n");
        output.push_str("# TYPE gl_posting_errors_total counter\n");
        output.push_str(&format!(
            "gl_posting_errors_total {}\n",
            self.posting_errors.load(Ordering::Relaxed)
        ));

        // 业务指标
        output.push_str("# HELP gl_open_items_count Current open items count\n");
        output.push_str("# TYPE gl_open_items_count gauge\n");
        output.push_str(&format!(
            "gl_open_items_count {}\n",
            self.open_items_count.load(Ordering::Relaxed)
        ));

        output.push_str("# HELP gl_pending_approvals Current pending approvals\n");
        output.push_str("# TYPE gl_pending_approvals gauge\n");
        output.push_str(&format!(
            "gl_pending_approvals {}\n",
            self.pending_approvals.load(Ordering::Relaxed)
        ));

        output
    }
}

impl Default for GlServiceMetrics {
    fn default() -> Self {
        Self {
            journal_entries_created: AtomicU64::new(0),
            journal_entries_posted: AtomicU64::new(0),
            journal_entries_reversed: AtomicU64::new(0),
            validation_errors: AtomicU64::new(0),
            posting_errors: AtomicU64::new(0),
            open_items_count: AtomicU64::new(0),
            pending_approvals: AtomicU64::new(0),
        }
    }
}

// ============================================================================
// Health Check
// ============================================================================

#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub status: HealthState,
    pub database: ComponentHealth,
    pub kafka: ComponentHealth,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthState,
    pub message: Option<String>,
}

impl HealthStatus {
    pub fn healthy() -> Self {
        Self {
            status: HealthState::Healthy,
            database: ComponentHealth {
                name: "database".to_string(),
                status: HealthState::Healthy,
                message: None,
            },
            kafka: ComponentHealth {
                name: "kafka".to_string(),
                status: HealthState::Healthy,
                message: None,
            },
        }
    }

    pub fn to_json(&self) -> String {
        format!(
            r#"{{"status":"{}","components":{{"database":"{}","kafka":"{}"}}}}"#,
            match self.status {
                HealthState::Healthy => "healthy",
                HealthState::Degraded => "degraded",
                HealthState::Unhealthy => "unhealthy",
            },
            match self.database.status {
                HealthState::Healthy => "up",
                HealthState::Degraded => "degraded",
                HealthState::Unhealthy => "down",
            },
            match self.kafka.status {
                HealthState::Healthy => "up",
                HealthState::Degraded => "degraded",
                HealthState::Unhealthy => "down",
            }
        )
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metrics_counters() {
        let metrics = GlServiceMetrics::new();
        
        metrics.inc_created();
        metrics.inc_created();
        metrics.inc_posted();

        assert_eq!(metrics.journal_entries_created.load(Ordering::Relaxed), 2);
        assert_eq!(metrics.journal_entries_posted.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_prometheus_output() {
        let metrics = GlServiceMetrics::new();
        metrics.inc_created();
        
        let output = metrics.to_prometheus();
        assert!(output.contains("gl_journal_entries_created_total 1"));
    }

    #[test]
    fn test_health_status() {
        let health = HealthStatus::healthy();
        assert_eq!(health.status, HealthState::Healthy);
        
        let json = health.to_json();
        assert!(json.contains("healthy"));
    }
}
