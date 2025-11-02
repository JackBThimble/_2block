// crates/scene_3d/src/crane_renderer.rs - COMPLETE REWRITE

use crate::components::*;
use bevy::prelude::*;
use crane_core::CraneConfiguration;

/// Spawn a complete crane from a CraneConfiguration
pub fn spawn_crane(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    crane_config: CraneConfiguration,
) -> Entity {
    println!("\n=== SPAWNING CRANE ===");
    println!(
        "Model: {} {}",
        crane_config.spec.manufacturer, crane_config.spec.model
    );
    println!("Type: {}", crane_config.spec.crane_type);
    println!("Capacity: {:.0} kg", crane_config.spec.max_capacity_kg);
    println!(
        "Boom: {:.1}m - {:.1}m",
        crane_config.spec.boom_length_range.0, crane_config.spec.boom_length_range.1
    );

    // Create materials
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.8, 0.2),
        metallic: 0.3,
        perceptual_roughness: 0.5,
        reflectance: 0.5,
        ..default()
    });

    let boom_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.9, 0.9),
        metallic: 0.8,
        perceptual_roughness: 0.3,
        reflectance: 0.6,
        ..default()
    });

    let cable_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.15, 0.15, 0.15),
        metallic: 0.8,
        perceptual_roughness: 0.2,
        reflectance: 0.7,
        ..default()
    });

    let hook_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.2, 0.2, 0.2),
        metallic: 0.9,
        perceptual_roughness: 0.2,
        reflectance: 0.8,
        ..default()
    });

    let outrigger_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 0.3, 0.3),
        metallic: 0.7,
        perceptual_roughness: 0.4,
        reflectance: 0.5,
        ..default()
    });

    let counterweight_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.5, 0.5, 0.5),
        metallic: 0.5,
        perceptual_roughness: 0.6,
        reflectance: 0.4,
        ..default()
    });

    // Convert crane position from nalgebra to Bevy
    let crane_pos_bevy = Vec3::new(
        crane_config.position.x,
        crane_config.position.z,
        crane_config.position.y,
    );

    // Root crane entity
    let crane_entity = commands
        .spawn((
            Transform::from_translation(crane_pos_bevy),
            Crane {
                config: crane_config.clone(),
            },
            Name::new(format!(
                "{} {}",
                crane_config.spec.manufacturer, crane_config.spec.model
            )),
        ))
        .id();

    // Crane body - height based on boom pivot
    let body_height = crane_config.spec.boom_pivot_height_m * 1.25;
    let body_mesh = meshes.add(Cuboid::new(
        crane_config.spec.width_m,
        body_height,
        crane_config.spec.length_m,
    ));

    let body_entity = commands
        .spawn((
            Mesh3d(body_mesh),
            MeshMaterial3d(body_material),
            Transform::from_xyz(0.0, body_height / 2.0, 0.0),
            CraneVisualPart::Body,
            Name::new("Crane Body"),
        ))
        .id();
    commands.entity(crane_entity).add_child(body_entity);

    println!(
        "✓ Body: {:.1}m × {:.1}m × {:.1}m",
        crane_config.spec.width_m, body_height, crane_config.spec.length_m
    );

    // Boom
    let boom_pivot_height = crane_config.spec.boom_pivot_height_m;
    let boom_angle_rad = crane_config.boom_angle_deg.to_radians();
    let boom_center_y =
        boom_pivot_height + (crane_config.boom_length_m / 2.0) * boom_angle_rad.sin();
    let boom_center_z = (crane_config.boom_length_m / 2.0) * boom_angle_rad.cos();

    let boom_mesh = meshes.add(Cylinder::new(0.4, crane_config.boom_length_m));
    let boom_entity = commands
        .spawn((
            Mesh3d(boom_mesh),
            MeshMaterial3d(boom_material),
            Transform::from_xyz(0.0, boom_center_y, boom_center_z).with_rotation(
                Quat::from_rotation_x(std::f32::consts::FRAC_PI_2 - boom_angle_rad),
            ),
            CraneVisualPart::Boom,
            Name::new("Boom"),
        ))
        .id();
    commands.entity(crane_entity).add_child(boom_entity);

    println!(
        "✓ Boom: {:.1}m at {:.0}° (pivot at {:.1}m)",
        crane_config.boom_length_m, crane_config.boom_angle_deg, boom_pivot_height
    );

    // Get boom tip and hook positions using crane_core
    let boom_tip = crane_config.get_boom_tip_position();
    let hook_pos = crane_config.get_hook_position();

    let boom_tip_bevy = Vec3::new(boom_tip.x, boom_tip.z, boom_tip.y);
    let hook_pos_bevy = Vec3::new(hook_pos.x, hook_pos.z, hook_pos.y);

    println!(
        "✓ Boom tip: [{:.1}, {:.1}, {:.1}]",
        boom_tip_bevy.x, boom_tip_bevy.y, boom_tip_bevy.z
    );

    // Hoist cable
    let cable_direction = hook_pos_bevy - boom_tip_bevy;
    let cable_length = cable_direction.length();

    if cable_length > 0.1 {
        let cable_midpoint = (boom_tip_bevy + hook_pos_bevy) / 2.0;
        let cable_mesh = meshes.add(Cylinder::new(0.05, cable_length));
        let cable_rotation = Quat::from_rotation_arc(Vec3::Y, cable_direction.normalize());

        let cable_entity = commands
            .spawn((
                Mesh3d(cable_mesh),
                MeshMaterial3d(cable_material),
                Transform::from_translation(cable_midpoint).with_rotation(cable_rotation),
                CraneVisualPart::Cable,
                Name::new("Hoist Cable"),
            ))
            .id();

        commands.entity(crane_entity).add_child(cable_entity);

        println!(
            "✓ Cable: {:.1}m (range: {:.1}m - {:.1}m)",
            cable_length,
            crane_config.spec.hoist_length_range.0,
            crane_config.spec.hoist_length_range.1
        );
    }

    // Hook
    let hook_mesh = meshes.add(Sphere::new(0.5));
    let hook_entity = commands
        .spawn((
            Mesh3d(hook_mesh),
            MeshMaterial3d(hook_material),
            Transform::from_translation(hook_pos_bevy),
            CraneVisualPart::Hook,
            Name::new("Hook"),
        ))
        .id();
    commands.entity(crane_entity).add_child(hook_entity);

    println!(
        "✓ Hook: [{:.1}, {:.1}, {:.1}]",
        hook_pos_bevy.x, hook_pos_bevy.y, hook_pos_bevy.z
    );

    // Outriggers (if deployed)
    let deployed_count = crane_config
        .outriggers
        .outriggers
        .iter()
        .filter(|o| o.is_deployed())
        .count();

    if deployed_count > 0 {
        for outrigger in &crane_config.outriggers.outriggers {
            if outrigger.is_deployed() {
                spawn_outrigger(
                    commands,
                    meshes,
                    outrigger_material.clone(),
                    outrigger,
                    &crane_config.outriggers,
                    crane_entity,
                );
            }
        }
        println!(
            "✓ Outriggers: {} deployed at {:.1}m extension",
            deployed_count, crane_config.outriggers.outriggers[0].extension_m
        );
    }

    // Counterweight slabs
    let cw_count = crane_config.counterweight.get_slab_count();
    if cw_count > 0 {
        spawn_counterweight(
            commands,
            meshes,
            counterweight_material,
            &crane_config.counterweight,
            crane_entity,
        );

        println!(
            "✓ Counterweight: {} × {:.0}kg = {:.0}kg total",
            cw_count,
            crane_config.spec.counterweight_slab_weight_kg,
            crane_config.counterweight.get_total_weight_kg()
        );
    }

    // Total crane weight
    println!(
        "✓ Total weight: {:.0}kg (base) + {:.0}kg (counterweight) = {:.0}kg",
        crane_config.spec.base_weight_kg,
        crane_config.counterweight.get_total_weight_kg(),
        crane_config.get_total_weight_kg()
    );

    println!("=== CRANE SPAWN COMPLETE ===\n");
    crane_entity
}

fn spawn_outrigger(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    outrigger: &crane_core::OutriggerConfig,
    system: &crane_core::OutriggerSystem,
    parent: Entity,
) {
    // Get contact point in crane local space
    let contact_point = outrigger.get_contact_point(system.crane_base_width_m);

    // Convert to Bevy coordinates
    let pos = Vec3::new(contact_point.x, 0.1, contact_point.y);

    // Outrigger pad
    let pad_radius = outrigger.pad_diameter_m.unwrap_or(0.6) / 2.0;
    let pad_mesh = meshes.add(Cylinder::new(pad_radius, 0.2));

    let outrigger_entity = commands
        .spawn((
            Mesh3d(pad_mesh),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(pos),
            OutriggerVisual {
                position: outrigger.position,
                pressure_kpa: 0.0,
            },
            Name::new(format!("Outrigger {:?}", outrigger.position)),
        ))
        .id();

    commands.entity(parent).add_child(outrigger_entity);

    // Outrigger beam (horizontal extension)
    let beam_length = (contact_point.x.powi(2) + contact_point.y.powi(2)).sqrt();
    let beam_mesh = meshes.add(Cuboid::new(0.3, 0.3, beam_length));

    let beam_angle = contact_point.y.atan2(contact_point.x);
    let beam_pos = Vec3::new(contact_point.x / 2.0, 2.0, contact_point.y / 2.0);

    let beam_entity = commands
        .spawn((
            Mesh3d(beam_mesh),
            MeshMaterial3d(material),
            Transform::from_translation(beam_pos).with_rotation(Quat::from_rotation_y(beam_angle)),
            Name::new(format!("Outrigger Beam {:?}", outrigger.position)),
        ))
        .id();

    commands.entity(parent).add_child(beam_entity);
}

fn spawn_counterweight(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    counterweight: &crane_core::CounterweightConfig,
    parent: Entity,
) {
    let slab_height = 0.5;
    let slab_width = 2.5;
    let slab_depth = 2.0;

    for (i, _slab) in counterweight.slabs.iter().enumerate() {
        let slab_mesh = meshes.add(Cuboid::new(slab_width, slab_height, slab_depth));

        // Stack slabs vertically behind crane
        let pos = Vec3::new(
            0.0,
            0.25 + i as f32 * slab_height,
            -counterweight.moment_arm_m,
        );

        let slab_entity = commands
            .spawn((
                Mesh3d(slab_mesh),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(pos),
                CraneVisualPart::CounterweightSlab { index: i },
                Name::new(format!("Counterweight Slab {}", i + 1)),
            ))
            .id();

        commands.entity(parent).add_child(slab_entity);
    }
}

/// System that updates crane visuals when configuration changes
pub fn update_crane_visuals_system(
    crane_query: Query<(&Crane, &Children), Changed<Crane>>,
    mut part_query: Query<(&mut Transform, &CraneVisualPart)>,
) {
    for (crane, children) in crane_query.iter() {
        let config = &crane.config;

        let boom_tip = config.get_boom_tip_position();
        let hook_pos = config.get_hook_position();

        let boom_tip_bevy = Vec3::new(boom_tip.x, boom_tip.z, boom_tip.y);
        let hook_pos_bevy = Vec3::new(hook_pos.x, hook_pos.z, hook_pos.y);

        let boom_pivot_height = config.spec.boom_pivot_height_m;
        let boom_angle_rad = config.boom_angle_deg.to_radians();
        let boom_center_y = boom_pivot_height + (config.boom_length_m / 2.0) * boom_angle_rad.sin();
        let boom_center_z = (config.boom_length_m / 2.0) * boom_angle_rad.cos();

        for child in children.iter() {
            if let Ok((mut transform, part)) = part_query.get_mut(child) {
                match part {
                    CraneVisualPart::Boom => {
                        transform.translation = Vec3::new(0.0, boom_center_y, boom_center_z);
                        transform.rotation =
                            Quat::from_rotation_x(std::f32::consts::FRAC_PI_2 - boom_angle_rad);
                    }
                    CraneVisualPart::Cable => {
                        let cable_dir = hook_pos_bevy - boom_tip_bevy;
                        let cable_midpoint = (boom_tip_bevy + hook_pos_bevy) / 2.0;

                        transform.translation = cable_midpoint;
                        transform.rotation =
                            Quat::from_rotation_arc(Vec3::Y, cable_dir.normalize());
                    }
                    CraneVisualPart::Hook => {
                        transform.translation = hook_pos_bevy;
                    }
                    _ => {}
                }
            }
        }
    }
}
