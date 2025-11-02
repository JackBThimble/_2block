// crates/crane_core/src/crane_data/capacity.rs - COMPLETE REWRITE

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Single point on a load chart
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityPoint {
    pub radius_m: f32,
    pub capacity_kg: f32,
}

/// Load chart for a specific boom length and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadChart {
    pub boom_length_m: f32,
    pub points: Vec<CapacityPoint>,

    /// Optional configuration notes
    pub notes: Option<String>,
}

impl LoadChart {
    pub fn new(boom_length_m: f32) -> Self {
        Self {
            boom_length_m,
            points: Vec::new(),
            notes: None,
        }
    }

    /// Add a capacity point
    pub fn add_point(&mut self, radius_m: f32, capacity_kg: f32) {
        self.points.push(CapacityPoint {
            radius_m,
            capacity_kg,
        });

        // Keep points sorted by radius
        self.points
            .sort_by(|a, b| a.radius_m.partial_cmp(&b.radius_m).unwrap());
    }

    /// Get capacity at specific radius (linear interpolation)
    pub fn get_capacity_at_radius(&self, radius_m: f32) -> Option<f32> {
        if self.points.is_empty() {
            return None;
        }

        // Find surrounding points
        let mut lower: Option<&CapacityPoint> = None;
        let mut upper: Option<&CapacityPoint> = None;

        for point in &self.points {
            if point.radius_m <= radius_m
                && (lower.is_none() || point.radius_m > lower.unwrap().radius_m)
            {
                lower = Some(point);
            }
            if point.radius_m >= radius_m
                && (upper.is_none() || point.radius_m < upper.unwrap().radius_m)
            {
                upper = Some(point);
            }
        }

        match (lower, upper) {
            (Some(l), Some(u)) if (l.radius_m - u.radius_m).abs() < 0.01 => {
                // Exact match
                Some(l.capacity_kg)
            }
            (Some(l), Some(u)) => {
                // Linear interpolation
                let t = (radius_m - l.radius_m) / (u.radius_m - l.radius_m);
                Some(l.capacity_kg + t * (u.capacity_kg - l.capacity_kg))
            }
            (Some(l), None) => {
                // Beyond max radius - return last known capacity (conservative)
                Some(l.capacity_kg)
            }
            (None, Some(u)) => {
                // Before min radius - return first capacity
                Some(u.capacity_kg)
            }
            _ => None,
        }
    }

    /// Get maximum radius on this chart
    pub fn max_radius(&self) -> f32 {
        self.points.iter().map(|p| p.radius_m).fold(0.0, f32::max)
    }

    /// Get minimum radius on this chart
    pub fn min_radius(&self) -> f32 {
        self.points
            .iter()
            .map(|p| p.radius_m)
            .fold(f32::MAX, f32::min)
    }

    /// Get maximum capacity on this chart
    pub fn max_capacity(&self) -> f32 {
        self.points
            .iter()
            .map(|p| p.capacity_kg)
            .fold(0.0, f32::max)
    }
}

/// Complete capacity chart system for a crane
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityChart {
    /// Load charts indexed by boom length (as string for HashMap key)
    pub charts: HashMap<String, LoadChart>,

    /// De-rating factors for different operating conditions
    pub over_side_factor: f32, // Typically 0.75-0.85
    pub over_rear_factor: f32, // Typically 0.75-0.85
    pub dynamic_factor: f32,   // For moving loads (0.85)

    /// Configuration-specific factors
    pub outrigger_intermediate_factor: f32, // When not fully extended
    pub on_tires_factor: f32, // Very low ~0.3-0.5
}

impl CapacityChart {
    pub fn new() -> Self {
        Self {
            charts: HashMap::new(),
            over_side_factor: 0.85,
            over_rear_factor: 0.75,
            dynamic_factor: 0.85,
            outrigger_intermediate_factor: 0.85,
            on_tires_factor: 0.40,
        }
    }

    /// Add a load chart for a specific boom length
    pub fn add_chart(&mut self, chart: LoadChart) {
        let key = format!("{:.1}", chart.boom_length_m);
        self.charts.insert(key, chart);
    }

    /// Get capacity for given configuration
    pub fn get_capacity(
        &self,
        boom_length_m: f32,
        radius_m: f32,
        swing_angle_deg: f32,
        outrigger_extension_pct: f32,
        on_tires: bool,
    ) -> Option<f32> {
        // Find chart for boom length (or interpolate between charts)
        let chart = self.find_chart_for_boom_length(boom_length_m)?;

        // Get base capacity from chart
        let mut capacity = chart.get_capacity_at_radius(radius_m)?;

        // Apply swing angle factor (over side/rear)
        capacity *= self.get_swing_factor(swing_angle_deg);

        // Apply outrigger extension factor
        if outrigger_extension_pct < 1.0 {
            capacity *= self.outrigger_intermediate_factor;
        }

        // Apply on-tires factor (major de-rating)
        if on_tires {
            capacity *= self.on_tires_factor;
        }

        Some(capacity)
    }

    /// Get swing angle factor
    fn get_swing_factor(&self, swing_angle_deg: f32) -> f32 {
        // Normalize angle to 0-360
        let angle = swing_angle_deg.rem_euclid(360.0);

        // 0° = front, 90° = side, 180° = rear, 270° = side
        if (45.0..135.0).contains(&angle) {
            self.over_side_factor // Right side
        } else if (135.0..225.0).contains(&angle) {
            self.over_rear_factor // Rear
        } else if (225.0..315.0).contains(&angle) {
            self.over_side_factor // Left side
        } else {
            1.0 // Front (no de-rating)
        }
    }

    /// Find chart for boom length (exact match or closest)
    fn find_chart_for_boom_length(&self, boom_length_m: f32) -> Option<&LoadChart> {
        let key = format!("{:.1}", boom_length_m);

        // Try exact match first
        if let Some(chart) = self.charts.get(&key) {
            return Some(chart);
        }

        // Find closest chart
        let mut closest: Option<(&String, &LoadChart)> = None;
        let mut min_diff = f32::MAX;

        for (k, chart) in &self.charts {
            let diff = (chart.boom_length_m - boom_length_m).abs();
            if diff < min_diff {
                min_diff = diff;
                closest = Some((k, chart));
            }
        }

        closest.map(|(_, chart)| chart)
    }

    /// Interpolate capacity between two boom lengths
    pub fn get_capacity_interpolated(&self, boom_length_m: f32, radius_m: f32) -> Option<f32> {
        // Find charts above and below boom length
        let mut lower_chart: Option<&LoadChart> = None;
        let mut upper_chart: Option<&LoadChart> = None;

        for chart in self.charts.values() {
            if chart.boom_length_m <= boom_length_m
                && (lower_chart.is_none()
                    || chart.boom_length_m > lower_chart.unwrap().boom_length_m)
            {
                lower_chart = Some(chart);
            }
            if chart.boom_length_m >= boom_length_m
                && (upper_chart.is_none()
                    || chart.boom_length_m < upper_chart.unwrap().boom_length_m)
            {
                upper_chart = Some(chart);
            }
        }

        match (lower_chart, upper_chart) {
            (Some(lower), Some(upper))
                if (lower.boom_length_m - upper.boom_length_m).abs() < 0.1 =>
            {
                // Exact match or very close
                lower.get_capacity_at_radius(radius_m)
            }
            (Some(lower), Some(upper)) => {
                // Interpolate between charts
                let lower_cap = lower.get_capacity_at_radius(radius_m)?;
                let upper_cap = upper.get_capacity_at_radius(radius_m)?;

                let t = (boom_length_m - lower.boom_length_m)
                    / (upper.boom_length_m - lower.boom_length_m);

                Some(lower_cap + t * (upper_cap - lower_cap))
            }
            (Some(chart), None) | (None, Some(chart)) => {
                // Use closest available
                chart.get_capacity_at_radius(radius_m)
            }
            _ => None,
        }
    }
}

impl Default for CapacityChart {
    fn default() -> Self {
        Self::new()
    }
}

/// Load chart parser for CSV and text formats
pub struct LoadChartParser;

impl LoadChartParser {
    /// Parse CSV format load chart
    ///
    /// Expected CSV format:
    /// ```csv
    /// boom_length,radius,capacity
    /// 30.0,3.0,100000
    /// 30.0,5.0,80000
    /// 30.0,10.0,40000
    /// ```
    pub fn parse_csv(csv_data: &str) -> Result<Vec<LoadChart>, String> {
        let mut charts_map: HashMap<String, LoadChart> = HashMap::new();

        for (line_num, line) in csv_data.lines().enumerate() {
            // Skip header and empty lines
            if line_num == 0 || line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if parts.len() < 3 {
                return Err(format!(
                    "Line {}: Expected 3 columns, got {}",
                    line_num + 1,
                    parts.len()
                ));
            }

            let boom_length: f32 = parts[0].parse().map_err(|_| {
                format!("Line {}: Invalid boom length '{}'", line_num + 1, parts[0])
            })?;
            let radius: f32 = parts[1]
                .parse()
                .map_err(|_| format!("Line {}: Invalid radius '{}'", line_num + 1, parts[1]))?;
            let capacity: f32 = parts[2]
                .parse()
                .map_err(|_| format!("Line {}: Invalid capacity '{}'", line_num + 1, parts[2]))?;

            let key = format!("{:.1}", boom_length);
            let chart = charts_map
                .entry(key)
                .or_insert_with(|| LoadChart::new(boom_length));
            chart.add_point(radius, capacity);
        }

        Ok(charts_map.into_values().collect())
    }

    /// Parse JSON format load chart
    ///
    /// Expected JSON format:
    /// ```json
    /// {
    ///   "charts": [
    ///     {
    ///       "boom_length_m": 30.0,
    ///       "points": [
    ///         {"radius_m": 3.0, "capacity_kg": 100000},
    ///         {"radius_m": 5.0, "capacity_kg": 80000}
    ///       ]
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_json(json_data: &str) -> Result<Vec<LoadChart>, String> {
        #[derive(Deserialize)]
        struct ChartData {
            charts: Vec<LoadChart>,
        }

        let data: ChartData =
            serde_json::from_str(json_data).map_err(|e| format!("JSON parse error: {}", e))?;

        Ok(data.charts)
    }

    /// Parse manufacturer-style table format
    ///
    /// Expected format:
    /// ```text
    /// BOOM LENGTH: 30.0m
    /// Radius(m)  Capacity(kg)
    /// 3.0        100000
    /// 5.0        80000
    /// 10.0       40000
    ///
    /// BOOM LENGTH: 40.0m
    /// Radius(m)  Capacity(kg)
    /// 3.0        90000
    /// 5.0        70000
    /// ```
    pub fn parse_table(table_data: &str) -> Result<Vec<LoadChart>, String> {
        let mut charts = Vec::new();
        let mut current_chart: Option<LoadChart> = None;

        for line in table_data.lines() {
            let line = line.trim();

            // Check for boom length header
            if line.to_uppercase().starts_with("BOOM LENGTH:") {
                // Save previous chart if exists
                if let Some(chart) = current_chart.take()
                    && !chart.points.is_empty()
                {
                    charts.push(chart);
                }

                // Parse boom length
                let boom_str = line
                    .split(':')
                    .nth(1)
                    .ok_or_else(|| "Invalid BOOM LENGTH line".to_string())?
                    .trim()
                    .trim_end_matches('m');

                let boom_length: f32 = boom_str
                    .parse()
                    .map_err(|_| format!("Invalid boom length: {}", boom_str))?;

                current_chart = Some(LoadChart::new(boom_length));
                continue;
            }

            // Skip header lines
            if line.to_uppercase().contains("RADIUS") || line.is_empty() {
                continue;
            }

            // Parse data line
            if let Some(chart) = &mut current_chart {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2
                    && let (Ok(radius), Ok(capacity)) =
                        (parts[0].parse::<f32>(), parts[1].parse::<f32>())
                {
                    chart.add_point(radius, capacity);
                }
            }
        }

        // Add final chart
        if let Some(chart) = current_chart
            && !chart.points.is_empty()
        {
            charts.push(chart);
        }

        Ok(charts)
    }
}

/// Builder for creating capacity charts programmatically
pub struct CapacityChartBuilder {
    chart: CapacityChart,
}

impl CapacityChartBuilder {
    pub fn new() -> Self {
        Self {
            chart: CapacityChart::new(),
        }
    }

    pub fn with_over_side_factor(mut self, factor: f32) -> Self {
        self.chart.over_side_factor = factor;
        self
    }

    pub fn with_over_rear_factor(mut self, factor: f32) -> Self {
        self.chart.over_rear_factor = factor;
        self
    }

    pub fn with_dynamic_factor(mut self, factor: f32) -> Self {
        self.chart.dynamic_factor = factor;
        self
    }

    pub fn add_chart(mut self, chart: LoadChart) -> Self {
        self.chart.add_chart(chart);
        self
    }

    pub fn add_charts_from_csv(mut self, csv_data: &str) -> Result<Self, String> {
        let charts = LoadChartParser::parse_csv(csv_data)?;
        for chart in charts {
            self.chart.add_chart(chart);
        }
        Ok(self)
    }

    pub fn add_charts_from_json(mut self, json_data: &str) -> Result<Self, String> {
        let charts = LoadChartParser::parse_json(json_data)?;
        for chart in charts {
            self.chart.add_chart(chart);
        }
        Ok(self)
    }

    pub fn add_charts_from_table(mut self, table_data: &str) -> Result<Self, String> {
        let charts = LoadChartParser::parse_table(table_data)?;
        for chart in charts {
            self.chart.add_chart(chart);
        }
        Ok(self)
    }

    pub fn build(self) -> CapacityChart {
        self.chart
    }
}

impl Default for CapacityChartBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Example load charts for testing
impl CapacityChart {
    /// Liebherr LTM 1100-5.2 example capacity chart
    pub fn example_liebherr_ltm_1100() -> Self {
        let csv_data = r#"boom_length,radius,capacity
30.0,3.0,100000
30.0,5.0,80000
30.0,10.0,40000
30.0,15.0,25000
30.0,20.0,15000
30.0,25.0,10000
40.0,3.0,90000
40.0,5.0,70000
40.0,10.0,35000
40.0,20.0,12000
40.0,30.0,7000
40.0,35.0,5000
50.0,3.0,80000
50.0,10.0,30000
50.0,20.0,10000
50.0,30.0,6000
50.0,40.0,4000
50.0,45.0,3000"#;

        CapacityChartBuilder::new()
            .with_over_side_factor(0.85)
            .with_over_rear_factor(0.75)
            .add_charts_from_csv(csv_data)
            .unwrap()
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_chart_interpolation() {
        let mut chart = LoadChart::new(30.0);
        chart.add_point(3.0, 100_000.0);
        chart.add_point(10.0, 40_000.0);

        // Test exact match
        assert_eq!(chart.get_capacity_at_radius(3.0), Some(100_000.0));

        // Test interpolation
        let mid_capacity = chart.get_capacity_at_radius(6.5).unwrap();
        assert!((mid_capacity - 70_000.0).abs() < 1.0);
    }

    #[test]
    fn test_csv_parser() {
        let csv = "boom_length,radius,capacity\n30.0,3.0,100000\n30.0,5.0,80000\n40.0,3.0,90000";
        let charts = LoadChartParser::parse_csv(csv).unwrap();

        assert_eq!(charts.len(), 2);
        assert_eq!(charts[0].points.len(), 2);
    }

    #[test]
    fn test_table_parser() {
        let table = r#"
BOOM LENGTH: 30.0m
Radius(m)  Capacity(kg)
3.0        100000
5.0        80000

BOOM LENGTH: 40.0m
Radius(m)  Capacity(kg)
3.0        90000
        "#;

        let charts = LoadChartParser::parse_table(table).unwrap();
        assert_eq!(charts.len(), 2);
    }

    #[test]
    fn test_json_parser() {
        let json = r#"{
            "charts": [
                {
                    "boom_length_m": 30.0,
                    "points": [
                        {"radius_m": 3.0, "capacity_kg": 100000},
                        {"radius_m": 5.0, "capacity_kg": 80000}
                    ]
                }
            ]
        }"#;

        let charts = LoadChartParser::parse_json(json).unwrap();
        assert_eq!(charts.len(), 1);
        assert_eq!(charts[0].points.len(), 2);
    }

    #[test]
    fn test_swing_factor() {
        let chart = CapacityChart::new();

        // Front (no de-rating)
        assert_eq!(chart.get_swing_factor(0.0), 1.0);
        assert_eq!(chart.get_swing_factor(30.0), 1.0);

        // Side
        assert_eq!(chart.get_swing_factor(90.0), 0.85);
        assert_eq!(chart.get_swing_factor(270.0), 0.85);

        // Rear
        assert_eq!(chart.get_swing_factor(180.0), 0.75);
    }

    #[test]
    fn test_capacity_chart_builder() {
        let csv = "boom_length,radius,capacity\n30.0,3.0,100000";

        let chart = CapacityChartBuilder::new()
            .with_over_side_factor(0.8)
            .add_charts_from_csv(csv)
            .unwrap()
            .build();

        assert_eq!(chart.over_side_factor, 0.8);
        assert_eq!(chart.charts.len(), 1);
    }

    #[test]
    fn test_capacity_interpolation_between_booms() {
        let mut chart = CapacityChart::new();

        let mut chart_30 = LoadChart::new(30.0);
        chart_30.add_point(10.0, 100_000.0);

        let mut chart_40 = LoadChart::new(40.0);
        chart_40.add_point(10.0, 80_000.0);

        chart.add_chart(chart_30);
        chart.add_chart(chart_40);

        // Test interpolation at 35m boom
        let capacity = chart.get_capacity_interpolated(35.0, 10.0).unwrap();
        assert!((capacity - 90_000.0).abs() < 1.0);
    }
}
