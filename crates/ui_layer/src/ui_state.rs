use bevy::prelude::*;
use crane_core::CraneSpec;

#[derive(Resource)]
pub struct UiState {
    // Crane configuration
    pub selected_crane_id: String,
    pub boom_length_m: f32,
    pub boom_angle_deg: f32,
    pub swing_angle_deg: f32,
    pub hoist_length_m: f32,
    pub outrigger_extension_pct: f32,
    pub counterweight_slabs: usize,

    // Load configuration
    pub load_weight_kg: f32,
    pub load_length_m: f32,
    pub load_width_m: f32,
    pub load_height_m: f32,
    pub num_pick_points: usize,

    // UI state
    pub show_crane_panel: bool,
    pub show_load_panel: bool,
    pub show_analysis_panel: bool,
    pub show_scene_controls: bool,
    pub show_crane_selector: bool,
    pub show_main_menu: bool,
    pub viewport_width: f32,
    pub viewport_height: f32,

    // Display toggles
    pub show_outriggers: bool,
    pub show_counterweight: bool,
    pub show_cable: bool,
    pub show_slings: bool,
    pub show_pick_points: bool,
    pub show_cog: bool,
    pub show_ground_pressure: bool,

    // Flags
    pub needs_recalculation: bool,
    pub needs_crane_respawn: bool,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            // Crane defaults (Liebherr LTM 1100)
            selected_crane_id: "liebherr_ltm_1100_5_2".to_string(),
            boom_length_m: 35.0,
            boom_angle_deg: 65.0,
            swing_angle_deg: 0.0,
            hoist_length_m: 12.0,
            outrigger_extension_pct: 100.0,
            counterweight_slabs: 8,

            // Load defaults
            load_weight_kg: 8000.0,
            load_length_m: 5.0,
            load_width_m: 2.5,
            load_height_m: 1.2,
            num_pick_points: 4,

            // UI panels
            show_crane_panel: true,
            show_load_panel: true,
            show_analysis_panel: true,
            show_scene_controls: false,
            show_crane_selector: false,

            // Display toggles
            show_outriggers: true,
            show_counterweight: true,
            show_cable: true,
            show_slings: true,
            show_pick_points: true,
            show_cog: true,
            show_ground_pressure: false,
            show_main_menu: false,
            viewport_width: 1280.0,
            viewport_height: 760.0,

            // Flags
            needs_recalculation: false,
            needs_crane_respawn: false,
        }
    }
}

impl UiState {
    /// Mark that configuration changed and needs recalculation
    pub fn mark_dirty(&mut self) {
        self.needs_recalculation = true;
    }

    /// Mark that crane needs to be respawned (model changed)
    pub fn mark_crane_respawn(&mut self) {
        self.needs_crane_respawn = true;
    }

    /// Get currently selected crane spec
    pub fn get_selected_crane_spec(&self) -> CraneSpec {
        CraneSpec::all_specs()
            .into_iter()
            .find(|spec| spec.id == self.selected_crane_id)
            .unwrap_or_else(|| CraneSpec::liebherr_ltm_1100())
    }
}
