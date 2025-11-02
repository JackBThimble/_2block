use crate::components::*;
use crate::coordinate_conversion::CoordinateConversion;
use bevy::prelude::*;
use crane_core::rigging::Sling;

// pub fn spawn_slings(
//     commands: &mut Commands,
//     meshes: &mut ResMut<Assets<Mesh>>,
//     materials: &mut ResMut<Assets<StandardMaterial>>,
//     slings: &[Sling],
//     rigging_analysis: &crane_core::rigging::RiggingAnalysis,
// ) {
//     for (sling, analysis) in slings.iter().zip(rigging_analysis.sling_tensions.iter()) {
//         spawn_sling(commands, meshes, materials, sling, analysis);
//     }
// }

pub(crate) fn spawn_sling(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    sling: &Sling,
    tension: &crane_core::rigging::SlingTensionAnalysis,
    load_pos_bevy: Vec3,
    hook_pos_bevy: Vec3,
    index: usize,
) {
    let attach_local_bevy = Vec3::new(
        sling.attachment_point.x,
        sling.attachment_point.z,
        sling.attachment_point.y,
    );
    let start = load_pos_bevy + attach_local_bevy;
    let end = hook_pos_bevy;

    let direction = end - start;
    let length = direction.length();

    if length < 0.1 {
        eprintln!("⚠ Sling {} too short: {:.3}m", index, length);
        return;
    }

    let midpoint = (start + end) / 2.0;
    let color = get_tension_color(tension.utilization_percent);

    let mat = materials.add(StandardMaterial {
        base_color: color,
        emissive: LinearRgba::from(color) * 0.6,
        metallic: 0.7,
        perceptual_roughness: 0.3,
        ..default()
    });

    let rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());
    let mesh = meshes.add(Cylinder::new(0.1, length));

    commands.spawn((
        Mesh3d(mesh),
        MeshMaterial3d(mat),
        Transform::from_translation(midpoint).with_rotation(rotation),
        Visibility::default(),
        SlingComponent {
            sling_data: sling.clone(),
            tension_kg: tension.tension_kg,
            is_safe: tension.is_safe,
        },
        Name::new(format!("Sling_{}", sling.spec.id)),
    ));
}

fn get_tension_color(utilization_percent: f32) -> Color {
    if utilization_percent < 50.0 {
        let t = utilization_percent / 50.0;
        Color::srgb(t, 1.0, 0.0)
    } else if utilization_percent < 90.0 {
        let t = (utilization_percent - 50.0) / 40.0;
        Color::srgb(1.0, 1.0 - (t * 0.5), 0.0)
    } else if utilization_percent < 100.0 {
        let t = (utilization_percent - 90.0) / 10.0;
        Color::srgb(1.0, 0.5 - (t * 0.5), 0.0)
    } else {
        Color::srgb(1.0, 0.0, 0.0)
    }
}

pub fn update_sling_colors(
    sling_query: Query<(&SlingComponent, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (sling, material_handle) in sling_query.iter() {
        if let Some(material) = materials.get_mut(&material_handle.0) {
            let utilization_percent = if sling.sling_data.spec.rated_capacity_kg > 0.0 {
                (sling.tension_kg / sling.sling_data.spec.rated_capacity_kg) * 100.0
            } else {
                0.0
            };

            let color = get_tension_color(utilization_percent);
            material.base_color = color;
            material.emissive = LinearRgba::from(color) * 0.6;
        }
    }
}

pub fn update_sling_geometry_system(
    mut sling_query: Query<(Entity, &SlingComponent, &mut Transform, &Mesh3d)>,
    load_query: Query<(&LiftLoad, &Transform), Without<SlingComponent>>,
    crane_query: Query<&Crane>,
) {
    let hook_pos_bevy = if let Ok(crane) = crane_query.single() {
        let hook_pos = crane.config.get_hook_position();
        hook_pos.to_bevy()
    } else {
        return;
    };

    let load_transform = if let Ok((_, transform)) = load_query.single() {
        transform
    } else {
        return;
    };

    let load_pos_bevy = load_transform.translation;

    for (_, sling_component, mut transform, _) in sling_query.iter_mut() {
        let attach_local_bevy = sling_component.sling_data.attachment_point.to_bevy();
        let start = load_pos_bevy + attach_local_bevy;
        let end = hook_pos_bevy;
        let direction = end - start;
        let length = direction.length();

        if length < 0.1 {
            continue;
        }

        // update transform
        let midpoint = (start + end) / 2.0;
        transform.translation = midpoint;
        transform.rotation = Quat::from_rotation_arc(Vec3::Y, direction.normalize());

        // update mesh length if it changed significantly
        // Note, this is expensive, only do so if length changed by more than 10cm
        // in practice, might want to update this less frequently or only when needed
    }
}

pub fn get_tension_gradient(utilization_percent: f32) -> LinearRgba {
    let color = get_tension_color(utilization_percent);
    LinearRgba::from(color)
}

pub fn show_tension_labels_system(
    mut commands: Commands,
    sling_query: Query<(&SlingComponent, &Transform)>,
    asset_server: Res<AssetServer>,
) {
    for (sling, transform) in sling_query.iter() {
        let utilization_percent = if sling.sling_data.spec.rated_capacity_kg > 0.0 {
            (sling.tension_kg / sling.sling_data.spec.rated_capacity_kg) * 100.0
        } else {
            0.0
        };

        let text = format!("{:.0}kg\n{:.0}%", sling.tension_kg, utilization_percent);

        // Spawn 3D text at sling midpoint
        // (Implementation depends on text rendering solution)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tension_colors() {
        // Safe - green
        let color = get_tension_color(40.0);
        assert_eq!(color, Color::srgb(0.2, 1.0, 0.2));

        // Caution - yellow
        let color = get_tension_color(60.0);
        assert_eq!(color, Color::srgb(1.0, 1.0, 0.2));

        // Warning - orange
        let color = get_tension_color(85.0);
        assert_eq!(color, Color::srgb(1.0, 0.6, 0.0));

        // Danger - dark orange
        let color = get_tension_color(95.0);
        assert_eq!(color, Color::srgb(1.0, 0.3, 0.0));

        // Unsafe - red
        let color = get_tension_color(105.0);
        assert_eq!(color, Color::srgb(1.0, 0.0, 0.0));
    }
}
