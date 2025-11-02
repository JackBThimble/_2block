use crate::components::*;
use bevy::prelude::*;
use crane_core::*;

pub fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Ground plane (Bevy 0.17: Plane 3d is now a primitive)
    let ground_size = 100.0;
    let ground_mesh = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(ground_size / 2.0)));
    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.4, 0.5, 0.4),
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, 0.0, 0.0),
        Visibility::default(),
        GroundPlane {
            soil_type: ground_bearing::SoilType::MediumClay,
        },
        Name::new("Ground"),
    ));

    spawn_grid(&mut commands, &mut meshes, &mut materials, ground_size, 5.0);

    commands.spawn((
        DirectionalLight {
            illuminance: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.7, 0.5, 0.0)),
        Visibility::default(),
        Name::new("Sun"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 300.0,
        affects_lightmapped_meshes: true,
    });

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(20.0, 15.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
        Visibility::default(),
        MainCamera,
        Name::new("MainCamera"),
    ));
}

fn spawn_grid(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    size: f32,
    spacing: f32,
) {
    let grid_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.3, 0.3, 0.3, 0.5),
        unlit: true,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    let num_lines = (size / spacing) as i32;

    for i in -num_lines..=num_lines {
        let pos = i as f32 * spacing;

        // lines parallel to x axis
        let mesh_x = meshes.add(Cuboid::new(size, 0.02, 0.02));
        commands.spawn((
            Mesh3d(mesh_x),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(0.0, 0.01, pos),
            Visibility::default(),
            Name::new(format!("GridLine_X_{}", i)),
        ));

        // lines parallel to z axis
        let mesh_z = meshes.add(Cuboid::new(0.02, 0.02, size));
        commands.spawn((
            Mesh3d(mesh_z),
            MeshMaterial3d(grid_material.clone()),
            Transform::from_xyz(pos, 0.01, 0.0),
            Visibility::default(),
            Name::new(format!("GridLine_Z_{}", i)),
        ));
    }
}
