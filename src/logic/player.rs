use std::collections::BTreeMap;
use macroquad::camera::set_camera;
use macroquad::color::{Color, RED, WHITE};
use macroquad::input::{is_key_down, is_key_pressed, is_mouse_button_pressed, mouse_position, KeyCode, MouseButton};
use macroquad::math::{vec2, Vec2};
use macroquad::prelude::{draw_texture_ex, get_frame_time, screen_height, Camera2D, DrawTextureParams, Rect, Texture2D};
use macroquad::shapes::draw_rectangle;
use macroquad::time::get_time;
use macroquad::window::screen_width;
use macroquad_platformer::{Actor, World};
use crate::logic::collider::Collider;
use crate::scenes::levels::structs::{LevelData, Projectile, ProjectileOrigin, Trigger};
use crate::utils::structs::Settings;
use crate::utils::enums::{Animation, AnimationType, Direction, TextureKey};
use crate::utils::mathemann::point_to_point_direction_with_speed;

// This file contains everything that is for the player
#[derive(PartialEq, Clone, Debug)]
pub struct PowerUp {
    pub collected: bool,
    pub power_up: PlayerPowerUp,
    /// Stores the duration of the [PowerUp]
    pub duration: f64,
    pub collider: Collider,
    pub texture_key: TextureKey,
    pub animation: Animation,
    pub size: Vec2,
    pub speed: Vec2,
}

impl PowerUp {
    pub async fn new(power_up: PlayerPowerUp, duration: f64, pos: Vec2, size: Vec2, texture_key: TextureKey, texture_range: (u32, u32), texture_speed: f64) -> Self {
        let collected = false;
        let collider = Collider::new_collectible(pos, size.x, size.y, vec2(0.0, 0.0)).await;
        let animation = Animation::new(AnimationType::Cycle(texture_range.0, texture_range.1, texture_speed));
        let speed = vec2(0.0, 0.0);

        Self {
            collected,
            power_up,
            duration,
            collider,
            texture_key,
            animation,
            size,
            speed,
        }
    }

    /// Runs all checks that may be needed on an [PowerUp]
    pub async fn tick(&mut self, player: &mut Player) {
        // Check if the collectible collides with another thing
        if self.collider.touching_player(player).await {
            self.collected = true;
            player.power_ups.insert(self.power_up.clone(), self.duration);
            player.power_ups_exec.insert(self.power_up.clone(), get_time());
        }
    }

    pub async fn render(&mut self, textures: &BTreeMap<TextureKey, Vec<Texture2D>>) {
        let pos = self.collider.pos().await;

        match self.animation.animation_type {
            AnimationType::Cycle(_, _, _) => {
                self.animation.animate().await;
                let texture = textures.get(&self.texture_key).unwrap().get(self.animation.index as usize).unwrap();
                draw_texture_ex(
                    texture,
                    pos.x,
                    pos.y,
                    WHITE,
                    DrawTextureParams {
                        dest_size: Some(self.size),
                        ..Default::default()
                    },
                );
            }
        }
    }
}

#[derive(PartialEq, Clone)]
pub struct Player {
    pub health: i16,
    pub color: Color,
    pub width: f32,
    pub height: f32,
    /// 0: Left <br>
    /// 1: Right
    pub state: i8,
    pub collider: Actor,
    pub collider_new: Collider,
    pub camera_collider: [Actor; 4],
    pub speed: Vec2,
    /// All triggers and if a Trigger is active or not
    pub triggers: BTreeMap<PlayerTrigger, bool>,
    /// Contains the last time a trigger was executed
    pub triggers_exec: BTreeMap<PlayerTrigger, f64>,
    /// All power ups and its duration
    pub power_ups: BTreeMap<PlayerPowerUp, f64>,
    /// Contains the time the power up was there
    pub power_ups_exec: BTreeMap<PlayerPowerUp, f64>,
}

#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Copy)]
pub enum PlayerTrigger {
    DamageOverlay,
    DamageCooldown,
    ShootTimeout
}

#[derive(PartialEq, Eq, Clone, Ord, PartialOrd, Copy, Debug)]
pub enum PlayerPowerUp {
    JumpBoost,
    SpeedBoost,
    Coins2x,
    Damage2x,
}

impl Player {
    pub async fn new(width: f32, height: f32, pos: Vec2, state: i8, world: &mut World) -> Self {
        Player {
            health: 1000,
            color: WHITE,
            width,
            height,
            state,
            collider: world.add_actor(pos, width as i32, height as i32),
            collider_new: Collider::new_actor(pos, width, height, vec2(0.0, 0.0)).await,
            camera_collider: [
                // Left
                world.add_actor(vec2(0.0, 0.0), (screen_width() / 4.0) as i32, screen_height() as i32),
                // Right
                world.add_actor(vec2(screen_width() - (screen_width() / 4.0), 0.0), (screen_width() / 4.0) as i32, screen_height() as i32),
                // Up
                world.add_actor(vec2(0.0, 0.0), screen_width() as i32, (screen_height() / 8.0) as i32),
                // Down
                world.add_actor(vec2(0.0,  screen_height() - screen_height() / 8.0), screen_width() as i32, (screen_height() / 8.0) as i32),
            ],
            speed: vec2(0.0, 0.0),
            triggers: BTreeMap::new(),
            triggers_exec: BTreeMap::new(),
            power_ups: BTreeMap::new(),
            power_ups_exec: BTreeMap::new(),
        }
    }

    /// This function handles everything regarding the controls of the player (including moving)
    pub async fn control(&mut self, world: &mut World, level_data: &mut LevelData, settings: &Settings) {
        // gets the current position of the player from the world
        let pos = world.actor_pos(self.collider);
        // Checks if the player is on another collider by checking if one collider is 1px beyond him
        let on_ground = world.collide_check(self.collider, pos + vec2(0.0, 1.0));
        // Checks if the player is hitting a sealing by checking if one collider is 1px above him
        let sealing_hit = world.collide_check(self.collider, pos + vec2(0.0, -1.0));

        // If the player is hitting the sealing reset the velocity to 0
        if sealing_hit {
            self.speed.y = (100.0 * settings.gui_scale) * get_frame_time(); // I have no idea why but if this doesn't get multiplied by the frame time it's inconsistent on different Frame Rates
        }

        // If the player is not on the ground change velocity of y to 500 (to simulate gravity)
        if !on_ground {      // multiplies by get_frame_time() so the speed is on all refresh rates the same
            self.speed.y += (4800.0 * settings.gui_scale) * get_frame_time();
        } else {
            self.speed.y = 0.0;
        }

        // 1 = Left, 2 = Right
        let mut direction = 0;


        let movement_speed = {
            if self.power_ups.contains_key(&PlayerPowerUp::SpeedBoost) {
                2000.0 * settings.gui_scale
            } else {
                1300.0 * settings.gui_scale
            }
        };

        // Checks if key is currently pressed
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            // If D or Right Arrow is pressed the Player will be moved to the right by increasing the speed on the x-axis
            self.speed.x = movement_speed;
            self.state = 1;
            direction = 2;
        } else if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            self.speed.x = -movement_speed;
            self.state = 0;
            direction = 1;
        } else {
            // Resets the speed if nothing is pressed
            self.speed.x = 0.0;
        }

        if is_key_down(KeyCode::Space) {
            if on_ground {
                if self.power_ups.contains_key(&PlayerPowerUp::JumpBoost) {
                    self.speed.y = 2500.0 * -settings.gui_scale;
                } else {
                    self.speed.y = 1900.0 * -settings.gui_scale;
                }
            }
        }

        self.perform_move(world).await;
        self.tick(level_data, world, settings).await;

        let pos = world.actor_pos(self.collider);

        { // Make camera follow player
            fn move_camera_collider(collider: Actor, world: &mut World, direction: Direction, trigger: bool, pos: &Vec2, player: &Player) {
                let y = pos.y + player.height - screen_height();

                match direction {
                    Direction::Right => {
                        if trigger {
                            world.set_actor_position(collider, vec2(pos.x + player.width, y));
                        } else {
                            world.set_actor_position(collider, vec2(pos.x + (screen_width() / 2.0), y));
                        }
                    }
                    Direction::Left => {
                        if trigger {
                            world.set_actor_position(collider, vec2(pos.x - screen_width() / 4.0, y));
                        } else {
                            world.set_actor_position(collider, vec2(pos.x + player.width + screen_width() / 4.0 - screen_width(), y));
                        }
                    }
                    Direction::Up => {
                        if trigger {
                            world.set_actor_position(collider, vec2(pos.x, pos.y - screen_height() / 8.0));
                        } else {
                            world.set_actor_position(collider, vec2(pos.x, pos.y + screen_height() / 8.0 + player.height - screen_height()));
                        }
                    }
                    Direction::Down => {
                        if trigger {
                            world.set_actor_position(collider, vec2(pos.x, pos.y + player.height));
                        } else {
                            world.set_actor_position(collider, vec2(pos.x, (pos.y + screen_height() / 8.0) + screen_height() - screen_height() / 2.0 + screen_height() / 8.0));
                        }
                    }
                }
            }

            if pos.x - 1.0 <= world.actor_pos(self.camera_collider[0]).x + screen_width() / 4.0 && direction != 2 {
                let y = world.actor_pos(self.camera_collider[2]);
                set_camera(&Camera2D::from_display_rect(Rect::new(pos.x - screen_width() / 4.0, y.y + screen_height(), screen_width(), -screen_height())));

                move_camera_collider(self.camera_collider[0], world, Direction::Left, true, &pos, self);
                move_camera_collider(self.camera_collider[1], world, Direction::Right, false, &pos, self);
            } else if pos.x + self.height + 1.0 >= world.actor_pos(self.camera_collider[1]).x && direction != 1 {
                let y = world.actor_pos(self.camera_collider[2]);
                set_camera(&Camera2D::from_display_rect(Rect::new(pos.x + self.width - (screen_width() - screen_width() / 4.0), y.y + screen_height(), screen_width(), -screen_height())));

                move_camera_collider(self.camera_collider[0], world, Direction::Left, false, &pos, self);
                move_camera_collider(self.camera_collider[1], world, Direction::Right, true, &pos, self);
            }

            if pos.y -1.0 <= world.actor_pos(self.camera_collider[2]).y + screen_height() / 8.0 && !(pos.y + self.height + 1.0 >= world.actor_pos(self.camera_collider[3]).y) {
                let x = world.actor_pos(self.camera_collider[0]);
                set_camera(&Camera2D::from_display_rect(Rect::new(x.x, pos.y - screen_height() / 8.0 + screen_height(), screen_width(), -screen_height())));
                move_camera_collider(self.camera_collider[2], world, Direction::Up, true, &pos, self);
                move_camera_collider(self.camera_collider[3], world, Direction::Down, false, &pos, self);
            } else if pos.y + self.height + 1.0 >= world.actor_pos(self.camera_collider[3]).y && !(pos.y -1.0 <= world.actor_pos(self.camera_collider[2]).y + screen_height() / 8.0) {
                let x = world.actor_pos(self.camera_collider[0]);
                set_camera(&Camera2D::from_display_rect(Rect::new(x.x, ((pos.y + self.height + screen_height() / 8.0) - screen_height()) + screen_height(), screen_width(), -screen_height())));
                move_camera_collider(self.camera_collider[2], world, Direction::Up, false, &pos, self);
                move_camera_collider(self.camera_collider[3], world, Direction::Down, true, &pos, self);
            }
        }
    }

    pub async fn tick(&mut self, level_data: &mut LevelData, world: &World, settings: &Settings) {
        let enemies = &level_data.enemies;
        let colliding_enemies = self.collider_new.collide_check_enemy(enemies, vec2(0.0, 0.0)).await;
        if !colliding_enemies.is_empty() {
            for enemy in colliding_enemies {
                let damage = enemies.get(enemy).expect("Oh no! This should be impossible!").damage;
                self.damage(damage).await;
            }
        }

        if *self.triggers.get(&PlayerTrigger::DamageOverlay).unwrap_or(&false) {
            self.color = RED;
            if self.triggers_exec.get(&PlayerTrigger::DamageOverlay).unwrap() + 0.25 < get_time() {
                self.triggers.remove(&PlayerTrigger::DamageOverlay);
                self.triggers_exec.remove(&PlayerTrigger::DamageOverlay);
                self.color = WHITE;
            }
        }

        if self.health == 0 {
            level_data.triggers.insert(Trigger::GameOver, true);
        }

        if !self.triggers.get(&PlayerTrigger::ShootTimeout).unwrap_or(&false) {
            if is_mouse_button_pressed(MouseButton::Left) {
                let size = vec2(32.0, 32.0) * settings.gui_scale;
                let pos = world.actor_pos(self.collider) + vec2(self.width / 2.0, self.height / 2.0) - vec2(size.x / 2.0, size.y / 2.0);
                let pos_c_x = world.actor_pos(self.camera_collider[0]);
                let pos_c_y = world.actor_pos(self.camera_collider[3]);
                let (mut mouse_x, mut mouse_y) = mouse_position();
                mouse_x += pos_c_x.x;
                mouse_y -= pos_c_y.normalize().y;

                let movement_vector = point_to_point_direction_with_speed(pos, vec2(mouse_x, mouse_y), 1000.0 * settings.gui_scale).await;

                let projectile  = Projectile::new(
                    pos,
                    size,
                    -100,
                    4.0,
                    TextureKey::Coin0, ProjectileOrigin::Player, movement_vector).await;

                self.triggers.insert(PlayerTrigger::ShootTimeout, true);
                self.triggers_exec.insert(PlayerTrigger::ShootTimeout, get_time());

                level_data.projectiles.push(projectile);
            } else if is_key_pressed(KeyCode::Q) {
                let size = vec2(32.0, 32.0) * settings.gui_scale;
                let pos = world.actor_pos(self.collider) + vec2(self.width / 2.0, self.height / 2.0)- vec2(size.x / 2.0, size.y / 2.0);

                let movement_vector = vec2(-1.0, 0.0) * (1000.0 * settings.gui_scale);

                let projectile  = Projectile::new(
                    pos,
                    size,
                    -100,
                    4.0,
                    TextureKey::Coin0, ProjectileOrigin::Player, movement_vector).await;

                self.triggers.insert(PlayerTrigger::ShootTimeout, true);
                self.triggers_exec.insert(PlayerTrigger::ShootTimeout, get_time());

                level_data.projectiles.push(projectile);
            } else if is_key_pressed(KeyCode::E) {
                let size = vec2(32.0, 32.0) * settings.gui_scale;
                let pos = world.actor_pos(self.collider) + vec2(self.width / 2.0, self.height / 2.0)- vec2(size.x / 2.0, size.y / 2.0);

                let movement_vector = vec2(1.0, 0.0) * (1000.0 * settings.gui_scale);

                let projectile  = Projectile::new(
                    pos,
                    size,
                    -100,
                    4.0,
                    TextureKey::Coin0, ProjectileOrigin::Player, movement_vector).await;

                self.triggers.insert(PlayerTrigger::ShootTimeout, true);
                self.triggers_exec.insert(PlayerTrigger::ShootTimeout, get_time());

                level_data.projectiles.push(projectile);
            }
        } else if self.triggers_exec.get(&PlayerTrigger::ShootTimeout).unwrap_or(&0.0) + 0.05 < get_time() {
            self.triggers.remove(&PlayerTrigger::ShootTimeout);
            self.triggers_exec.remove(&PlayerTrigger::ShootTimeout);
        }

        for (power_up, duration) in self.power_ups.clone() {
            let start_time = self.power_ups_exec.get(&power_up).unwrap_or(&0.0);
            if start_time + duration < get_time()  {
                self.power_ups.remove(&power_up);
                self.power_ups_exec.remove(&power_up);
            }
        }
    }

    /// Moves the player and checks for all necessary things (like collision)
    pub async fn perform_move(&mut self, world: &mut World) {
        // Set positions using the previously defined speeds
        world.move_h(self.collider, self.speed.x * get_frame_time());
        world.move_v(self.collider, self.speed.y * get_frame_time());

        let pos = world.actor_pos(self.collider);
        self.collider_new.change_pos(pos).await;
    }

    pub async fn damage(&mut self, health: i16) {
        if !self.triggers.get(&PlayerTrigger::DamageCooldown).unwrap_or(&false) {
            self.health += health;
            if self.health < 0 { self.health = 0; }
            self.triggers.insert(PlayerTrigger::DamageOverlay, true);
            self.triggers_exec.insert(PlayerTrigger::DamageOverlay, get_time());
            self.triggers.insert(PlayerTrigger::DamageCooldown, true);
            self.triggers_exec.insert(PlayerTrigger::DamageCooldown, get_time());
        } else if self.triggers_exec.get(&PlayerTrigger::DamageCooldown).unwrap_or(&0.0) + 0.5 < get_time() {
            self.triggers.remove(&PlayerTrigger::DamageCooldown);
            self.triggers_exec.remove(&PlayerTrigger::DamageCooldown);
        }
    }

    pub async fn render(&mut self, world: &World, textures: &BTreeMap<TextureKey, Vec<Texture2D>>, settings: &Settings) {
        let pos = world.actor_pos(self.collider);

        draw_texture_ex(
            &textures.get(&TextureKey::Player).unwrap().get(self.state as usize).unwrap(), pos.x, pos.y, self.color,
            DrawTextureParams {
                dest_size: Some(vec2(self.width, self.height)),
                ..Default::default()
            },
        );

        // Draw Health bar
        let height = 32.0 * settings.gui_scale;
        let camera_collider_pos = world.actor_pos(self.camera_collider[0]);
        let camera_collider_pos_2 = world.actor_pos(self.camera_collider[2]);
        draw_rectangle(camera_collider_pos.x, camera_collider_pos_2.y, (self.health as f32 / 2.0) * settings.gui_scale, height, RED);
    }
}