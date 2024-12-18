use macroquad::time::get_time;
use crate::logic;

#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone)]
pub enum SceneTextureKey {
    MainMenu,
    LevelSelector,
    SettingsMenu,

    Level0,
    Level1,
    Level2,
    Level3,
}

/// All textures
#[derive(Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Debug)]
pub enum TextureKey {
    Player,
    Enemy0,
    Projectile0,

    // Platforms
    Platform0,

    // Collectibles
    /// This texture needs to be animated ([AnimationType::Cycle]) <br>
    /// `0, 5` is the range
    Coin0,

    /// This texture needs to be animated ([AnimationType::Cycle]) <br>
    /// `0, 17` is the jump boost texture <br>
    /// `18, 40` is the speed boost texture <br>
    /// `41, 63` is the double coins texture <br>
    /// `64, 83` is the damage boost texture
    PowerUps0,

    /// ##### This texture needs to be animated ([AnimationType::Cycle]) <br>
    /// ###### Ranges:
    /// `0, 20` is the sword (kills) texture <br>
    Icons0,

    /// No Animation
    /// `0` is [Direction::Left]
    /// `1` is [Direction::Right]
    /// `2` is [Direction::Up]
    /// `3` is [Direction::Down]
    Cannon0,

    Button0,
}

pub enum Scene {
    MainMenu,
    SettingsMenu,
    /// The i32 is the Page
    LevelSelector(i32),
    Level(logic::level::Level)
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Animation {
    pub animation_type: AnimationType,
    pub last_time: f64,
    /// Contains the current index or frame of the animation (should be -1 at first)
    pub index: i32,
}

impl Animation {
    pub fn new(animation_type: AnimationType) -> Self {
        Self {animation_type, last_time: get_time(), index: -1}
    }

    /// Executes the current animation <b>
    /// Depending on what animation you are trying to animate you may need to do some steps manually <br>
    /// For more information about what to do please refer to the documentation of the chosen animation
    pub async fn animate(&mut self) {
        match self.animation_type {
            AnimationType::Cycle(start, end, speed) => {
                // Set index to start (if not already done)
                if self.index == -1 {
                    self.index = start as i32 - 1
                }

                if self.last_time < get_time() - speed {

                    // Reset index if above max
                    if self.index < end as i32 {
                        self.index += 1;
                    } else {
                        self.index = start as i32
                    }

                    self.last_time = get_time();
                }

                // Set index to start (if not already done)
                if self.index == start as i32 - 1 {
                    self.index = start as i32
                }
            }
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AnimationType {
    /// Goes through a fixed number of textures <br>
    /// For this animation the index represents the current texture index. <br>
    /// **This animation needs to be rendered manually** <br>
    /// The first [u32] represents the start, the second the end and the last the speed
    Cycle(u32, u32, f64)
}

#[derive(Copy, Clone, PartialEq, Ord, Eq, PartialOrd, Debug)]
pub enum Direction {
    Right,
    Left,
    Up,
    Down,
}