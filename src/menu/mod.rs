//! MineRust menu, HUD, settings, benchmark controls, and UI subsystems.

pub mod descriptions;
pub mod interactions;
pub mod seed;
pub mod sync;
pub mod types;
pub mod ui;
pub mod widgets;

#[cfg(test)]
mod tests;

use bevy::prelude::*;

pub use descriptions::*;
pub use interactions::*;
pub use seed::*;
pub use sync::*;
pub use types::*;
pub use ui::*;
pub use widgets::*;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<GraphicsSettings>() {
            app.insert_resource(GraphicsSettings::load_or_default());
        }
        app.init_resource::<MenuState>()
            .init_resource::<DevSettings>()
            .init_resource::<FpsLimiter>()
            .init_resource::<SeedInputState>()
            .add_systems(Startup, setup_menu_ui)
            .add_systems(
                Update,
                (
                    menu_input_system.in_set(crate::stage::VoxelStage::InputHandling),
                    update_seed_input_system.in_set(crate::stage::VoxelStage::InputHandling),
                    update_menu_visibility_system,
                    menu_button_hover_system,
                    menu_button_click_system,
                    slider_interaction_system,
                    update_slider_visuals_system,
                    update_settings_button_text_system,
                    update_dev_button_text_system,
                    update_dev_settings_system,
                    update_option_tooltip_system,
                    auto_save_graphics_settings_system,
                    fps_limiter_system,
                ),
            );
    }
}
