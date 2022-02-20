use flappy_ferris::assets::Assets;
use flappy_ferris::entities::{
    BoostEntity, BoostType, EnemyEntity, PipeEntity, PlayState, PlayerEntity, BOOST_WIDTH,
    ENEMY_WIDTH, PIPE_WIDTH, SCREEN_HEIGHT, SCREEN_WIDTH,
};
use ggez::audio::SoundSource;
use ggez::conf::{Backend, Conf, ModuleConf, WindowMode, WindowSetup};
use ggez::mint::Point2;
use ggez::ContextBuilder;
use ggez::{event, event::EventHandler, graphics, Context, GameResult};
use rand::rngs::ThreadRng;
use rand::Rng;
use std::collections::VecDeque;
use std::path;

pub const BOOST_DURATION: f32 = 10000000000.0;

// The struct of the game.
pub struct MainState {
    player: PlayerEntity,
    pipes: VecDeque<PipeEntity>,
    enemies: VecDeque<EnemyEntity>,
    boosts: VecDeque<BoostEntity>,

    time_until_next_pipe: f32,
    time_until_next_enemy: f32,
    time_until_next_boost: f32,

    boost_duration: f32,
    multiplier: f32,

    hit_pipe: bool,
    hit_enemy: bool,
    has_boost: bool,

    play_state: PlayState,

    lifes: i128,
    score: i128,
    best_score: i128,

    assets: Assets,

    rng: ThreadRng,
}
impl MainState {
    pub fn new(ctx: &mut Context) -> Self {
        let assets = Assets::new(ctx).unwrap();

        Self {
            player: PlayerEntity::new(),
            pipes: VecDeque::new(),
            enemies: VecDeque::new(),
            boosts: VecDeque::new(),

            // Time until each new entity is stored in ns and each frame's length is subtracted.
            time_until_next_pipe: 1000000000.0,
            time_until_next_enemy: 10000000000.0,
            time_until_next_boost: 10000000000.0,

            boost_duration: 0.0,
            multiplier: 1.0,

            hit_pipe: false,
            hit_enemy: false,
            has_boost: false,

            play_state: PlayState::StartScreen,

            lifes: 1,
            score: 0,
            best_score: 0,

            assets: assets,

            rng: rand::thread_rng(),
        }
    }

    // Resets all fields after a given game ends.
    fn restart(&mut self) {
        self.player = PlayerEntity::new();
        self.pipes = VecDeque::new();
        self.enemies = VecDeque::new();
        self.boosts = VecDeque::new();

        self.time_until_next_pipe = 1000000000.0;
        self.time_until_next_enemy = 10000000000.0;
        self.time_until_next_boost = 10000000000.0;

        self.boost_duration = 0.0;
        self.multiplier = 1.0;

        self.hit_pipe = false;
        self.hit_enemy = false;
        self.has_boost = false;

        // In order to actually display the GameOver logo instead of the StartScreen one.
        self.play_state = PlayState::Dead;

        // Before resetting the scores, we change the best score if needed.
        self.swap_scores();
        self.lifes = 1;
        self.score = 0;
    }

    // Updates the scores after a given game ends.
    fn swap_scores(&mut self) {
        if self.score > self.best_score {
            self.best_score = self.score;
        }
    }

    // Checks if the player lost the current game.
    fn is_over(&mut self) -> bool {
        if ((self.player.hits_ground()) || self.hit_enemy || self.hit_pipe)
            && self.play_state.is_playing()
        {
            return true;
        }

        false
    }

    // Displays the current score and lifes left during the game.
    fn draw_stats(&mut self, ctx: &mut Context) {
        let font = graphics::Font::new(ctx, "/FlappyBird.ttf").unwrap();

        // Displays the lifes available
        let mut scores = graphics::Text::new(format!("Lifes available: {}", self.lifes));
        scores.set_font(font, graphics::PxScale::from(30.0));
        graphics::draw(
            ctx,
            &scores,
            graphics::DrawParam::default()
                .dest(Point2 {
                    x: (SCREEN_WIDTH - scores.width(ctx)) / 2.0,
                    y: (SCREEN_HEIGHT - scores.height(ctx)) / 8.0,
                })
                .color(graphics::Color::BLACK),
        )
        .unwrap();

        // Displays the current score count
        let mut text = graphics::Text::new(format!("{}", self.score));
        text.set_font(font, graphics::PxScale::from(100.0));
        graphics::draw(
            ctx,
            &text,
            graphics::DrawParam::default()
                .dest(Point2 {
                    x: (SCREEN_WIDTH - text.width(ctx)) / 2.0,
                    y: (SCREEN_HEIGHT - text.height(ctx)) / 6.0,
                })
                .color(graphics::Color::BLACK),
        )
        .unwrap();
    }
}
impl EventHandler for MainState {
    fn update(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Restarts the game if player is dead.
        if self.play_state == PlayState::Dead {
            self.restart();
        }

        // Subtracts length of last frame from countdown fields (only if in Play state).
        let delta = ggez::timer::delta(ctx).as_nanos() as f32;
        if self.play_state.is_playing() {
            self.time_until_next_pipe -= delta;
            self.time_until_next_enemy -= delta;
            self.time_until_next_boost -= delta;
        }

        // Removes boost if there is an active one and the countdown is over.
        if self.has_boost {
            self.boost_duration -= delta;

            if self.boost_duration <= 0.0 {
                self.has_boost = false;

                self.multiplier = 1.0;
            }
        }

        // Generates a new pipe and resets the countdown until the next one.
        if self.play_state.is_playing() && self.time_until_next_pipe <= 0.0 {
            let random_y = self.rng.gen_range(67.0..481.0);

            let pipe = PipeEntity::new(random_y);
            self.pipes.push_back(pipe);

            self.time_until_next_pipe = self.rng.gen_range(1.0..4.5) * 1000000000.0;
        }

        // Generates a new enemy and resets the countdown until the next one.
        if self.play_state.is_playing() && self.time_until_next_enemy <= 0.0 {
            let random_y = self.rng.gen_range(63.0..705.0);

            let enemy = EnemyEntity::new(random_y);
            self.enemies.push_back(enemy);

            self.time_until_next_enemy = self.rng.gen_range(6.0..12.0) * 1000000000.0;
        }

        // Create a new boost (if there are no active ones at the moment) and resets the countdown until the next one.
        if self.play_state.is_playing()
            && self.time_until_next_boost <= 0.0
            && self.has_boost == false
        {
            // The second random value is used for determining the type of the newly created boost.
            let random_y = self.rng.gen_range(48.0..720.0);
            let random_val = self.rng.gen_range(0.0..18.0);

            let boost = BoostEntity::new(random_y, random_val);
            self.boosts.push_back(boost);

            self.time_until_next_boost = self.rng.gen_range(10.0..30.0) * 1000000000.0;
        }

        // Gets the new state of the player (but stores it in a new variable to compare it with the previous one).
        let state = self.player.update(ctx, &self.play_state);

        // Checks if the player touches the ground and has a spare life to use.
        if self.player.hits_ground() && self.lifes > 1 {
            self.player.prevent_hitting_ground();
            self.lifes -= 1;
        }

        // Starts the game if it is not.
        if self.play_state.is_playing() == false && state.is_playing() {
            self.play_state = PlayState::Play;
        }

        // Updates pipes and marks these that need to be removed.
        for pipe in self.pipes.iter_mut() {
            pipe.update(self.multiplier);

            let pos = pipe.position;
            if self.player.zone.overlaps(&pipe.bottom_zone)
                || self.player.zone.overlaps(&pipe.top_zone)
            {
                self.lifes -= 1;

                if self.lifes > 0 {
                    pipe.is_passed = true;
                } else {
                    self.hit_pipe = true;
                }
            }

            if pos.x <= -(PIPE_WIDTH / 2.0) {
                self.score += 1;
                pipe.is_passed = true;
            }
        }

        // Updates enemies and marks these that need to be removed.
        for enemy in self.enemies.iter_mut() {
            enemy.update(self.multiplier);

            let pos = enemy.position;

            if self.player.zone.overlaps(&enemy.zone) {
                self.lifes -= 1;

                if self.lifes > 0 {
                    enemy.is_passed = true;
                } else {
                    self.hit_enemy = true;
                }
            }

            if pos.x <= -(ENEMY_WIDTH / 2.0) {
                enemy.is_passed = true;
            }
        }

        // Updates boosts and marks these that need to be removed.
        for boost in self.boosts.iter_mut() {
            boost.update();

            let pos = boost.position;

            if self.player.zone.overlaps(&boost.zone) {
                boost.is_collected = true;
                self.assets.boost_sound.play_detached(ctx).unwrap();

                match boost.effect {
                    BoostType::BonusLife => {
                        self.lifes += 1;
                    }
                    BoostType::SlowDown => {
                        self.has_boost = true;
                        self.boost_duration = BOOST_DURATION;
                        self.multiplier = 0.5;
                    }
                    BoostType::SpeedUp => {
                        self.has_boost = true;
                        self.boost_duration = BOOST_DURATION;
                        self.multiplier = 1.5;
                    }
                };
            }

            if pos.x <= -(BOOST_WIDTH / 2.0) {
                boost.is_passed = true;
            }
        }

        // Checks if the game is over.
        if self.is_over() {
            self.assets.death_sound.play_detached(ctx).unwrap();
            self.play_state.set_dead();
        }

        // Removes all pipes that are already passed.
        self.pipes.retain(|pipe| pipe.is_passed == false);

        // Removes all enemies that are already passed.
        self.enemies.retain(|enemy| enemy.is_passed == false);

        // Removes all boosts that are already passed or collected.
        self.boosts
            .retain(|boost| boost.is_passed == false && boost.is_collected == false);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult<()> {
        // Sets the background color to light blue before displaying the background image.
        let light_blue = graphics::Color::from_rgb(77, 193, 203);
        graphics::clear(ctx, light_blue);

        // Displays the background image.
        graphics::draw(
            ctx,
            &self.assets.background_image,
            graphics::DrawParam::default(),
        )?;

        // Draw parameters for the two different state logos. (StartScreen and Dead)
        let offset = Point2 { x: 0.5, y: 0.5 };
        let pos = Point2 {
            x: SCREEN_WIDTH / 2.0,
            y: SCREEN_HEIGHT / 4.0,
        };

        // Displays the logo during the StartScreen state.
        if self.play_state == PlayState::StartScreen {
            graphics::draw(
                ctx,
                &self.assets.logo_start_screen_image,
                graphics::DrawParam::default().dest(pos).offset(offset),
            )?;
        }

        // Displays 'Game Over' message and the best score during the Dead state.
        if self.play_state == PlayState::Dead {
            graphics::draw(
                ctx,
                &self.assets.logo_game_over_image,
                graphics::DrawParam::default().dest(pos).offset(offset),
            )?;

            let font = graphics::Font::new(ctx, "/FlappyBird.ttf")?;
            let mut text = graphics::Text::new(format!("Best score: {}", self.best_score));
            text.set_font(font, graphics::PxScale::from(50.0));

            let text_pos = Point2 {
                x: (SCREEN_WIDTH - text.width(ctx)) / 2.0,
                y: (SCREEN_HEIGHT - text.height(ctx)) / 2.0,
            };

            graphics::draw(
                ctx,
                &text,
                graphics::DrawParam::default()
                    .dest(text_pos)
                    .color(graphics::Color::BLACK),
            )?;
        }

        // Draws the player.
        self.player.draw(ctx, &self.assets)?;

        // Draws the pipes.
        for pipe in self.pipes.iter_mut() {
            pipe.draw(ctx, &self.assets)?;
        }

        // Draws the enemies.
        for enemy in self.enemies.iter_mut() {
            enemy.draw(ctx, &self.assets)?;
        }

        // Draws the boosts.
        for boost in self.boosts.iter_mut() {
            boost.draw(ctx, &self.assets)?;
        }

        // Drawss the scores.
        if self.play_state.is_playing() {
            self.draw_stats(ctx);
        }

        graphics::present(ctx)?;
        std::thread::yield_now();

        Ok(())
    }
}

fn main() {
    // Path to resources
    let path = path::PathBuf::from("./resources");

    // Setting the window size
    // I saw that it is possible to make the game resizeable but at this point I decided to keep the size strictly defined.
    let win_mode = WindowMode::default().dimensions(SCREEN_WIDTH, SCREEN_HEIGHT);

    // Customizing the window
    let win_setup = WindowSetup::default()
        .title("Flappy Ferrris")
        .icon("/icon.png");

    // Generating the configuration
    let conf = Conf {
        window_mode: win_mode,
        window_setup: win_setup,
        backend: Backend::default(),
        modules: ModuleConf::default(),
    };

    // Building the ContextBuilder and adding the resources path
    let (mut ctx, event_loop) = ContextBuilder::new("Flappy Ferris", "Georgi Sabev")
        .add_resource_path(path)
        .default_conf(conf.clone())
        .build()
        .unwrap();

    // Running the game
    let state = MainState::new(&mut ctx);
    event::run(ctx, event_loop, state);
}
