use std::collections::BTreeMap;

use bevy::prelude::*;
use bevy_egui::{
    EguiContext, EguiContexts, EguiPlugin, EguiPrimaryContextPass, PrimaryEguiContext,
    egui::{
        self, FontData, FontDefinitions, FontFamily,
        epaint::text::{FontInsert, InsertFontFamily},
    },
};

#[derive(Resource, Default)]
pub struct UiState {
    config: bool,
}

pub struct UiLayerPlugin;
const MENU_ICON: char = '\u{E5D2}';
const MATERIAL_SYMBOLS_REL_PATH: &str =
    "ui/fonts/MaterialSymbolsRounded[FILL,GRAD,opsz,wght] (1).ttf";
const MATERIAL_SYMBOLS_FONT_NAME: &str = "MaterialSymbolsRounded[FILL,GRAD,opsz,wght] (1).ttf";

impl Plugin for UiLayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>();
        app.add_plugins(EguiPlugin {
            ..Default::default()
        });
        app.add_systems(Startup, configure_egui_fonts);
        app.add_systems(EguiPrimaryContextPass, ui_system);
    }
}

fn configure_egui_fonts(mut contexts: EguiContexts) {
    let ctx = match contexts.ctx_mut() {
        Ok(ctx) => ctx,
        Err(_) => {
            println!("Fuck!!!!\n");
            return;
        }
    };

    ctx.add_font(FontInsert::new(
        "MaterialSymbols",
        egui::FontData::from_static(include_bytes!(
            "../assets/fonts/MaterialSymbolsRounded[FILL,GRAD,opsz,wght] (1).ttf"
        )),
        vec![
            InsertFontFamily {
                family: egui::FontFamily::Proportional,
                priority: egui::epaint::text::FontPriority::Highest,
            },
            InsertFontFamily {
                family: egui::FontFamily::Monospace,
                priority: egui::epaint::text::FontPriority::Lowest,
            },
        ],
    ));
}

fn ui_system(mut contexts: EguiContexts, state: ResMut<UiState>) -> Result {
    let ctx = contexts.ctx_mut().unwrap();

    egui::Window::new("Main Menu")
        .anchor(egui::Align2::LEFT_TOP, egui::Vec2::new(10.0, 10.0))
        .collapsible(false)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new(String::from(MENU_ICON))
                    .font(egui::FontId::new(
                        48.0,
                        FontFamily::Name("MaterialSymbols".into()),
                    ))
                    .color(egui::Color32::DARK_BLUE),
            );
        });
    Ok(())
}
