use bevy::{prelude::*, window::WindowResolution};
use scene_3d::Scene3DPlugin;
use ui_layer::UiLayerPlugin;

fn main() {
    App::new()
        .add_plugins((DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "2Block Lift Planner".into(),
                resolution: WindowResolution::new(1280, 800),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }),))
        .add_plugins(Scene3DPlugin)
        .add_plugins(UiLayerPlugin)
        .run();
}
