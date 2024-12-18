use std::collections::BTreeMap;
use macroquad::camera::set_default_camera;
use macroquad::color::{BLACK, WHITE};
use macroquad::prelude::{get_time, Texture2D};
use macroquad::window::{clear_background, screen_height};
use std::time::SystemTime;
use macroquad_platformer::World;
use stopwatch2::Stopwatch;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use macroquad::math::{vec2, Vec2};
use crate::logic::cannon::Cannon;
use crate::logic::collectible::{Collectible, CollectibleType};
use crate::logic::enemy::Enemy;
use crate::logic::platform::Platform;
use crate::logic::player::{Player, PlayerPowerUp, PowerUp};
use crate::logic::projectile::Projectile;
use crate::utils::structs::{Settings};
use crate::utils::enums::TextureKey;
use crate::utils::text::{draw_text_center, draw_text_centered};

pub async fn render_level(level_scene_data: &mut LevelSceneData, textures: &BTreeMap<TextureKey, Vec<Texture2D>>, settings: &Settings) {
    match level_scene_data.level_data.triggers.get(&Trigger::GameOver).unwrap_or(&false) {
        true => {
            set_default_camera();
            clear_background(BLACK);
            draw_text_center("GAME OVER", 250.0 * settings.gui_scale, WHITE).await;
            draw_text_centered("Press ESC to go back or Ctrl + R to retry", screen_height() / 2.0 + 250.0 * settings.gui_scale, 60.0 * settings.gui_scale, WHITE).await;
        }
        false => {
            let world = &level_scene_data.world;
            let platforms = &level_scene_data.level_data.platforms;
            let collectibles = &mut level_scene_data.level_data.collectibles;
            let enemies = &level_scene_data.level_data.enemies;
            let cannons = &level_scene_data.level_data.cannons;
            let projectiles = &level_scene_data.level_data.projectiles;
            let power_ups = &mut level_scene_data.level_data.power_ups;

            // Render collectibles
            for collectible in collectibles {
                collectible.render(textures).await;
            }

            let platforms = async {
                // Render Platforms
                for platform in platforms {
                    platform.render(textures, world).await;
                }
            };

            let enemies = async {
                // Render enemies
                for enemy in enemies {
                    enemy.render(textures, settings).await;
                }
            };

            let cannons = async {
                for cannon in cannons {
                    cannon.render(textures).await;
                }
            };

            let projectiles = async {
                // Render projectiles
                for projectile in projectiles {
                    projectile.render(textures).await;
                }
            };

            // Render power ups
            for power_up in power_ups {
                power_up.render(textures).await;
            }

            platforms.await;
            enemies.await;
            cannons.await;
            projectiles.await;


            // Render Player
            level_scene_data.level_data.player.as_mut().unwrap().render(&world, textures, settings).await;
        }
    }
}

pub async fn tick_level(level_scene_data: &mut LevelSceneData, settings: &Settings) {
    {   // Tick collectibles
        let mut collectibles_to_remove = Vec::new();

        for (i, collectible) in level_scene_data.level_data.collectibles.iter_mut().enumerate() {
            collectible.check(level_scene_data.level_data.player.as_ref().unwrap()).await;
            if collectible.collected {
                collectibles_to_remove.push(i);
            }
        }

        for i in collectibles_to_remove {
            if level_scene_data.level_data.collectibles.get(i).unwrap().collectible_type == CollectibleType::Coin {
                let player = level_scene_data.level_data.player.as_mut().unwrap();
                if player.power_ups.contains_key(&PlayerPowerUp::Coins2x) {
                    player.coins += 2;
                } else {
                    player.coins += 1;
                }
            }
            level_scene_data.level_data.collectibles.remove(i);
        }
    }
    { // Tick enemies
        let enemies = &mut level_scene_data.level_data.enemies;
        let projectiles = &mut level_scene_data.level_data.projectiles;
        let world = &mut level_scene_data.world;
        let player = &mut level_scene_data.level_data.player.as_mut().unwrap();

        let mut enemies_to_remove = Vec::new();

        for (i, enemy) in enemies.iter_mut().enumerate() {
            if enemy.deletable {
                enemies_to_remove.push(i);
                continue;
            }
            enemy.tick(world, player, projectiles, settings).await;
        }

        let new_enemies = {
            let mut result = Vec::new();

            for (i, enemy) in enemies.iter().enumerate() {
                if !enemies_to_remove.contains(&i)  {
                    result.push(enemy.to_owned());
                }
            }

            result
        };

        *enemies = new_enemies.to_owned();
    }
    { // Tick cannons
        let cannons = &mut level_scene_data.level_data.cannons;
        let projectiles = &mut level_scene_data.level_data.projectiles;

        for cannon in cannons {
            cannon.tick(projectiles).await;
        }
    }
    { // Tick projectiles
        let mut level_data = level_scene_data.level_data.clone();
        let projectiles = &mut level_scene_data.level_data.projectiles;

        let mut projectiles_to_remove = Vec::new();

        for (i, projectile) in projectiles.iter_mut().enumerate() {
            projectile.tick(&level_data).await;

            if projectile.deletable {
                projectiles_to_remove.push(i);
            }
        }

        let new_projectiles = {
            let mut result = Vec::new();

            for (i, projectile) in projectiles.iter().enumerate() {
                if !projectiles_to_remove.contains(&i)  {
                    result.push(projectile.to_owned());
                }
            }

            result
        };

        level_data.projectiles = new_projectiles.to_owned();
        level_scene_data.level_data = level_data;
    }
    { // Tick power ups
        let player = level_scene_data.level_data.player.as_mut().unwrap();

        let mut power_ups_to_remove = Vec::new();

        for (i, power_up) in level_scene_data.level_data.power_ups.iter_mut().enumerate() {
            power_up.tick(player).await;
            if power_up.collected {
                power_ups_to_remove.push(i);
            }
        }

        for power_up in power_ups_to_remove {
            level_scene_data.level_data.power_ups.remove(power_up);
        }
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
/// Contains ALL data that may be saved across multiple sessions of levels
pub struct PersistentLevelData {
    pub stats: BTreeMap<Level, LevelStat>,
    pub scores: BTreeMap<Level, Vec<LevelScore>>
}

impl PersistentLevelData {
    pub fn new() -> Self {
        let stats = BTreeMap::new();
        let scores= BTreeMap::new();

        Self { stats, scores }
    }

    pub async fn save(&self, settings: &Settings) {
        let mut file = fs::File::create(&settings.level_data_path).unwrap();

        let s_persistent_level_data = serde_json::to_string_pretty(&self).expect("Couldn't serialize level data");
        file.write_all(s_persistent_level_data.as_bytes()).expect("Couldn't write level data file");
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct LevelStat {
    pub level: Level,
    /// The total amount of tries to play the level
    pub plays: u32,
    /// The total amount of all deaths
    pub deaths: u32,
}

impl LevelStat {
    pub fn new(level: Level) -> Self {
        Self { level, plays: 0, deaths: 0 }
    }

    pub fn update(&mut self, deaths: u32) {
        self.plays += 1;
        self.deaths += deaths;
    }
}

#[derive(PartialEq, Clone, Serialize, Deserialize, Debug)]
pub struct LevelScore {
    /// The time this was created
    pub time: SystemTime,
    /// The total playtime of the level
    pub playtime: f64,
    /// The total amount of collected coins
    pub coins: u32,
    /// The total amount of kills
    pub kills: u32,
    /// The total mount of damage that the player has done
    pub total_damage: u32,
    /// The total amount of damage that the player received
    pub total_damage_received: u32
}

impl LevelScore {
    pub fn new(playtime: f64, coins: u32, kills: u32, total_damage: u32, total_damage_received: u32) -> LevelScore {
        let time = SystemTime::now();
        Self { time, playtime, coins, kills, total_damage, total_damage_received }
    }
}

/// This enum defines all existing levels
#[derive(Eq, PartialEq, Clone, Ord, PartialOrd, Serialize, Deserialize, Debug)]
pub enum Level {
    Level0,
    Level1,
    Level2,
    Level3,
}

#[derive(Eq, PartialEq, Clone, Ord, PartialOrd, Debug)]
pub enum Trigger {
    ShowCameraColliders,
    ShowColliders,
    ShowFPS,

    LevelCompleted,
    GameOver,

    // Tutorial Trigger
    TutorialWalking,
    TutorialSpace,
    TutorialCoins
}

#[derive(Clone)]
pub struct LevelData {
    pub start_time: f64,

    /// The zero position of the current canvas
    pub zero: Vec2,

    pub level: Option<Level>,
    pub player: Option<Player>,
    pub platforms: Vec<Platform>,
    pub collectibles: Vec<Collectible>,
    pub enemies: Vec<Enemy>,
    pub cannons: Vec<Cannon>,
    pub projectiles: Vec<Projectile>,
    pub power_ups: Vec<PowerUp>,
    /// Saves temporary triggers / settings
    pub triggers: BTreeMap<Trigger, bool>,
    pub triggers_exec: BTreeMap<Trigger, f64>,
    pub trigger_locks: BTreeMap<Trigger, bool>
}

impl LevelData {
    pub async fn new(level: Level, player: Player, platforms: Vec<Platform>, collectibles: Vec<Collectible>, enemies: Vec<Enemy>, cannons: Vec<Cannon>, power_ups: Vec<PowerUp>) -> Self {
        let start_time = get_time();
        let zero = vec2(0.0, 0.0);
        let level = Some(level);
        let player = Some(player);
        let projectiles = Vec::new();
        let triggers = BTreeMap::new();
        let triggers_exec = BTreeMap::new();
        let trigger_locks = BTreeMap::new();

        Self { start_time, zero, level, player, platforms, collectibles, enemies, cannons, projectiles, power_ups, triggers, triggers_exec, trigger_locks  }
    }

    pub async fn save(&self, persistent_level_data: &mut PersistentLevelData, settings: &Settings) {
        let mut stopwatch = Stopwatch::default();
        println!("Saving level score and updating stats...");
        stopwatch.start();
        let playtime = get_time() - self.start_time;
        let player = self.player.as_ref().unwrap();
        let level = self.level.as_ref().unwrap();

        let score = LevelScore::new(
            playtime,
            player.coins,
            player.kills,
            player.total_damage,
            player.total_damage_received
        );

        if persistent_level_data.scores.get(level).is_none() {
            persistent_level_data.scores.insert(level.to_owned(), Vec::new());
        }

        if persistent_level_data.stats.get(level).is_none() {
            persistent_level_data.stats.insert(level.to_owned(), LevelStat::new(level.to_owned()));
        }
        let stats_ref = persistent_level_data.stats.get_mut(level).unwrap();
        let deaths = {
            if *self.triggers.get(&Trigger::GameOver).unwrap_or(&false) {
                1
            } else {
                0
            }
        };
        stats_ref.update(deaths);

        let score_space = persistent_level_data.scores.get_mut(level).unwrap();
        // 10000 entries are about 3 MB large, and we don't want to go larger than that
        if score_space.len() > 10000 { score_space.remove(0); }
        score_space.push(score);

        persistent_level_data.save(settings).await;

        stopwatch.stop();
        println!("Saved level level score and updated stats! Took {}ms", stopwatch.elapsed().as_millis());
    }

    pub async fn insert_trigger(&mut self, trigger: Trigger, value: bool) {
        self.triggers.insert(trigger.to_owned(), value);
        self.triggers_exec.insert(trigger, get_time());
    }
}

/// Holds all data a level can possibly have
pub struct LevelSceneData {
    pub level_data: LevelData,
    pub world: World,
}

impl LevelSceneData {
    pub async fn new(level_data: LevelData, world: World) -> Self {
        Self { level_data, world }
    }

    pub async fn empty() -> Self {
        let level_data = LevelData {
            start_time: 0.0,

            zero: vec2(0.0, 0.0),

            level: None,
            player: None,
            platforms: Vec::new(),
            collectibles: Vec::new(),
            enemies: Vec::new(),
            cannons: Vec::new(),
            projectiles: Vec::new(),
            power_ups: Vec::new(),
            triggers: BTreeMap::new(),
            triggers_exec: BTreeMap::new(),
            trigger_locks: BTreeMap::new()
        };

        Self {
            level_data,
            world: World::new(),
        }
    }
}