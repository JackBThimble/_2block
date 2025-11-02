use bevy::{prelude::*, window::WindowResolution};
use scene_3d::Scene3DPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "2Block Lift Planner".into(),
                    resolution: WindowResolution::new(1920, 1080),
                    present_mode: bevy::window::PresentMode::AutoVsync,
                    ..default()
                }),
                ..default()
            }),
            Scene3DPlugin,
        ))
        .run();
}
