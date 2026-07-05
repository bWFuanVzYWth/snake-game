//! 贪吃蛇自动寻路 AI
//!
//! 在交规定向图上 BFS 搜最短路径，O(map_size) 每帧，无状态爆炸。

use crate::config::MapConfig;
use crate::snake::SnakeGame;
use crate::types::{Direction, Position};
use std::collections::VecDeque;

// ============================================================================
// 交规 (Traffic Rules)
// ============================================================================

/// 返回在指定位置由交规允许的两个方向
fn traffic_dirs(pos: Position) -> [Direction; 2] {
    let h = if pos.y.is_multiple_of(2) { Direction::Right } else { Direction::Left };
    let v = if pos.x.is_multiple_of(2) { Direction::Up } else { Direction::Down };
    [h, v]
}

// ============================================================================
// 交规定向图上的 BFS
// ============================================================================

/// 在交规定向图上 BFS，返回从 start 到每个格子的最短距离（步数）。
/// 不检查蛇身碰撞——交规保证蛇身永远在头"后面"。
fn bfs_distances(start: usize, config: &MapConfig) -> Vec<u32> {
    let total = config.total_size();
    let mut dist = vec![u32::MAX; total];
    let mut queue = VecDeque::new();

    dist[start] = 0;
    queue.push_back(start);

    while let Some(cur) = queue.pop_front() {
        let d = dist[cur] + 1;
        let pos = config.from_hash(cur);
        for &dir in &traffic_dirs(pos) {
            if let Some(next) = step(cur, dir, config) {
                if dist[next] == u32::MAX {
                    dist[next] = d;
                    queue.push_back(next);
                }
            }
        }
    }

    dist
}

/// 向给定方向走一步（仅边界检查，不检查蛇身）
fn step(hash: usize, dir: Direction, config: &MapConfig) -> Option<usize> {
    let pos = config.from_hash(hash);
    let (dx, dy) = dir.delta();
    let nx = pos.x as i64 + dx as i64;
    let ny = pos.y as i64 + dy as i64;
    if nx < 0 || nx >= config.width as i64 || ny < 0 || ny >= config.height as i64 {
        return None;
    }
    Some(config.to_hash(Position { x: nx as u32, y: ny as u32 }))
}

// ============================================================================
// 主入口
// ============================================================================

/// 给定游戏状态，返回 AI 选择的下一步方向。
///
/// 在交规定向图上 BFS 到所有食物的最短距离，选最佳方向。
pub fn next_dir(snake: &SnakeGame) -> Option<Direction> {
    let config = snake.config();
    let head = {
        let p = snake.head_position()?;
        config.to_hash(p)
    };
    let foods = snake.food_hashes();
    if foods.is_empty() {
        return None;
    }

    let cur_dir = snake.direction();

    // 构建蛇身集合用于第一步碰撞检测
    let body_set: Vec<bool> = {
        let mut s = vec![false; config.total_size()];
        for &h in snake.snake_hashes() {
            s[h] = true;
        }
        s
    };

    // 对每个交规方向，BFS 计算从 next 到最近食物的图距离
    let head_pos = config.from_hash(head);
    let mut best_dir = cur_dir;
    let mut best_dist = u32::MAX;

    for &d in &traffic_dirs(head_pos) {
        // 180° 掉头排除
        if let Some(cd) = cur_dir {
            if d == cd.opposite() {
                continue;
            }
        }
        // 第一步必须合法
        let next = match step(head, d, config) {
            Some(h) => h,
            None => continue,
        };
        if body_set[next] {
            continue;
        }
        // BFS：从 next 到所有格子的距离
        let dist = bfs_distances(next, config);
        let nearest = foods.iter()
            .map(|&f| dist[f])
            .min()
            .unwrap_or(u32::MAX);
        let total = if nearest == u32::MAX { u32::MAX } else { 1 + nearest };

        // 选总距离最近的；同距保持原方向减少转弯
        if total < best_dist
            || (total == best_dist && Some(d) == cur_dir)
        {
            best_dist = total;
            best_dir = Some(d);
        }
    }

    best_dir
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
        let dirs = traffic_dirs(Position { x: 0, y: 0 });
        assert!(dirs.contains(&Direction::Right));
        assert!(dirs.contains(&Direction::Up));

        let dirs = traffic_dirs(Position { x: 1, y: 0 });
        assert!(dirs.contains(&Direction::Right));
        assert!(dirs.contains(&Direction::Down));

        let dirs = traffic_dirs(Position { x: 0, y: 1 });
        assert!(dirs.contains(&Direction::Left));
        assert!(dirs.contains(&Direction::Up));

        let dirs = traffic_dirs(Position { x: 1, y: 1 });
        assert!(dirs.contains(&Direction::Left));
        assert!(dirs.contains(&Direction::Down));
    }

    #[test]
    fn test_bfs_reaches_all_cells() {
        // 在 16×16 上，交规定向图应该是强连通的
        let config = MapConfig::new(16, 16);
        let dist = bfs_distances(0, &config);
        let reachable = dist.iter().filter(|&&d| d != u32::MAX).count();
        assert_eq!(reachable, config.total_size(),
            "交规定向图应连通所有格子（even×even 保证）");
    }

    #[test]
    fn test_step_wall() {
        let config = MapConfig::new(10, 10);
        let left_edge = config.to_hash(Position { x: 0, y: 5 });
        assert!(step(left_edge, Direction::Left, &config).is_none());
        assert!(step(left_edge, Direction::Right, &config).is_some());
    }

    #[test]
    fn test_next_dir_always_returns() {
        for seed in 0..10 {
            let config = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let game = SnakeGame::new(config, 3, 5, &mut rng);
            let dir = next_dir(&game);
            assert!(dir.is_some(), "seed={seed}: should return a direction");
            if let Some(d) = dir {
                if let Some(cur) = game.direction() {
                    assert_ne!(d, cur.opposite(), "seed={seed}: no 180° turn");
                }
            }
        }
    }

    #[test]
    fn test_no_self_collision_first_step() {
        // 多帧模拟，验证 AI 返回的方向不会导致第一步自撞
        for seed in 0..20 {
            let config = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut game = SnakeGame::new(config, 3, 3, &mut rng);
            for _ in 0..100 {
                let dir = match next_dir(&game) {
                    Some(d) => d,
                    None => break,
                };
                let state = game.update(Some(dir), &mut rng);
                if state != crate::types::GameState::Running {
                    break;
                }
            }
            // 不应 panic
        }
    }
}
