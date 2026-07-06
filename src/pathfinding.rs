//! 贪吃蛇自动寻路 AI
//!
//! **原理**：交规将 16×16 地图变成强连通有向图（每格 2 出边），BFS 找最短路径。
//! 蛇身仅用于第一步碰撞检测——BFS 距离只用作方向排序的启发值，不需要精确模拟蛇身。

use crate::config::MapConfig;
use crate::snake::SnakeGame;
use crate::types::{Direction, Position};
use std::collections::VecDeque;

// ============================================================================
// 交规 (Traffic Rules)
// ============================================================================

/// 返回 (x,y) 处交规允许的两个方向：
/// - 偶数行 → 右，奇数行 → 左
/// - 偶数列 → 上，奇数列 → 下
fn traffic_dirs(pos: Position) -> [Direction; 2] {
    let h = if pos.y.is_multiple_of(2) { Direction::Right } else { Direction::Left };
    let v = if pos.x.is_multiple_of(2) { Direction::Up } else { Direction::Down };
    [h, v]
}

// ============================================================================
// 交规定向图上的 BFS（纯图距离，不感知蛇身）
// ============================================================================

/// 在交规定向图上 BFS，返回 start 到每个格子的最短步数。
///
/// `body_idx[h]` = 格子 `h` 在蛇身（尾→头）中的索引，不在蛇身则为 `usize::MAX`。
/// BFS 距离 `d` 对应总步数 `1+d`，原始蛇身第 `i` 节在 `i <= d` 时已被弹出，格子可用。
fn bfs(start: usize, config: &MapConfig, body_idx: &[usize], body_len: usize) -> Vec<u32> {
    let n = config.total_size();
    let mut dist = vec![u32::MAX; n];
    let head_idx = body_len.saturating_sub(1);

    let mut q = VecDeque::new();
    dist[start] = 0;
    q.push_back(start);
    while let Some(cur) = q.pop_front() {
        let d = dist[cur] + 1;
        for &dir in &traffic_dirs(config.from_hash(cur)) {
            let nxt = match step(cur, dir, config) {
                Some(h) => h,
                None => continue,
            };
            if dist[nxt] != u32::MAX {
                continue;
            }
            let bi = body_idx[nxt];
            // 不在蛇身 / 是旧蛇头 / 已经弹出 → 可通行
            if bi != usize::MAX && bi != head_idx && bi > d as usize {
                continue;
            }
            dist[nxt] = d;
            q.push_back(nxt);
        }
    }
    dist
}

/// 向给定方向走一步（仅边界检查）
fn step(hash: usize, dir: Direction, cfg: &MapConfig) -> Option<usize> {
    let p = cfg.from_hash(hash);
    let (dx, dy) = dir.delta();
    let nx = p.x as i64 + dx as i64;
    let ny = p.y as i64 + dy as i64;
    if nx < 0 || nx >= cfg.width as i64 || ny < 0 || ny >= cfg.height as i64 {
        return None;
    }
    Some(cfg.to_hash(Position { x: nx as u32, y: ny as u32 }))
}

// ============================================================================
// 空白区连通性
// ============================================================================

/// 模拟一步（头占 next，尾放 body[0]）后，空白区是否保持单连通。
///
/// `body_idx[h]` = 格子 `h` 在蛇身中的索引，不在蛇身则为 `usize::MAX`。
/// 索引 0 是蛇尾，走一步后会被释放。
fn keeps_empty_connected(
    next: usize, body_idx: &[usize], cfg: &MapConfig,
) -> bool {
    let n = cfg.total_size();
    let mut open = vec![false; n];
    let mut start = None;
    for i in 0..n {
        // 不在蛇身 或 是蛇尾（即将释放）→ 空格；且不能是新头位置
        if (body_idx[i] == usize::MAX || body_idx[i] == 0) && i != next {
            open[i] = true;
            start = Some(i);
        }
    }
    let start = match start {
        Some(s) => s,
        None => return true, // 无空格
    };

    // 4-方向 flood-fill
    let mut stack = vec![start];
    let mut seen = vec![false; n];
    seen[start] = true;
    let mut cnt = 1;
    while let Some(cur) = stack.pop() {
        for &d in &[Direction::Right, Direction::Left, Direction::Up, Direction::Down] {
            if let Some(nbr) = step(cur, d, cfg) {
                if open[nbr] && !seen[nbr] {
                    seen[nbr] = true;
                    stack.push(nbr);
                    cnt += 1;
                }
            }
        }
    }
    cnt == open.iter().filter(|&&x| x).count()
}

// ============================================================================
// 主入口
// ============================================================================

/// 返回 AI 选择的下一步方向。
///
/// 优先级：不割裂空白区 > 到食物步数少 > 保持原方向。
pub fn next_dir(snake: &SnakeGame) -> Option<Direction> {
    let cfg = snake.config();
    let head = cfg.to_hash(snake.head_position()?);
    let foods = snake.food_hashes();
    if foods.is_empty() {
        return None;
    }
    let cur = snake.direction();

    let body: Vec<usize> = snake.snake_hashes().copied().collect();
    let body_len = body.len();
    let body_idx: Vec<usize> = {
        let mut idx = vec![usize::MAX; cfg.total_size()];
        for (i, &h) in body.iter().enumerate() {
            idx[h] = i;
        }
        idx
    };

    let mut best = cur;
    let mut best_total = u32::MAX;
    let mut best_conn = false; // 是否保持连通

    for &d in &traffic_dirs(cfg.from_hash(head)) {
        if Some(d) == cur.map(|c| c.opposite()) {
            continue;
        }
        let nxt = match step(head, d, cfg) {
            Some(h) => h,
            None => continue,
        };
        if body_idx[nxt] != usize::MAX {
            continue;
        }

        let dist = bfs(nxt, cfg, &body_idx, body_len);
        let nearest = foods.iter().map(|&f| dist[f]).min().unwrap_or(u32::MAX);
        let total = nearest.saturating_add(1);
        let conn = keeps_empty_connected(nxt, &body_idx, cfg);

        // 优先级：连通 > 不连通；连通中选距离短的；同距保持原方向
        let better = match (conn, best_conn) {
            (true, false) => true,
            (false, true) => false,
            _ => total < best_total || (total == best_total && Some(d) == cur),
        };
        if better {
            best_total = total;
            best_conn = conn;
            best = Some(d);
        }
    }

    best
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::MapConfig;
    use crate::snake::SnakeGame;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    #[test]
    fn test_traffic_rules() {
        assert!(traffic_dirs(Position { x: 0, y: 0 }).contains(&Direction::Right));
        assert!(traffic_dirs(Position { x: 0, y: 0 }).contains(&Direction::Up));
        assert!(traffic_dirs(Position { x: 1, y: 0 }).contains(&Direction::Down));
        assert!(traffic_dirs(Position { x: 0, y: 1 }).contains(&Direction::Left));
    }

    #[test]
    fn test_bfs_reaches_all() {
        let cfg = MapConfig::new(16, 16);
        let empty_idx = vec![usize::MAX; cfg.total_size()];
        let d = bfs(0, &cfg, &empty_idx, 0);
        assert_eq!(d.iter().filter(|&&x| x != u32::MAX).count(), 256,
            "even×even 交规图强连通");
    }

    #[test]
    fn test_step_wall() {
        let cfg = MapConfig::new(10, 10);
        let edge = cfg.to_hash(Position { x: 0, y: 5 });
        assert!(step(edge, Direction::Left, &cfg).is_none());
        assert!(step(edge, Direction::Right, &cfg).is_some());
    }

    #[test]
    fn test_never_180_turn() {
        for seed in 0..20 {
            let cfg = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let game = SnakeGame::new(cfg, 3, 5, &mut rng);
            if let (Some(d), Some(cur)) = (next_dir(&game), game.direction()) {
                assert_ne!(d, cur.opposite(), "seed={seed}");
            }
        }
    }

    #[test]
    fn test_no_first_step_self_collision() {
        for seed in 0..30 {
            let cfg = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut game = SnakeGame::new(cfg, 3, 3, &mut rng);
            for _ in 0..200 {
                let dir = match next_dir(&game) {
                    Some(d) => d,
                    None => break,
                };
                if game.update(Some(dir), &mut rng) != crate::types::GameState::Running {
                    break;
                }
            }
        }
    }

    #[test]
    fn test_always_returns_while_alive() {
        // 存活期间永不返回 None — 强连通保证
        for seed in 0..30 {
            let cfg = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut game = SnakeGame::new(cfg, 3, 5, &mut rng);
            for step in 0..500 {
                let dir = next_dir(&game);
                let state = game.update(dir, &mut rng);
                if state != crate::types::GameState::Running {
                    break;
                }
                assert!(dir.is_some(),
                    "seed={seed} step={step} len={}: None while Running",
                    game.length());
            }
        }
    }
}
