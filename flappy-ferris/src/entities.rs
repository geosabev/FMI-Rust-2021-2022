use crate::assets::Assets;
use ggez::event::KeyCode;
use ggez::graphics::Rect;
use ggez::input::keyboard;
use ggez::mint::{Point2, Vector2};
use ggez::{graphics, Context, GameResult};

// Used for toggling outline drawing for entities.
pub const DEBUG_MODE: bool = false;

// Used for entity movement.
pub const GRAVITY: f32 = 0.50;
pub const JUMP: f32 = 8.0;
pub const PIPE_SPEED: f32 = 4.5;
pub const ENEMY_SPEED: f32 = 5.5;
pub const BOOST_SPEED: f32 = 7.0;

// Used for calculating entity positions.
pub const SCREEN_WIDTH: f32 = 1024.0;
pub const SCREEN_HEIGHT: f32 = 768.0;
pub const MIDDLE: f32 = 704.0 / 2.0;
pub const FLOOR_LEVEL: f32 = 683.0;
pub const FERRIS_WIDTH: f32 = 64.0;
pub const FERRIS_HEIGHT: f32 = 42.0;
pub const PIPE_WIDTH: f32 = 128.0;
pub const PIPE_GAP: f32 = 160.0;
pub const ENEMY_WIDTH: f32 = 128.0;
pub const ENEMY_HEIGHT: f32 = 84.0;
pub const BOOST_WIDTH: f32 = 64.0;
pub const BOOST_HEIGHT: f32 = 64.0;

// Used for debugging overlapping of different entities.
// Source: rust-shooter game in GitHub by andrew.
pub fn draw_outline(bounding_box: graphics::Rect, ctx: &mut Context) -> GameResult<()> {
    if DEBUG_MODE {
        let draw_mode =
            graphics::DrawMode::Stroke(graphics::StrokeOptions::default().with_line_width(1.0));
        let red = graphics::Color::from_rgb(255, 0, 0);
        let outline = graphics::MeshBuilder::new()
            .rectangle(draw_mode, bounding_box, red)
            .unwrap()
            .build(ctx)
            .unwrap();

        graphics::draw(ctx, &outline, graphics::DrawParam::default())?;
    }

    Ok(())
}

// States the game could be in.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PlayState {
    StartScreen,
    Play,
    Dead,
}
impl PlayState {
    pub fn is_playing(&self) -> bool {
        *self == PlayState::Play
    }

    pub fn set_dead(&mut self) {
        *self = PlayState::Dead;
    }
}

// Different types of boosts.
pub enum BoostType {
    SpeedUp,
    SlowDown,
    BonusLife,
}

// Used for moving the player. Only y-based movement needed since they move only up and down.
pub struct Physics {
    pub velocity: f32,
    pub acceleration: f32,
}
impl Physics {
    pub fn new() -> Self {
        Self {
            velocity: 0.0,
            acceleration: 0.0,
        }
    }
}

// The player entity
pub struct PlayerEntity {
    pub position: Point2<f32>,
    pub physics: Physics,
    pub zone: Rect,
    pub can_jump: bool,
}
impl PlayerEntity {
    pub fn new() -> Self {
        Self {
            position: Point2 {
                x: SCREEN_WIDTH / 4.0,
                y: MIDDLE,
            },
            physics: Physics::new(),
            zone: Rect {
                x: (SCREEN_WIDTH / 4.0) - (FERRIS_WIDTH / 2.0),
                y: MIDDLE - (FERRIS_HEIGHT / 2.0),
                w: FERRIS_WIDTH,
                h: FERRIS_HEIGHT,
            },
            can_jump: true,
        }
    }

    pub fn update(&mut self, ctx: &mut Context, state: &PlayState) -> PlayState {
        let physics = &mut self.physics;
        physics.acceleration = GRAVITY;

        if !(keyboard::pressed_keys(ctx).contains(&KeyCode::Space)) && !(self.can_jump) {
            self.can_jump = true;
        }

        let mut new_state = state.clone();
        if keyboard::is_key_pressed(ctx, KeyCode::Space) && self.can_jump {
            let physics = &mut self.physics;

            self.can_jump = false;

            PlayerEntity::jump(physics);

            if new_state == PlayState::StartScreen || new_state == PlayState::Dead {
                new_state = PlayState::Play;
            }
        }

        if new_state == PlayState::StartScreen {
            self.auto_jump();
        }

        self.change_player_position();
        self.prevent_going_out();

        new_state
    }

    pub fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let p = &self.physics;

        let x = if p.velocity >= 0.0 {
            &assets.ferris_stable_image
        } else {
            &assets.ferris_jumping_image
        };

        graphics::draw(
            ctx,
            x,
            graphics::DrawParam::default()
                .dest(self.position.clone())
                .offset(Point2 { x: 0.5, y: 0.5 }),
        )
        .unwrap();

        draw_outline(self.zone, ctx).unwrap();

        Ok(())
    }

    // The main jump function
    fn jump(physics: &mut Physics) {
        physics.acceleration = GRAVITY;
        physics.velocity = -JUMP;
    }

    // Used during the StartScreen state.
    fn auto_jump(&mut self) {
        let physics = &mut self.physics;

        if self.position.y >= MIDDLE {
            PlayerEntity::jump(physics);
        }
    }

    // Calculates the changes to position and zone of player after jumping.
    fn change_player_position(&mut self) {
        let physics = &mut self.physics;

        physics.velocity += physics.acceleration;

        self.position = Point2 {
            x: self.position.x,
            y: self.position.y + physics.velocity,
        };

        let offset = Vector2 {
            x: 0.0,
            y: physics.velocity,
        };

        self.zone.translate(offset);
    }

    // Stops the player from going over the top and "cheating" the pipes.
    fn prevent_going_out(&mut self) {
        self.position.y = if self.position.y < (FERRIS_HEIGHT / 2.0) {
            self.zone.y = 0.0;
            FERRIS_HEIGHT / 2.0
        } else {
            self.position.y
        }
    }

    // Sends the player back to the middle of the screen.
    pub fn prevent_hitting_ground(&mut self) {
        self.position.y = MIDDLE;
        self.zone.y = MIDDLE - (FERRIS_HEIGHT / 2.0);
    }

    // Checks if the player is touching the given ground level.
    pub fn hits_ground(&mut self) -> bool {
        self.position.y > FLOOR_LEVEL
    }
}

// The pipe entity. (the only one with two zones and two sprites (top and bottom) instead of one, since calculating the deviation and safe spaces between top and bottom was a nightmare if it was one)
pub struct PipeEntity {
    pub position: Point2<f32>,
    pub top_zone: Rect,
    pub bottom_zone: Rect,
    pub is_passed: bool,
}
impl PipeEntity {
    pub fn new(y: f32) -> Self {
        Self {
            position: Point2 {
                x: SCREEN_WIDTH + (PIPE_WIDTH / 2.0),
                y: y,
            },
            top_zone: Rect {
                x: SCREEN_WIDTH,
                y: 0.0,
                w: PIPE_WIDTH,
                h: y,
            },
            bottom_zone: Rect {
                x: SCREEN_WIDTH,
                y: y + PIPE_GAP,
                w: PIPE_WIDTH,
                h: SCREEN_HEIGHT - y - PIPE_GAP,
            },
            is_passed: false,
        }
    }

    pub fn update(&mut self, multiplier: f32) {
        let pos = &mut self.position;

        self.position = Point2 {
            x: pos.x - (PIPE_SPEED * multiplier),
            y: pos.y,
        };

        let offset = Vector2 {
            x: -(PIPE_SPEED * multiplier),
            y: 0.0,
        };

        self.bottom_zone.translate(offset);
        self.top_zone.translate(offset);
    }

    pub fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let top = &assets.pipe_top_image;
        let dest_top = Point2 {
            x: self.position.x,
            y: self.position.y,
        };
        let offset_top = Point2 { x: 0.5, y: 1.0 };

        let bottom = &assets.pipe_bottom_image;
        let dest_bottom = Point2 {
            x: self.position.x,
            y: self.position.y + PIPE_GAP,
        };
        let offset_bottom = Point2 { x: 0.5, y: 0.0 };

        graphics::draw(
            ctx,
            top,
            graphics::DrawParam::default()
                .dest(dest_top.clone())
                .offset(offset_top),
        )
        .unwrap();

        graphics::draw(
            ctx,
            bottom,
            graphics::DrawParam::default()
                .dest(dest_bottom.clone())
                .offset(offset_bottom),
        )
        .unwrap();

        draw_outline(self.bottom_zone, ctx).unwrap();
        draw_outline(self.top_zone, ctx).unwrap();

        Ok(())
    }
}

// The enemy entity.
pub struct EnemyEntity {
    pub position: Point2<f32>,
    pub zone: Rect,
    pub is_passed: bool,
}
impl EnemyEntity {
    pub fn new(y: f32) -> Self {
        Self {
            position: Point2 {
                x: SCREEN_WIDTH + (ENEMY_WIDTH / 2.0),
                y: y,
            },
            zone: Rect {
                x: SCREEN_WIDTH,
                y: y - (ENEMY_HEIGHT / 2.0),
                w: ENEMY_WIDTH,
                h: ENEMY_HEIGHT,
            },
            is_passed: false,
        }
    }

    pub fn update(&mut self, multiplier: f32) {
        let pos = &mut self.position;

        self.position = Point2 {
            x: pos.x - (ENEMY_SPEED * multiplier),
            y: pos.y,
        };

        let offset = Vector2 {
            x: -(ENEMY_SPEED * multiplier),
            y: 0.0,
        };
        self.zone.translate(offset);
    }

    pub fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let x = &assets.enemy_image;
        let offset = Point2 { x: 0.5, y: 0.5 };

        graphics::draw(
            ctx,
            x,
            graphics::DrawParam::default()
                .dest(self.position.clone())
                .offset(offset),
        )
        .unwrap();

        draw_outline(self.zone, ctx).unwrap();

        Ok(())
    }
}

// The boost entity.
pub struct BoostEntity {
    pub position: Point2<f32>,
    pub zone: Rect,
    pub effect: BoostType,
    pub is_passed: bool,
    pub is_collected: bool,
}
impl BoostEntity {
    pub fn new(y: f32, val: f32) -> Self {
        let mut eff: BoostType = BoostType::SlowDown;
        if val < 10.0 {
            eff = BoostType::SpeedUp;
        }

        if val < 2.0 {
            eff = BoostType::BonusLife;
        }

        Self {
            position: Point2 {
                x: SCREEN_WIDTH + (BOOST_WIDTH / 2.0),
                y: y,
            },
            zone: Rect {
                x: SCREEN_WIDTH,
                y: y - (BOOST_HEIGHT / 2.0),
                w: BOOST_WIDTH,
                h: BOOST_HEIGHT,
            },
            effect: eff,
            is_passed: false,
            is_collected: false,
        }
    }

    pub fn update(&mut self) {
        let pos = &mut self.position;

        self.position = Point2 {
            x: pos.x - BOOST_SPEED,
            y: pos.y,
        };

        let offset = Vector2 {
            x: -BOOST_SPEED,
            y: 0.0,
        };
        self.zone.translate(offset);
    }

    pub fn draw(&mut self, ctx: &mut Context, assets: &Assets) -> GameResult {
        let x = match self.effect {
            BoostType::BonusLife => &assets.boost_life_image,
            BoostType::SlowDown => &assets.boost_slow_down_image,
            BoostType::SpeedUp => &assets.boost_speed_up_image,
        };

        let offset = Point2 { x: 0.5, y: 0.5 };

        graphics::draw(
            ctx,
            x,
            graphics::DrawParam::default()
                .dest(self.position.clone())
                .offset(offset),
        )
        .unwrap();

        draw_outline(self.zone, ctx).unwrap();

        Ok(())
    }
}
