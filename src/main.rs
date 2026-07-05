mod config;
mod game;
mod render;
mod snake;
mod types;

#[cfg(feature = "ai")]
mod pathfinding;

use crate::config::MapConfig;
use crate::game::Game;
use crate::types::GameState;
use std::time::Duration;

/// 默认更新间隔（毫秒）
const UPDATE_INTERVAL_MS: u64 = 1;

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let config = MapConfig::new(16, 16);
    let mut game = Game::new(config, 3, 1);
    let mut render_buf = String::new();

    // 初始渲染
    game.render(&mut render_buf);
    print!("{render_buf}");

    loop {
        #[cfg(not(feature = "ai"))]
        let mut direction = None;

        // 键盘轮询：Ctrl+C 退出 + 手动模式方向输入
        while crossterm::event::poll(Duration::from_millis(0))? {
            if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                use crossterm::event::KeyCode;
                if key_event.code == KeyCode::Char('c')
                    && key_event
                        .modifiers
                        .contains(crossterm::event::KeyModifiers::CONTROL)
                {
                    println!("Interrupted after {} moves", game.moves_count());
                    crossterm::terminal::disable_raw_mode()?;
                    return Ok(());
                }
                #[cfg(not(feature = "ai"))]
                {
                    direction = game::key_to_direction(key_event.code);
                }
            }
        }

        #[cfg(feature = "ai")]
        let direction = pathfinding::next_dir(game.snake());

        let end_state = game.tick(direction);

        if matches!(end_state, GameState::Over | GameState::Won) {
            let label = match end_state {
                GameState::Won => "You win",
                _ => "Game over",
            };
            println!("{label} after {} moves", game.moves_count());
            break;
        }

        game.render(&mut render_buf);
        #[cfg(feature = "ai")]
        render_buf.push_str("[AI] ");
        print!("{render_buf}");

        std::thread::sleep(Duration::from_millis(UPDATE_INTERVAL_MS));
    }

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
