pub fn load_configuration_panel(mut contexts: EguiContexts, mut ui_state: ResMut<UIState>) {
    if !ui_state.show_load_panel {
        return;
    }

    let ctx = contexts.ctx_mut();

    // Clone the flag to avoid borrow issues
    let mut show_panel = ui_state.show_load_panel;

    egui::Window::new("📦 Load Configuration")
        .default_width(320.0)
        .default_pos([10.0, 500.0])
        .open(&mut show_panel)
        .show(ctx, |ui| {
            ui.heading("Load Properties");

            // Weight
            ui.horizontal(|ui| {
                ui.label("Weight:");
                if ui
                    .add(
                        egui::Slider::new(&mut ui_state.load_weight_kg, 100.0..=100_000.0)
                            .suffix(" kg")
                            .logarithmic(true),
                    )
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            // Quick weight presets
            ui.horizontal(|ui| {
                if ui.button("1t").clicked() {
                    ui_state.load_weight_kg = 1_000.0;
                    ui_state.mark_dirty();
                }
                if ui.button("5t").clicked() {
                    ui_state.load_weight_kg = 5_000.0;
                    ui_state.mark_dirty();
                }
                if ui.button("10t").clicked() {
                    ui_state.load_weight_kg = 10_000.0;
                    ui_state.mark_dirty();
                }
                if ui.button("20t").clicked() {
                    ui_state.load_weight_kg = 20_000.0;
                    ui_state.mark_dirty();
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Dimensions");

            // Length
            ui.horizontal(|ui| {
                ui.label("Length:");
                if ui
                    .add(egui::Slider::new(&mut ui_state.load_length_m, 0.5..=20.0).suffix(" m"))
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            // Width
            ui.horizontal(|ui| {
                ui.label("Width:");
                if ui
                    .add(egui::Slider::new(&mut ui_state.load_width_m, 0.5..=10.0).suffix(" m"))
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            // Height
            ui.horizontal(|ui| {
                ui.label("Height:");
                if ui
                    .add(egui::Slider::new(&mut ui_state.load_height_m, 0.2..=5.0).suffix(" m"))
                    .changed()
                {
                    ui_state.mark_dirty();
                }
            });

            let volume = ui_state.load_length_m * ui_state.load_width_m * ui_state.load_height_m;
            ui.label(format!("Volume: {:.2} m³", volume));

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            ui.heading("Rigging");

            // Pick points
            let mut pick_points_f32 = ui_state.num_pick_points as f32;
            ui.horizontal(|ui| {
                ui.label("Pick Points:");
                if ui
                    .add(egui::Slider::new(&mut pick_points_f32, 2.0..=4.0).step_by(2.0))
                    .changed()
                {
                    ui_state.num_pick_points = pick_points_f32 as usize;
                    ui_state.mark_dirty();
                }
            });

            ui.horizontal(|ui| {
                if ui.button("2-Point").clicked() {
                    ui_state.num_pick_points = 2;
                    ui_state.mark_dirty();
                }
                if ui.button("4-Point").clicked() {
                    ui_state.num_pick_points = 4;
                    ui_state.mark_dirty();
                }
            });

            ui.add_space(10.0);

            // Common load presets
            ui.separator();
            ui.heading("Load Presets");

            if ui.button("Steel Beam (5t, 12m)").clicked() {
                ui_state.load_weight_kg = 5_000.0;
                ui_state.load_length_m = 12.0;
                ui_state.load_width_m = 0.4;
                ui_state.load_height_m = 0.6;
                ui_state.num_pick_points = 2;
                ui_state.mark_dirty();
            }

            if ui.button("Concrete Panel (8t, 5m×2.5m)").clicked() {
                ui_state.load_weight_kg = 8_000.0;
                ui_state.load_length_m = 5.0;
                ui_state.load_width_m = 2.5;
                ui_state.load_height_m = 0.3;
                ui_state.num_pick_points = 4;
                ui_state.mark_dirty();
            }

            if ui.button("Shipping Container (3t, 6m)").clicked() {
                ui_state.load_weight_kg = 3_000.0;
                ui_state.load_length_m = 6.0;
                ui_state.load_width_m = 2.4;
                ui_state.load_height_m = 2.6;
                ui_state.num_pick_points = 4;
                ui_state.mark_dirty();
            }

            if ui.button("HVAC Unit (2t, 3m×1.5m)").clicked() {
                ui_state.load_weight_kg = 2_000.0;
                ui_state.load_length_m = 3.0;
                ui_state.load_width_m = 1.5;
                ui_state.load_height_m = 1.2;
                ui_state.num_pick_points = 4;
                ui_state.mark_dirty();
            }
        });

    // Update the state after the window closes
    ui_state.show_load_panel = show_panel;
}
