use crate::{
    apps::app::{App, AppCmd, Context},
    graphics::*,
    touch::TouchEvent,
};
use esp_hal::time::{Duration, Instant};

pub const MAX_LENGTH: usize = 256;

pub const FIELD_MIN_X: u16 = 2;
pub const FIELD_MIN_Y: u16 = 4;
pub const FIELD_MAX_X: u16 = 38;
pub const FIELD_MAX_Y: u16 = 29;

enum Direction {
    North,
    East,
    South,
    West,
}

impl Direction {
    fn left(&self) -> Self {
        match self {
            Direction::North => Direction::West,
            Direction::East => Direction::North,
            Direction::South => Direction::East,
            Direction::West => Direction::South,
        }
    }
    fn right(&self) -> Self {
        match self {
            Direction::North => Direction::East,
            Direction::East => Direction::South,
            Direction::South => Direction::West,
            Direction::West => Direction::North,
        }
    }
}

#[derive(PartialEq)]
enum GameState {
    Start,
    Playing,
    Dead,
}

pub struct SnakeApp {
    last_update: Instant,
    snake: [(u16, u16); MAX_LENGTH],
    length: u16,
    score: u16,
    dir_changed: bool,
    dir: Direction,
    state: GameState,
    food_pos: (u16, u16),
}

impl Default for SnakeApp {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            snake: [(0, 0); MAX_LENGTH],
            length: 0,
            score: 0,
            dir_changed: false,
            dir: Direction::East,
            state: GameState::Start,
            food_pos: (0, 0),
        }
    }
}

impl SnakeApp {
    fn reset_game(&mut self) {
        self.snake[0] = (10, 10);
        self.length = 1;
        self.dir = Direction::East;

        self.score = 0;
        self.state = GameState::Playing;

        self.update_food_pos();
    }

    fn update_position(&self, old_pos: (u16, u16), dir: &Direction) -> (u16, u16) {
        let mut new_pos = old_pos;
        match dir {
            Direction::North => {
                new_pos.1 -= 1;
            }
            Direction::East => {
                new_pos.0 += 1;
            }
            Direction::South => {
                new_pos.1 += 1;
            }
            Direction::West => {
                new_pos.0 -= 1;
            }
        }
        new_pos
    }

    fn check_game_over(&self, snake: &[(u16, u16); MAX_LENGTH], length: u16) -> bool {
        let (head_x, head_y) = snake[0];

        // Check if head is inside playing field
        if head_x < FIELD_MIN_X
            || head_x >= FIELD_MAX_X
            || head_y < FIELD_MIN_Y
            || head_y >= FIELD_MAX_Y
        {
            return true;
        }

        // Check if head collide with rest of snake
        for i in 1..length as usize {
            if head_x == snake[i].0 && head_y == snake[i].1 {
                return true;
            }
        }

        false
    }

    fn update_food_pos(&mut self) {
        // Use time since startup for random.
        let rand = Instant::now().duration_since_epoch().as_millis();

        self.food_pos = (
            FIELD_MIN_X + rand as u16 % (FIELD_MAX_X - FIELD_MIN_X),
            FIELD_MIN_Y + (rand as u16 / 2) % (FIELD_MAX_Y - FIELD_MIN_Y),
        );

        for snake_pos in self.snake.iter().take(self.length as usize) {
            if *snake_pos == self.food_pos {
                return self.update_food_pos();
            }
        }
    }
}

impl App for SnakeApp {
    fn init(&mut self, ctx: &mut Context) -> AppCmd {
        ctx.grid.clear(' ', BASE03, BASE03);

        ctx.buttons.clear();
        ctx.buttons.register_default_buttons();

        AppCmd::Dirty
    }
    fn update(&mut self, event: Option<TouchEvent>, ctx: &mut Context) -> AppCmd {
        if let Some(event) = event {
            if let Some(button_event) = ctx.buttons.update(&event) {
                match button_event {
                    crate::input::ButtonEvent::Up(id) => {
                        if id == "BACK" {
                            return AppCmd::SwitchApp(crate::apps::app::AppID::HomeApp);
                        }
                    }
                    _ => {}
                }
            } else {
                if let TouchEvent::Down { x, y: _ } = event {
                    // Reset Game
                    if self.state == GameState::Dead || self.state == GameState::Start {
                        self.reset_game();
                        ctx.grid.clear(' ', BASE03, BASE03);
                        return AppCmd::Dirty;
                    }

                    if !self.dir_changed {
                        if x < SCREEN_W / 2 {
                            self.dir = self.dir.left()
                        } else {
                            self.dir = self.dir.right()
                        }
                        self.dir_changed = true;
                    }
                }
            }
        }

        if self.last_update.elapsed() > Duration::from_millis(200)
            && self.state == GameState::Playing
        {
            let mut increase_score = false;
            self.dir_changed = false;

            for i in (0..self.length).rev() {
                if i == 0 {
                    self.snake[0] = self.update_position(self.snake[0], &self.dir);
                    if self.snake[0].0 == self.food_pos.0 && self.snake[0].1 == self.food_pos.1 {
                        increase_score = true;
                    }
                } else {
                    self.snake[i as usize] = self.snake[i as usize - 1];
                }
            }

            // Game Over
            if self.check_game_over(&self.snake, self.length) {
                self.state = GameState::Dead;
            }

            if increase_score {
                self.score += 1;

                if self.length < MAX_LENGTH as u16 {
                    self.snake[self.length as usize] = self.snake[self.length as usize - 1];
                    self.length += 1;
                }

                self.update_food_pos();
            }

            self.last_update = Instant::now();
            return AppCmd::Dirty;
        }
        AppCmd::None
    }
    fn render(&mut self, ctx: &mut Context) {
        // Playing field.
        for grid_x in FIELD_MIN_X..FIELD_MAX_X {
            for grid_y in FIELD_MIN_Y..FIELD_MAX_Y {
                ctx.grid.put_char(grid_x, grid_y, ' ', BASE01, BASE01);
            }
        }

        // TODO: Move score to status bar.
        ctx.grid.center_str(
            2,
            &heapless::format!(9; "Score: {}", self.score).unwrap_or_default(),
            BASE3,
            CYAN,
        );

        for i in 0..self.length {
            let (x, y) = self.snake[i as usize];

            ctx.grid.put_char(x, y, 'X', VIOLET, BLUE);
        }

        ctx.grid
            .put_char(self.food_pos.0, self.food_pos.1, '#', GREEN, BASE01);

        match self.state {
            GameState::Start => {
                ctx.grid.center_str(10, "SNAKE", BASE3, BLUE);

                ctx.grid.center_str(13, "Tap to rotate", BASE1, BASE01);
                ctx.grid.center_str(14, "<- LEFT | RIGHT ->", BASE1, BASE01);
                ctx.grid.center_str(16, "Tap to start", BASE3, BASE01);
            }
            GameState::Dead => {
                ctx.grid.center_str(14, "GAME OVER!", BASE3, RED);
                ctx.grid.center_str(16, "Tap to reset", BASE3, BASE01);
            }
            _ => {}
        }
    }
    fn get_name(&self) -> &'static str {
        "SNAKE"
    }
}
