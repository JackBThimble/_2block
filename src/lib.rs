// crates/lift_planner/src/main.rs

use bevy::prelude::*;
use scene_3d::Scene3DPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(Scene3DPlugin)
        .run();
}
