use crate::config::MapConfig;
use crate::snake::SnakeGame;
use crate::types::{Direction, GameState};
use rand::rngs::ThreadRng;

/// 游戏管理器：封装蛇游戏状态、RNG 和步数计数
pub struct Game {
    snake: SnakeGame,
    moves_count: u64,
    rng: ThreadRng,
}

/// 将 crossterm KeyCode 转换为游戏方向
///
/// 返回 None 表示非方向键。
pub fn key_to_direction(key_code: crossterm::event::KeyCode) -> Option<Direction> {
    use crossterm::event::KeyCode;
    match key_code {
        KeyCode::Up => Some(Direction::Up),
        KeyCode::Down => Some(Direction::Down),
        KeyCode::Left => Some(Direction::Left),
        KeyCode::Right => Some(Direction::Right),
        _ => None,
    }
}

impl Game {
    /// 创建新的游戏实例
    pub fn new(
        config: MapConfig,
        initial_length: usize,
        food_count: usize,
    ) -> Self {
        let mut rng = rand::rng();
        let snake = SnakeGame::new(config, initial_length, food_count, &mut rng);
        Self {
            snake,
            moves_count: 0,
            rng,
        }
    }

    /// 返回移动步数
    pub fn moves_count(&self) -> u64 {
        self.moves_count
    }

    /// 返回蛇的引用（供渲染使用）
    pub fn snake(&self) -> &SnakeGame {
        &self.snake
    }

    /// 执行一个游戏 tick：应用方向并推进一帧，返回新状态
    pub fn tick(&mut self, direction: Option<Direction>) -> GameState {
        let state = self.snake.update(direction, &mut self.rng);
        if state == GameState::Running {
            self.moves_count += 1;
        }
        state
    }

    /// 渲染当前游戏画面到给定缓冲区
    pub fn render(&self, output: &mut String) {
        crate::render::render(&self.snake, output);
    }
}
