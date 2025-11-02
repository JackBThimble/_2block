use crate::*;
use crane_core::{CraneConfiguration, CraneSpec, rigging::*};
use nalgebra::Point3;

pub fn spawn_test_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut scene_state: ResMut<SceneState>,
) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║     CRANE LIFT PLANNER - TEST SCENE      ║");
    println!("╚═══════════════════════════════════════════╝\n");

    // ========== CRANE SETUP ==========

    let crane_spec = CraneSpec::liebherr_ltm_1100();
    print_crane_spec(&crane_spec);

    let mut crane_config = CraneConfiguration::new(crane_spec);

    // Configure crane
    crane_config.position = Point3::origin();
    crane_config.boom_length_m = 35.0;
    crane_config.boom_angle_deg = 65.0;
    crane_config.swing_angle_deg = 0.0;
    crane_config.hoist_length_m = 12.0;

    // Deploy outriggers
    crane_config.outriggers.preset_max_extension();

    // Add counterweight
    crane_config.counterweight.preset_medium().unwrap();

    // Validate configuration
    println!("\n--- Validating Crane Configuration ---");
    match crane_config.validate() {
        Ok(_) => println!("✓ Configuration valid"),
        Err(e) => {
            eprintln!("✗ Configuration invalid: {}", e);
            return;
        }
    }

    // Check capacity
    let radius = crane_config.get_radius();
    println!("✓ Operating radius: {:.1}m", radius);

    if let Some(capacity) = crane_config.get_current_capacity() {
        println!(
            "✓ Current capacity: {:.0}kg at {:.1}m radius",
            capacity, radius
        );
    } else {
        println!("⚠ No capacity data available for this configuration");
    }

    // Spawn crane visual
    let _crane_entity = crane_renderer::spawn_crane(
        &mut commands,
        &mut meshes,
        &mut materials,
        crane_config.clone(),
    );

    scene_state.crane_config = Some(crane_config.clone());

    // ========== LOAD SETUP ==========

    println!("\n--- Setting Up Load ---");

    let load = create_test_load();
    print_load_info(&load);

    // Position load on ground directly below hook
    let hook_pos = crane_config.get_hook_position();
    let hook_pos_bevy = Vec3::new(hook_pos.x, hook_pos.z, hook_pos.y);

    let load_pos_bevy = Vec3::new(
        hook_pos_bevy.x,
        load.dimensions.z / 2.0, // Half height up from ground
        hook_pos_bevy.z,
    );

    println!("✓ Hook height: {:.1}m", hook_pos_bevy.y);
    println!(
        "✓ Load positioned at: [{:.1}, {:.1}, {:.1}]",
        load_pos_bevy.x, load_pos_bevy.y, load_pos_bevy.z
    );

    let _load_entity = load_renderer::spawn_load_at_position(
        &mut commands,
        &mut meshes,
        &mut materials,
        load.clone(),
        load_pos_bevy,
    );

    // ========== RIGGING SETUP ==========

    println!("\n--- Setting Up Rigging ---");

    let sling_spec = create_test_sling_spec();
    println!(
        "Sling: {} ({}kg capacity)",
        sling_spec.id, sling_spec.rated_capacity_kg
    );

    let slings = create_test_slings(&load, hook_pos, sling_spec);
    println!("✓ Created {} slings", slings.len());

    // ========== ANALYSIS ==========

    println!("\n--- Analyzing Rigging ---");

    let rigging_config = RiggingConfiguration {
        load: load.clone(),
        slings: slings.clone(),
        hardware: vec![],
        crane_hook_position: hook_pos,
    };

    match RiggingCalculator::analyze(&rigging_config) {
        Ok(analysis) => {
            print_rigging_analysis(&analysis);

            // Store results
            scene_state.rigging_config = Some(rigging_config.clone());
            scene_state.rigging_analysis = Some(analysis.clone());

            // Spawn slings visually
            for (i, (sling, tension)) in slings
                .iter()
                .zip(analysis.sling_tensions.iter())
                .enumerate()
            {
                sling_renderer::spawn_sling(
                    &mut commands,
                    &mut meshes,
                    &mut materials,
                    sling,
                    tension,
                    load_pos_bevy,
                    hook_pos_bevy,
                    i,
                );
            }

            println!("✓ Slings spawned");
        }
        Err(e) => {
            eprintln!("✗ Rigging analysis failed: {:?}", e);
            return;
        }
    }

    // ========== GROUND BEARING ==========

    println!("\n--- Analyzing Ground Bearing ---");

    if let Some(ground_analysis) = calculate_ground_bearing(&crane_config, &load) {
        print_ground_bearing_analysis(&ground_analysis);
        scene_state.ground_bearing_analysis = Some(ground_analysis);
    }

    println!("\n╔═══════════════════════════════════════════╗");
    println!("║          TEST SCENE COMPLETE!             ║");
    println!("╚═══════════════════════════════════════════╝\n");
}

// ========== HELPER FUNCTIONS ==========

fn create_test_load() -> Load {
    Load {
        weight_kg: 8000.0,
        center_of_gravity: Point3::new(0.0, 0.0, 0.6),
        dimensions: nalgebra::Vector3::new(5.0, 2.5, 1.2), // L, W, H
        pick_points: vec![
            crane_core::PickPoint {
                id: "front_left".to_string(),
                position: Point3::new(2.0, -1.0, 1.2),
                active: true,
            },
            crane_core::PickPoint {
                id: "front_right".to_string(),
                position: Point3::new(2.0, 1.0, 1.2),
                active: true,
            },
            crane_core::PickPoint {
                id: "rear_left".to_string(),
                position: Point3::new(-2.0, -1.0, 1.2),
                active: true,
            },
            crane_core::PickPoint {
                id: "rear_right".to_string(),
                position: Point3::new(-2.0, 1.0, 1.2),
                active: true,
            },
        ],
    }
}

fn create_test_sling_spec() -> SlingSpec {
    SlingSpec {
        id: "polyester_3t".to_string(),
        material: SlingMaterial::Synthetic {
            material: SyntheticMaterial::Polyester,
        },
        diameter_mm: None,
        width_mm: Some(90.0),
        length_m: 5.0,
        rated_capacity_kg: 3000.0,
        safety_factor: 5.0,
    }
}

fn create_test_slings(load: &Load, hook_pos: Point3<f32>, sling_spec: SlingSpec) -> Vec<Sling> {
    load.pick_points
        .iter()
        .filter(|pp| pp.active)
        .map(|pp| Sling {
            spec: sling_spec.clone(),
            hitch_type: HitchType::Vertical,
            attachment_point: pp.position,
            hook_point: hook_pos,
            angle_from_vertical: None,
            tension_kg: None,
        })
        .collect()
}

fn calculate_ground_bearing(
    crane_config: &CraneConfiguration,
    load: &Load,
) -> Option<crane_core::GroundBearingAnalysis> {
    use crane_core::ground_bearing::*;

    let total_weight = crane_config.get_total_weight_kg() + load.weight_kg;
    let contact_points = crane_config.outriggers.get_all_contact_points();

    if contact_points.is_empty() {
        return None;
    }

    // Distribute load equally (simplified - real calc would consider moments)
    let load_per_outrigger = total_weight / contact_points.len() as f32;

    let mut support_points = Vec::new();
    for (position, point) in contact_points {
        if let Some(outrigger) = crane_config.outriggers.get_outrigger(position)
            && let Some(pad_diameter) = outrigger.pad_diameter_m
        {
            // Good ground - use steel pad
            let support_point =
                SupportPoint::with_pad(point, load_per_outrigger, pad_diameter, PadMaterial::Steel);

            // Soft ground - use mat withh pad
            // let support_point = SupportPoint::with_mat_and_pad(
            // point,
            // load_per_outrigger,
            // 4.0, // 4m mat length
            // 3.0, // 3m mat width,
            // MatMaterial::TimberMat,
            // pad_diameter,
            // PadMaterial::Steel,
            // )
            support_points.push(support_point);
        }
    }

    let ground_config = GroundConfiguration {
        support_points,
        soil_type: SoilType::MediumClay,
        safety_factor: 2.0,
    };

    GroundBearingCalculator::analyze(&ground_config).ok()
}

// ========== PRINT FUNCTIONS ==========

fn print_crane_spec(spec: &CraneSpec) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║              CRANE DETAILS                ║");
    println!("╠═══════════════════════════════════════════╣");
    println!("║ Manufacturer: {:<30} ║", spec.manufacturer);
    println!("║ Model:        {:<30} ║", spec.model);
    println!("║ Type:         {:<30} ║", format!("{}", spec.crane_type));
    println!(
        "║ Capacity:     {:<30} ║",
        format!("{:.0} kg", spec.max_capacity_kg)
    );
    println!(
        "║ Boom Range:   {:<30} ║",
        format!(
            "{:.1}m - {:.1}m",
            spec.boom_length_range.0, spec.boom_length_range.1
        )
    );
    println!(
        "║ Hoist Range:  {:<30} ║",
        format!(
            "{:.1}m - {:.1}m",
            spec.hoist_length_range.0, spec.hoist_length_range.1
        )
    );
    println!(
        "║ Weight:       {:<30} ║",
        format!("{:.0} kg", spec.base_weight_kg)
    );
    println!("╚═══════════════════════════════════════════╝");
}

fn print_load_info(load: &Load) {
    println!("Load Details:");
    println!("  Weight: {:.0}kg", load.weight_kg);
    println!(
        "  Dimensions: {:.1}m × {:.1}m × {:.1}m (L×W×H)",
        load.dimensions.x, load.dimensions.y, load.dimensions.z
    );
    println!(
        "  CoG: [{:.2}, {:.2}, {:.2}]",
        load.center_of_gravity.x, load.center_of_gravity.y, load.center_of_gravity.z
    );
    println!("  Pick points: {}", load.pick_points.len());
}

fn print_rigging_analysis(analysis: &RiggingAnalysis) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║           RIGGING ANALYSIS                ║");
    println!("╠═══════════════════════════════════════════╣");

    let status = if analysis.safety_analysis.is_configuration_safe {
        "✓ SAFE"
    } else {
        "✗ UNSAFE"
    };

    let balance = if analysis.is_balanced {
        "✓ BALANCED"
    } else {
        "⚠ UNBALANCED"
    };

    println!("║ Status:       {:<30} ║", status);
    println!("║ Balance:      {:<30} ║", balance);
    println!("╠═══════════════════════════════════════════╣");
    println!("║             SLING TENSIONS                ║");
    println!("╠═══════════════════════════════════════════╣");

    for (i, tension) in analysis.sling_tensions.iter().enumerate() {
        let status_icon = if tension.is_safe { "✓" } else { "✗" };

        println!(
            "║ {} Sling {}: {:<6.0}kg at {:<4.1}° ({:<3.0}%) ║",
            status_icon,
            i + 1,
            tension.tension_kg,
            tension.angle_from_vertical_deg,
            tension.utilization_percent
        );
    }

    println!("╚═══════════════════════════════════════════╝");
}

fn print_ground_bearing_analysis(analysis: &crane_core::ground_bearing::GroundBearingAnalysis) {
    println!("\n╔═══════════════════════════════════════════╗");
    println!("║         GROUND BEARING ANALYSIS           ║");
    println!("╠═══════════════════════════════════════════╣");

    let status = if analysis.is_safe {
        "✓ SAFE"
    } else {
        "✗ UNSAFE"
    };

    println!("║ Status:       {:<30} ║", status);
    println!(
        "║ Soil Type:    {:<30} ║",
        format!("{:?}", analysis.soil_type)
    );
    println!("╠═══════════════════════════════════════════╣");
    println!("║          OUTRIGGER PRESSURES              ║");
    println!("╠═══════════════════════════════════════════╣");

    for (i, pressure) in analysis.bearing_pressures.iter().enumerate() {
        let status_icon = if pressure.is_safe { "✓" } else { "✗" };
        let pct = (pressure.pressure_kpa / pressure.allowable_kpa) * 100.0;

        println!(
            "║ {} Point {}: {:<6.0} kPa ({:<3.0}% of {:.0} kPa) ║",
            status_icon,
            i + 1,
            pressure.pressure_kpa,
            pct,
            pressure.allowable_kpa
        );
    }

    println!("╚═══════════════════════════════════════════╝");
}
