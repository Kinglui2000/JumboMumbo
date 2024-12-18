use std::collections::BTreeMap;
use macroquad::color::Color;
use macroquad::input::MouseButton;
use macroquad::math::vec2;
use macroquad::prelude::{is_key_pressed, screen_height, screen_width, KeyCode, Texture2D};
use stopwatch2::Stopwatch;
use crate::logic::level::Level;
use crate::utils::enums::{Scene, SceneTextureKey, TextureKey};
use crate::ui::buttons::Button;
use crate::utils::structs::Settings;
use crate::utils::text::draw_text_centered;
use crate::utils::texture::{get_texture_path, load_textures_from_tile_map};

pub async fn level_selector(scene: &mut Scene, textures: &mut BTreeMap<SceneTextureKey, BTreeMap<TextureKey, Vec<Texture2D>>>, settings: &Settings) {
    if !textures.contains_key(&SceneTextureKey::LevelSelector) {
        textures.insert(SceneTextureKey::LevelSelector, load_textures().await);
    }

    if is_key_pressed(KeyCode::Escape) {
        *scene = Scene::MainMenu;
        textures.remove(&SceneTextureKey::LevelSelector);
        return;
    }

    draw_text_centered("Level Selector", screen_height() / 8.0, screen_height() / 8.0, Color::from_rgba(255, 255, 255, 255)).await;

    {
        let level_0_button = Button::new(
            vec2(screen_width() / 2.0 - (400.0 * settings.gui_scale) / 2.0, screen_height() / 2.0 - (200.0 * settings.gui_scale) / 2.0),
            vec2(400.0, 200.0) * settings.gui_scale,
            vec2(64.0, 64.0) * settings.gui_scale,
            "Tutorial".to_string(),
            64.0 * settings.gui_scale,
            TextureKey::Button0,
        ).await;
        level_0_button.render(textures.get(&SceneTextureKey::LevelSelector).unwrap()).await;
        if level_0_button.is_released(MouseButton::Left).await {
            *scene = Scene::Level(Level::Level0);
            textures.remove(&SceneTextureKey::LevelSelector);
            return;
        }

        let level_1_button = Button::new(
            vec2(screen_width() / 2.0 - (400.0 * settings.gui_scale) / 2.0, screen_height() / 2.0 + (200.0 * settings.gui_scale)),
            vec2(400.0, 200.0) * settings.gui_scale,
            vec2(64.0, 64.0) * settings.gui_scale,
            "Level 1".to_string(),
            64.0 * settings.gui_scale,
            TextureKey::Button0,
        ).await;
        level_1_button.render(textures.get(&SceneTextureKey::LevelSelector).unwrap()).await;
        if level_1_button.is_released(MouseButton::Left).await {
            *scene = Scene::Level(Level::Level1);
            textures.remove(&SceneTextureKey::LevelSelector);
            return;
        }

        let level_2_button = Button::new(
            vec2(screen_width() / 2.0 - (400.0 * settings.gui_scale) / 2.0, screen_height() / 2.0 + (500.0 * settings.gui_scale)),
            vec2(400.0, 200.0) * settings.gui_scale,
            vec2(64.0, 64.0) * settings.gui_scale,
            "Level 2".to_string(),
            64.0 * settings.gui_scale,
            TextureKey::Button0,
        ).await;
        level_2_button.render(textures.get(&SceneTextureKey::LevelSelector).unwrap()).await;
        if level_2_button.is_released(MouseButton::Left).await {
            *scene = Scene::Level(Level::Level2);
            textures.remove(&SceneTextureKey::LevelSelector);
            return;
        }

        let level_3_button = Button::new(
            vec2(screen_width() / 2.0 - (400.0 * settings.gui_scale) / 2.0, screen_height() / 2.0 + (500.0 * settings.gui_scale)),
            vec2(400.0, 200.0) * settings.gui_scale,
            vec2(64.0, 64.0) * settings.gui_scale,
            "Level 2".to_string(),
            64.0 * settings.gui_scale,
            TextureKey::Button0,
        ).await;
        level_3_button.render(textures.get(&SceneTextureKey::LevelSelector).unwrap()).await;
        if level_3_button.is_released(MouseButton::Left).await {
            *scene = Scene::Level(Level::Level3);
            textures.remove(&SceneTextureKey::LevelSelector);
            return;
        }
    }
}

async fn load_textures() -> BTreeMap<TextureKey, Vec<Texture2D>> {
    let mut stopwatch = Stopwatch::default();
    println!("Loading level selector textures...");
    stopwatch.start();
    let mut result = BTreeMap::new();

    let button_textures = {
        let path = get_texture_path(TextureKey::Button0).await;
        load_textures_from_tile_map(path)
    };



    result.insert(TextureKey::Button0, button_textures.await);

    stopwatch.stop();
    println!("Loaded level selector! Took {}ms", stopwatch.elapsed().as_millis());

    result
}