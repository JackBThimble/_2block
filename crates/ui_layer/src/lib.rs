mod ui_state;

use bevy::{prelude::*, window::PrimaryWindow};
use bevy_egui::{
    EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
    egui::{self},
};
use crane_core::CraneSpec;
use ui_state::UiState;

pub struct UiLayerPlugin;

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>();
        app.add_plugins(EguiPlugin {
            ..Default::default()
        });
        app.add_systems(
            EguiPrimaryContextPass,
            (ui_system, main_menu_panel, crane_configuration_panel),
        );
    }
}

fn ui_system(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) -> Result {
    let ctx = contexts.ctx_mut()?;

    egui::Window::new("*")
        .title_bar(false)
        .default_pos(egui::pos2(60.0, 40.0))
        .resizable(false)
        .movable(true)
        .fixed_size([60.0, 60.0])
        .frame(egui::Frame {
            fill: egui::Color32::from_rgba_unmultiplied(40, 40, 40, 230),
            corner_radius: egui::CornerRadius::same(10),
            stroke: egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
            inner_margin: egui::Margin::same(5),
            ..Default::default()
        })
        .show(ctx, |ui| {
            ui.centered_and_justified(|ui| {
                let button = egui::Button::new(
                    egui::RichText::new("☰")
                        .size(40.0)
                        .color(egui::Color32::WHITE),
                )
                .frame(false);

                if ui.add(button).clicked() {
                    ui_state.show_main_menu = !ui_state.show_main_menu;
                }
            });
        });
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CraneModel {
    LiebherrLTM1100,
    Grove,
    Taldano,
}

fn main_menu_panel(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) -> Result {
    if !ui_state.show_main_menu {
        return Ok(());
    }

    let ctx = contexts.ctx_mut()?;

    egui::Window::new("Main Menu")
        .default_width(300.0)
        .resizable(true)
        .collapsible(false)
        .scroll([false, true])
        .show(ctx, |ui| {
            ui.heading("Panels");
            ui.add_space(10.0);

            if ui
                .button(if ui_state.show_crane_panel {
                    "✓ Crane Configuration"
                } else {
                    " Crane Configuration"
                })
                .clicked()
            {
                ui_state.show_crane_panel = !ui_state.show_crane_panel;
            }
            if ui
                .button(if ui_state.show_load_panel {
                    "✓ Load Configuration"
                } else {
                    " Load Configuration"
                })
                .clicked()
            {
                ui_state.show_load_panel = !ui_state.show_load_panel;
            }

            if ui
                .button(if ui_state.show_analysis_panel {
                    "✓ Analysis"
                } else {
                    " Analysis"
                })
                .clicked()
            {
                ui_state.show_analysis_panel = !ui_state.show_analysis_panel;
            }

            if ui
                .button(if ui_state.show_scene_controls {
                    "✓ Scene Controls"
                } else {
                    " Scene Controls"
                })
                .clicked()
            {
                ui_state.show_scene_controls = !ui_state.show_scene_controls;
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Quick Actions");

            if ui.button("🏗 Select Crane").clicked() {
                ui_state.show_crane_selector = true;
            }

            if ui.button("Reset All").clicked() {
                let spec = ui_state.get_selected_crane_spec();
                ui_state.boom_length_m =
                    (spec.boom_length_range.0 + spec.boom_length_range.1) / 2.0;
                ui_state.boom_angle_deg = 60.0;
                ui_state.swing_angle_deg = 0.0;
                ui_state.hoist_length_m =
                    (spec.hoist_length_range.0 + spec.hoist_length_range.1) / 2.0;
                ui_state.outrigger_extension_pct = 100.0;
                ui_state.counterweight_slabs = spec.counterweight_max_slabs / 2;

                ui_state.load_weight_kg = 8000.0;
                ui_state.load_length_m = 5.0;
                ui_state.load_width_m = 2.5;
                ui_state.load_height_m = 1.2;
                ui_state.num_pick_points = 4;

                ui_state.mark_dirty();
            }

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Display Options");

            ui.checkbox(&mut ui_state.show_outriggers, "Show Outriggers");
            ui.checkbox(&mut ui_state.show_counterweight, "Show Counterweight");
            ui.checkbox(&mut ui_state.show_cable, "Show cable");
            ui.checkbox(&mut ui_state.show_slings, "Show Slings");
            ui.checkbox(&mut ui_state.show_pick_points, "Show Pick Points");
            ui.checkbox(&mut ui_state.show_cog, "Show Center of Gravity");
            ui.checkbox(&mut ui_state.show_ground_pressure, "Show Ground Pressure");

            ui.add_space(10.0);
            ui.separator();

            if ui.button("Close Menu").clicked() {
                ui_state.show_main_menu = false;
            }
        });

    Ok(())
}

fn crane_configuration_panel(mut contexts: EguiContexts, mut ui_state: ResMut<UiState>) {
    if !ui_state.show_crane_panel {
        return;
    }

    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => return,
    };

    let mut show_panel = ui_state.show_crane_panel;

    egui::Window::new("Crane Configuration")
        .default_width(320.0)
        .default_pos([10.0, 10.0])
        .open(&mut show_panel)
        .show(ctx, |ui| {
            let spec = ui_state.get_selected_crane_spec();

            ui.heading("Crane Model");
            ui.horizontal(|ui| {
                ui.label(format!("{} {}", spec.manufacturer, spec.model));
                if ui.button("Change...").clicked() {
                    ui_state.show_crane_selector = true;
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Boom");
            ui.horizontal(|ui| {
                ui.label("Length:");
                if ui
                    .add(
                        egui::Slider::new(
                            &mut ui_state.boom_length_m,
                            spec.boom_length_range.0..=spec.boom_length_range.1,
                        )
                        .suffix(" m"),
                    )
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Swing:");
                if ui
                    .add(egui::Slider::new(&mut ui_state.swing_angle_deg, 0.0..=360.0).suffix("°"))
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Hoist");
            ui.horizontal(|ui| {
                ui.label("Cable Length:");
                if ui
                    .add(
                        egui::Slider::new(
                            &mut ui_state.hoist_length_m,
                            spec.hoist_length_range.0..=spec.hoist_length_range.1,
                        )
                        .suffix(" m"),
                    )
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Outriggers");
            ui.horizontal(|ui| {
                ui.label("Extension:");
                if ui
                    .add(
                        egui::Slider::new(&mut ui_state.outrigger_extension_pct, 0.0..=100.0)
                            .suffix("%"),
                    )
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("Retract").clicked() {
                    ui_state.outrigger_extension_pct = 0.0;
                    ui_state.mark_dirty();
                }
                if ui.button("50%").clicked() {
                    ui_state.outrigger_extension_pct = 50.0;
                    ui_state.mark_dirty();
                }
                if ui.button("Full").clicked() {
                    ui_state.outrigger_extension_pct = 100.0;
                    ui_state.mark_dirty();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Counterweight");
            let mut cw_slabs_f32 = ui_state.counterweight_slabs as f32;
            ui.horizontal(|ui| {
                ui.label("Slabs:");
                if ui
                    .add(egui::Slider::new(
                        &mut cw_slabs_f32,
                        0.0..=spec.counterweight_max_slabs as f32,
                    ))
                    .changed()
                {
                    ui_state.counterweight_slabs = cw_slabs_f32 as usize;
                    ui_state.mark_dirty();
                }
            });

            let total_cw = ui_state.counterweight_slabs as f32 * spec.counterweight_slab_weight_kg;
            ui.label(format!("Total: {:.0} kg", total_cw));

            ui.horizontal(|ui| {
                if ui.button("None").clicked() {
                    ui_state.counterweight_slabs = 0;
                    ui_state.mark_dirty();
                }
                if ui.button("Light").clicked() {
                    ui_state.counterweight_slabs = spec.counterweight_max_slabs / 4;
                    ui_state.mark_dirty();
                }
                if ui.button("Medium").clicked() {
                    ui_state.counterweight_slabs = spec.counterweight_max_slabs / 2;
                    ui_state.mark_dirty();
                }
                if ui.button("Heavy").clicked() {
                    ui_state.counterweight_slabs = spec.counterweight_max_slabs;
                    ui_state.mark_dirty();
                }
            })
        });
}
