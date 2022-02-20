use ggez::{audio, graphics};
use ggez::{Context, GameResult};

// All the resources needed for playing sound effects and displaying graphics on screen.
pub struct Assets {
    pub ferris_stable_image: graphics::Image,
    pub ferris_jumping_image: graphics::Image,
    pub enemy_image: graphics::Image,
    pub boost_life_image: graphics::Image,
    pub boost_slow_down_image: graphics::Image,
    pub boost_speed_up_image: graphics::Image,
    pub pipe_top_image: graphics::Image,
    pub pipe_bottom_image: graphics::Image,
    pub background_image: graphics::Image,
    pub logo_start_screen_image: graphics::Image,
    pub logo_game_over_image: graphics::Image,

    pub boost_sound: audio::Source,
    pub death_sound: audio::Source,
}
impl Assets {
    pub fn new(ctx: &mut Context) -> GameResult<Assets> {
        let ferris_stable_image = graphics::Image::new(ctx, "/ferris_stable.png")?;
        let ferris_jumping_image = graphics::Image::new(ctx, "/ferris_jumping.png")?;
        let enemy_image = graphics::Image::new(ctx, "/enemy.png")?;
        let boost_life_image = graphics::Image::new(ctx, "/boost_life.png")?;
        let boost_slow_down_image = graphics::Image::new(ctx, "/boost_slow-down.png")?;
        let boost_speed_up_image = graphics::Image::new(ctx, "/boost_speed-up.png")?;
        let pipe_top_image = graphics::Image::new(ctx, "/pipe-top.png")?;
        let pipe_bottom_image = graphics::Image::new(ctx, "/pipe-bottom.png")?;
        let background_image = graphics::Image::new(ctx, "/background.png")?;
        let logo_start_screen_image = graphics::Image::new(ctx, "/logo_start_screen.png")?;
        let logo_game_over_image = graphics::Image::new(ctx, "/logo_game_over.png")?;

        let boost_sound = audio::Source::new(ctx, "/boost.ogg")?;
        let death_sound = audio::Source::new(ctx, "/death.ogg")?;

        Ok(Assets {
            ferris_stable_image,
            ferris_jumping_image,
            enemy_image,
            boost_life_image,
            boost_slow_down_image,
            boost_speed_up_image,
            pipe_top_image,
            pipe_bottom_image,
            background_image,
            logo_start_screen_image,
            logo_game_over_image,

            boost_sound,
            death_sound,
        })
    }
}
