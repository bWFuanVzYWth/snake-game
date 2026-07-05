use crate::snake::SnakeGame;
use crate::types::{CellState, Position};

/// 将当前游戏状态渲染到给定的 String 缓冲区
///
/// 缓冲区会被清空后重新填充。调用方可复用同一个 String 以避免每帧分配。
pub fn render(game: &SnakeGame, output: &mut String) {
    let config = game.config();
    let w = config.width as usize;
    let h = config.height as usize;
    let total = config.total_size();

    let border_line = "-".repeat(w + 2);
    let cap = 20 + (border_line.len() + 1) * 2 + total + h * 2;

    output.clear();
    output.reserve(cap.saturating_sub(output.capacity()));

    // ANSI 清屏 + 光标复位
    output.push_str("\x1B[2J\x1B[1;1H");
    output.push_str(&border_line);
    output.push('\n');

    for row in 0..h {
        output.push('|');
        for col in 0..w {
            let hash = config.to_hash(Position {
                x: col as u32,
                y: row as u32,
            });
            let ch = match game.cell_state(hash) {
                CellState::Empty => ' ',
                CellState::Snake => '#',
                CellState::Food => 'F',
            };
            output.push(ch);
        }
        output.push('|');
        output.push('\n');
    }

    output.push_str(&border_line);
    output.push('\n');
}
