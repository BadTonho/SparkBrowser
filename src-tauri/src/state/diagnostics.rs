use serde::Serialize;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Debug, Clone, Serialize)]
pub struct Sample {
    pub operation: &'static str,
    pub duration_ms: f64,
    pub affected_rows: u64,
    pub success: bool,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct Total {
    pub calls: u64,
    pub failures: u64,
    pub affected_rows: u64,
}
#[derive(Debug, Clone, Default, Serialize)]
pub struct Snapshot {
    pub samples: VecDeque<Sample>,
    pub totals: BTreeMap<&'static str, Total>,
}
#[derive(Default)]
pub struct Diagnostics(Mutex<Snapshot>);
impl Diagnostics {
    pub fn record(
        &self,
        operation: &'static str,
        elapsed: Duration,
        affected_rows: u64,
        success: bool,
    ) {
        if let Ok(mut data) = self.0.lock() {
            let total = data.totals.entry(operation).or_default();
            total.calls += 1;
            total.failures += u64::from(!success);
            total.affected_rows += affected_rows;
            data.samples.push_back(Sample {
                operation,
                duration_ms: elapsed.as_secs_f64() * 1000.0,
                affected_rows,
                success,
            });
            if data.samples.len() > 256 {
                data.samples.pop_front();
            }
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot, String> {
        self.0
            .lock()
            .map(|data| data.clone())
            .map_err(|e| e.to_string())
    }
    pub fn span(&self, operation: &'static str) -> Span<'_> {
        Span {
            diagnostics: self,
            operation,
            started: Instant::now(),
            success: false,
            affected_rows: 0,
        }
    }
}
pub struct Span<'a> {
    diagnostics: &'a Diagnostics,
    operation: &'static str,
    started: Instant,
    success: bool,
    affected_rows: u64,
}
impl Span<'_> {
    pub fn finish(&mut self, success: bool, affected_rows: u64) {
        self.success = success;
        self.affected_rows = affected_rows;
    }
}
impl Drop for Span<'_> {
    fn drop(&mut self) {
        self.diagnostics.record(
            self.operation,
            self.started.elapsed(),
            self.affected_rows,
            self.success,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn recent_samples_are_bounded_but_totals_keep_all_calls_and_writes() {
        let diagnostics = Diagnostics::default();
        for _ in 0..500 {
            diagnostics.record("session.persist", Duration::from_millis(1), 2, true);
        }
        let data = diagnostics.snapshot().unwrap();
        assert_eq!(data.samples.len(), 256);
        assert_eq!(data.totals["session.persist"].calls, 500);
        assert_eq!(data.totals["session.persist"].affected_rows, 1000);
        {
            let _failed = diagnostics.span("session.persist");
        }
        assert_eq!(
            diagnostics.snapshot().unwrap().totals["session.persist"].failures,
            1
        );
    }
}
