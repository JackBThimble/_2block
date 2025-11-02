// crates/scene_3d/src/load_renderer.rs - COMPLETE REWRITE

use crate::components::*;
use crate::coordinate_conversion::*;
use bevy::prelude::*;
use crane_core::rigging::Load;

/// Spawn a load at the origin (will be positioned by parent transform)
pub fn spawn_load(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    load_data: Load,
) -> Entity {
    spawn_load_at_position(commands, meshes, materials, load_data, Vec3::ZERO)
}

/// Spawn a load at a specific world position
pub fn spawn_load_at_position(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    load_data: Load,
    position: Vec3,
) -> Entity {
    println!("\n--- Spawning Load ---");

    // Create materials
    let load_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.5, 0.3),
        metallic: 0.2,
        perceptual_roughness: 0.8,
        reflectance: 0.3,
        ..default()
    });

    let pick_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.3, 1.0, 0.3),
        emissive: LinearRgba::rgb(0.0, 0.5, 0.0),
        metallic: 0.7,
        perceptual_roughness: 0.3,
        reflectance: 0.6,
        ..default()
    });

    let cog_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.3, 0.3),
        emissive: LinearRgba::rgb(0.5, 0.0, 0.0),
        metallic: 0.5,
        perceptual_roughness: 0.3,
        reflectance: 0.6,
        ..default()
    });

    // Convert dimensions from nalgebra (engineering) to Bevy (graphics)
    // nalgebra: x=length, y=width, z=height
    // Bevy cuboid: x=width, y=height, z=depth
    let dims_bevy = nalgebra_to_bevy_vector(load_data.dimensions);

    // Create load mesh
    let load_mesh = meshes.add(Cuboid::new(
        dims_bevy.x, // Width
        dims_bevy.y, // Height
        dims_bevy.z, // Depth
    ));

    // Spawn main load entity
    let load_entity = commands
        .spawn((
            Mesh3d(load_mesh),
            MeshMaterial3d(load_material),
            Transform::from_translation(position),
            LiftLoad {
                load_data: load_data.clone(),
                is_selected: false,
            },
            Name::new(format!("Load ({:.0}kg)", load_data.weight_kg)),
        ))
        .id();

    println!("✓ Load: {:.0}kg", load_data.weight_kg);
    println!(
        "  Dimensions: {:.1}m × {:.1}m × {:.1}m (L×W×H)",
        load_data.dimensions.x, load_data.dimensions.y, load_data.dimensions.z
    );
    println!(
        "  Position: [{:.1}, {:.1}, {:.1}]",
        position.x, position.y, position.z
    );

    // Spawn pick points
    let active_pick_points: Vec<_> = load_data
        .pick_points
        .iter()
        .filter(|pp| pp.active)
        .collect();

    for pick_point in &active_pick_points {
        spawn_pick_point(
            commands,
            meshes,
            pick_material.clone(),
            pick_point,
            load_entity,
        );
    }

    println!("  Pick points: {} active", active_pick_points.len());

    // Spawn center of gravity marker
    spawn_cog_marker(
        commands,
        meshes,
        materials,
        cog_material,
        load_data.center_of_gravity,
        load_entity,
    );
    // Add bounding box (wireframe outline)
    spawn_bounding_box(commands, meshes, materials, dims_bevy, load_entity);

    load_entity
}

/// Spawn a pick point as child of load
fn spawn_pick_point(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    material: Handle<StandardMaterial>,
    pick_point: &crane_core::rigging::PickPoint,
    parent: Entity,
) {
    // Convert pick point position from nalgebra to Bevy
    let pos_bevy = pick_point.position.to_bevy();

    // Create sphere mesh for pick point
    let pick_mesh = meshes.add(Sphere::new(0.25));

    let pick_entity = commands
        .spawn((
            Mesh3d(pick_mesh),
            MeshMaterial3d(material),
            Transform::from_translation(pos_bevy),
            PickPoint {
                id: pick_point.id.clone(),
                is_selected: false,
                is_hovered: false,
            },
            Name::new(format!("Pick Point: {}", pick_point.id)),
        ))
        .id();

    commands.entity(parent).add_child(pick_entity);
}

/// Spawn center of gravity marker
fn spawn_cog_marker(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>, // ADD materials param
    material: Handle<StandardMaterial>,
    cog: nalgebra::Point3<f32>,
    parent: Entity,
) {
    // Convert CoG position
    let cog_bevy = cog.to_bevy();

    // Create sphere for CoG
    let cog_mesh = meshes.add(Sphere::new(0.2));

    let cog_entity = commands
        .spawn((
            Mesh3d(cog_mesh),
            MeshMaterial3d(material),
            Transform::from_translation(cog_bevy),
            Name::new("Center of Gravity"),
        ))
        .id();

    commands.entity(parent).add_child(cog_entity);

    // Add axes at CoG to show orientation
    spawn_cog_axes(commands, meshes, materials, cog_entity); // Pass materials
}
/// Spawn XYZ axes at center of gravity
fn spawn_cog_axes(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    parent: Entity,
) {
    let axis_length = 0.5;
    let axis_radius = 0.02;

    // Create materials for each axis
    let x_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.0, 0.0), // Red
        emissive: LinearRgba::rgb(0.5, 0.0, 0.0),
        unlit: true,
        ..default()
    });

    let y_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 1.0, 0.0), // Green
        emissive: LinearRgba::rgb(0.0, 0.5, 0.0),
        unlit: true,
        ..default()
    });

    let z_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.0, 0.0, 1.0), // Blue
        emissive: LinearRgba::rgb(0.0, 0.0, 0.5),
        unlit: true,
        ..default()
    });

    // X axis (red) - side to side
    let x_mesh = meshes.add(Cylinder::new(axis_radius, axis_length));
    let x_entity = commands
        .spawn((
            Mesh3d(x_mesh),
            MeshMaterial3d(x_material),
            Transform::from_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(axis_length / 2.0, 0.0, 0.0)),
            Name::new("CoG X-Axis"),
        ))
        .id();
    commands.entity(parent).add_child(x_entity);

    // Y axis (green) - up/down
    let y_mesh = meshes.add(Cylinder::new(axis_radius, axis_length));
    let y_entity = commands
        .spawn((
            Mesh3d(y_mesh),
            MeshMaterial3d(y_material),
            Transform::from_translation(Vec3::new(0.0, axis_length / 2.0, 0.0)),
            Name::new("CoG Y-Axis"),
        ))
        .id();
    commands.entity(parent).add_child(y_entity);

    // Z axis (blue) - forward/back
    let z_mesh = meshes.add(Cylinder::new(axis_radius, axis_length));
    let z_entity = commands
        .spawn((
            Mesh3d(z_mesh),
            MeshMaterial3d(z_material),
            Transform::from_rotation(Quat::from_rotation_x(std::f32::consts::FRAC_PI_2))
                .with_translation(Vec3::new(0.0, 0.0, axis_length / 2.0)),
            Name::new("CoG Z-Axis"),
        ))
        .id();
    commands.entity(parent).add_child(z_entity);
}
/// Spawn wireframe bounding box around load
fn spawn_bounding_box(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    dimensions: Vec3,
    parent: Entity,
) {
    let wireframe_material = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 1.0, 1.0, 0.3),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    });

    let edge_radius = 0.02;

    let hx = dimensions.x / 2.0;
    let hy = dimensions.y / 2.0;
    let hz = dimensions.z / 2.0;

    // Define the 8 corners of the bounding box
    let corners = [
        Vec3::new(-hx, -hy, -hz), // 0: bottom-back-left
        Vec3::new(hx, -hy, -hz),  // 1: bottom-back-right
        Vec3::new(-hx, -hy, hz),  // 2: bottom-front-left
        Vec3::new(hx, -hy, hz),   // 3: bottom-front-right
        Vec3::new(-hx, hy, -hz),  // 4: top-back-left
        Vec3::new(hx, hy, -hz),   // 5: top-back-right
        Vec3::new(-hx, hy, hz),   // 6: top-front-left
        Vec3::new(hx, hy, hz),    // 7: top-front-right
    ];

    let edges = [
        (0, 1),
        (1, 3),
        (3, 2),
        (2, 0),
        (4, 5),
        (5, 7),
        (7, 6),
        (6, 4),
        (0, 4),
        (1, 5),
        (2, 6),
        (3, 7),
    ];

    // Create 12 edges of the bounding box
    for (start_idx, end_idx) in edges {
        let start = corners[start_idx];
        let end = corners[end_idx];
        let direction = end - start;
        let length = direction.length();
        if length < 0.01 {
            continue;
        }

        let midpoint = (start + end) / 2.0;
        let edge_mesh = meshes.add(Cylinder::new(edge_radius, length));

        let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

        let edge_entity = commands
            .spawn((
                Mesh3d(edge_mesh),
                MeshMaterial3d(wireframe_material.clone()),
                Transform::from_translation(midpoint).with_rotation(rotation),
                Name::new("Bounding Box Edge"),
            ))
            .id();

        commands.entity(parent).add_child(edge_entity);
    }
}

/// Update load visual when load data changes
pub fn update_load_visual_system(
    load_query: Query<(&LiftLoad, &Children), Changed<LiftLoad>>,
    mut transform_query: Query<&mut Transform>,
    pick_query: Query<&PickPoint>,
) {
    for (load, children) in load_query.iter() {
        // Update pick point positions if load data changed
        for child in children.iter() {
            if let Ok(pick_point) = pick_query.get(child) {
                // Find corresponding pick point in load data
                if let Some(load_pick) = load
                    .load_data
                    .pick_points
                    .iter()
                    .find(|pp| pp.id == pick_point.id)
                    && let Ok(mut transform) = transform_query.get_mut(child)
                {
                    transform.translation = load_pick.position.to_bevy();
                }
            }
        }
    }
}

/// Highlight selected load
pub fn highlight_selected_loads_system(
    mut load_query: Query<(&LiftLoad, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (load, material_handle) in load_query.iter_mut() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if load.is_selected {
                // Highlight selected load
                material.emissive = LinearRgba::rgb(0.2, 0.2, 0.0);
            } else {
                // Normal appearance
                material.emissive = LinearRgba::BLACK;
            }
        }
    }
}

/// Highlight hovered pick points
pub fn highlight_pick_points_system(
    pick_query: Query<(&PickPoint, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (pick_point, material_handle) in pick_query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            if pick_point.is_selected {
                // Selected pick point (bright green)
                material.emissive = LinearRgba::rgb(0.0, 1.0, 0.0);
                material.base_color = Color::srgb(0.5, 1.0, 0.5);
            } else if pick_point.is_hovered {
                // Hovered pick point (yellow)
                material.emissive = LinearRgba::rgb(0.5, 0.5, 0.0);
                material.base_color = Color::srgb(0.8, 1.0, 0.3);
            } else {
                // Normal pick point (green)
                material.emissive = LinearRgba::rgb(0.0, 0.5, 0.0);
                material.base_color = Color::srgb(0.3, 1.0, 0.3);
            }
        }
    }
}

/// Create a test load with realistic properties
pub fn create_test_load() -> Load {
    Load {
        weight_kg: 8000.0,
        center_of_gravity: nalgebra::Point3::new(0.0, 0.0, 0.6), // Slightly off-center in height
        dimensions: nalgebra::Vector3::new(5.0, 2.5, 1.2),       // L, W, H
        pick_points: vec![
            crane_core::rigging::PickPoint {
                id: "front_left".to_string(),
                position: nalgebra::Point3::new(2.0, -1.0, 1.2), // Front left top corner
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "front_right".to_string(),
                position: nalgebra::Point3::new(2.0, 1.0, 1.2), // Front right top corner
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "rear_left".to_string(),
                position: nalgebra::Point3::new(-2.0, -1.0, 1.2), // Rear left top corner
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "rear_right".to_string(),
                position: nalgebra::Point3::new(-2.0, 1.0, 1.2), // Rear right top corner
                active: true,
            },
        ],
    }
}

/// Create load with custom properties
pub fn create_custom_load(
    weight_kg: f32,
    length_m: f32,
    width_m: f32,
    height_m: f32,
    num_pick_points: usize,
) -> Load {
    let half_length = length_m / 2.0;
    let half_width = width_m / 2.0;

    // Generate pick points at corners
    let pick_points = match num_pick_points {
        2 => vec![
            crane_core::rigging::PickPoint {
                id: "front".to_string(),
                position: nalgebra::Point3::new(half_length, 0.0, height_m),
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "rear".to_string(),
                position: nalgebra::Point3::new(-half_length, 0.0, height_m),
                active: true,
            },
        ],
        4 => vec![
            crane_core::rigging::PickPoint {
                id: "front_left".to_string(),
                position: nalgebra::Point3::new(half_length, -half_width, height_m),
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "front_right".to_string(),
                position: nalgebra::Point3::new(half_length, half_width, height_m),
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "rear_left".to_string(),
                position: nalgebra::Point3::new(-half_length, -half_width, height_m),
                active: true,
            },
            crane_core::rigging::PickPoint {
                id: "rear_right".to_string(),
                position: nalgebra::Point3::new(-half_length, half_width, height_m),
                active: true,
            },
        ],
        _ => vec![],
    };

    Load {
        weight_kg,
        center_of_gravity: nalgebra::Point3::new(0.0, 0.0, height_m / 2.0),
        dimensions: nalgebra::Vector3::new(length_m, width_m, height_m),
        pick_points,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_test_load() {
        let load = create_test_load();

        assert_eq!(load.weight_kg, 8000.0);
        assert_eq!(load.pick_points.len(), 4);
        assert!(load.pick_points.iter().all(|pp| pp.active));
    }

    #[test]
    fn test_create_custom_load_2_points() {
        let load = create_custom_load(5000.0, 4.0, 2.0, 1.0, 2);

        assert_eq!(load.weight_kg, 5000.0);
        assert_eq!(load.pick_points.len(), 2);
        assert_eq!(load.pick_points[0].id, "front");
        assert_eq!(load.pick_points[1].id, "rear");
    }

    #[test]
    fn test_create_custom_load_4_points() {
        let load = create_custom_load(10000.0, 6.0, 3.0, 2.0, 4);

        assert_eq!(load.weight_kg, 10000.0);
        assert_eq!(load.pick_points.len(), 4);

        // Check corners are at correct positions
        let half_length = 3.0;
        let half_width = 1.5;
        let height = 2.0;

        assert_eq!(load.pick_points[0].position.x, half_length);
        assert_eq!(load.pick_points[0].position.y, -half_width);
        assert_eq!(load.pick_points[0].position.z, height);
    }
}
