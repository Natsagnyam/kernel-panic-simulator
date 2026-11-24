#[derive(Clone, Debug)]
pub struct FaultTelemetry {
    pub leak_events: u32,
    pub leaked_bytes: u64,
    pub race_events: u32,
    pub divzero_events: u32,
    pub badptr_events: u32,
}

impl Default for FaultTelemetry {
    fn default() -> Self {
        Self {
            leak_events: 0,
            leaked_bytes: 0,
            race_events: 0,
            divzero_events: 0,
            badptr_events: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct AnomalyAssessment {
    pub leak_growth_score: f32,
    pub divzero_score: f32,
    pub badptr_score: f32,
    pub composite_score: f32,
    pub verdict: &'static str,
}

pub struct MlAnomalyDetector {
    pub enabled: bool,
    pub telemetry: FaultTelemetry,
    // Logistic / linear feature weights trained offline on fault telemetry
    w_leak_growth: f32,
    w_divzero: f32,
    w_badptr: f32,
    bias: f32,
}

impl Default for MlAnomalyDetector {
    fn default() -> Self {
        Self {
            enabled: false,
            telemetry: FaultTelemetry::default(),
            w_leak_growth: 2.2,
            w_divzero: 2.6,
            w_badptr: 3.1,
            bias: -1.8,
        }
    }
}

impl MlAnomalyDetector {
    pub fn record_dispatch(&mut self, cmd: &str) {
        match cmd {
            "memleak" => {
                self.telemetry.leak_events += 1;
                self.telemetry.leaked_bytes += 4096;
            }
            "race" => {
                self.telemetry.race_events += 1;
            }
            "divzero" => {
                self.telemetry.divzero_events += 1;
            }
            "badptr" => {
                self.telemetry.badptr_events += 1;
            }
            _ => {}
        }
    }

    pub fn reset(&mut self) {
        self.telemetry = FaultTelemetry::default();
    }

    fn sigmoid(z: f32) -> f32 {
        1.0 / (1.0 + (-z).exp())
    }

    pub fn score(&self) -> Option<AnomalyAssessment> {
        if !self.enabled {
            return None;
        }

        // Normalize features into [0.0, 1.0] ranges
        let leak_growth_rate = (self.telemetry.leaked_bytes as f32 / (64.0 * 1024.0)).min(1.0);
        let divzero_norm = (self.telemetry.divzero_events as f32 / 3.0).min(1.0);
        let badptr_norm = (self.telemetry.badptr_events as f32 / 2.0).min(1.0);

        let leak_growth_score = Self::sigmoid((leak_growth_rate * 4.0) - 2.0);
        let divzero_score = Self::sigmoid((divzero_norm * 4.0) - 2.0);
        let badptr_score = Self::sigmoid((badptr_norm * 4.0) - 2.0);

        let logit = self.bias
            + (self.w_leak_growth * leak_growth_rate)
            + (self.w_divzero * divzero_norm)
            + (self.w_badptr * badptr_norm);

        let composite_score = Self::sigmoid(logit);
        let verdict = if composite_score >= 0.80 {
            "CRITICAL PANIC ANOMALY"
        } else if composite_score >= 0.45 {
            "ELEVATED FAULT DRIFT"
        } else {
            "NOMINAL / LOW RISK"
        };

        Some(AnomalyAssessment {
            leak_growth_score,
            divzero_score,
            badptr_score,
            composite_score,
            verdict,
        })
    }
}
