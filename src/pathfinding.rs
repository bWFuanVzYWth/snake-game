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
///
/// 模拟蛇尾让路：蛇身第 i 节（i=0 为尾）在 d > i 步后已被弹出，格子可用。
/// 展开邻居时优先贴障碍的格子。
fn bfs_distances(start: usize, config: &MapConfig, body: &[usize], body_set: &[bool]) -> Vec<u32> {
    let total = config.total_size();
    let mut dist = vec![u32::MAX; total];
    let mut queue = VecDeque::new();
    // body_idx[hash] = 该格在蛇身中的索引（0=尾），不在蛇身中为 usize::MAX
    let mut body_idx = vec![usize::MAX; total];
    for (i, &h) in body.iter().enumerate() {
        body_idx[h] = i;
    }
    // 蛇头总是可用（起点）
    let head_idx = body.len().saturating_sub(1);

    dist[start] = 0;
    queue.push_back(start);

    while let Some(cur) = queue.pop_front() {
        let d = dist[cur] + 1;
        let pos = config.from_hash(cur);
        let mut neighbors: Vec<usize> = traffic_dirs(pos)
            .iter()
            .filter_map(|&dir| step(cur, dir, config))
            .filter(|&n| {
                if dist[n] != u32::MAX {
                    return false;
                }
                // 蛇身格：d 步后前 d 节已被弹出，d > idx 时可用
                let bi = body_idx[n];
                if bi == usize::MAX { return true; }      // 不在蛇身中
                if bi == head_idx { return true; }         // 蛇头（起点）
                bi < d as usize // 已经弹出了
            })
            .collect();
        // 贴障碍的格子优先入队（同距离内优先处理）
        neighbors.sort_by_key(|&n| {
            let adj = count_obstacle_neighbors(n, config, body_set);
            std::cmp::Reverse(adj)
        });
        for n in neighbors {
            dist[n] = d;
            queue.push_back(n);
        }
    }

    dist
}

/// 统计某格子四邻域中"障碍"数：墙 + 蛇身。
/// 贴墙和贴蛇身都能减少地图割裂。
fn count_obstacle_neighbors(hash: usize, config: &MapConfig, body_set: &[bool]) -> u32 {
    let mut count = 0;
    let pos = config.from_hash(hash);
    for &d in &[Direction::Right, Direction::Left, Direction::Up, Direction::Down] {
        let (dx, dy) = d.delta();
        let nx = pos.x as i64 + dx as i64;
        let ny = pos.y as i64 + dy as i64;
        if nx < 0 || nx >= config.width as i64 || ny < 0 || ny >= config.height as i64 {
            count += 1; // 墙
        } else {
            let n = config.to_hash(Position { x: nx as u32, y: ny as u32 });
            if body_set[n] {
                count += 1; // 蛇身
            }
        }
    }
    count
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

    // 蛇身数组（尾→头），用于时间感知的碰撞检测
    let body: Vec<usize> = snake.snake_hashes().copied().collect();
    let body_set: Vec<bool> = {
        let mut s = vec![false; config.total_size()];
        for &h in &body {
            s[h] = true;
        }
        s
    };

    // 第一步碰撞：只能用 body_set 静态检查（此时还没开始模拟弹尾）
    let first_step_blocked = |h: usize| -> bool {
        body_set[h] && h != *body.last().unwrap() // 不能是蛇头自身
    };

    // 对每个交规方向，BFS 计算从 next 到最近食物的图距离
    let head_pos = config.from_hash(head);
    let mut best_dir = cur_dir;
    let mut best_dist = u32::MAX;
    let mut best_hug = 0u32;

    for &d in &traffic_dirs(head_pos) {
        if let Some(cd) = cur_dir {
            if d == cd.opposite() {
                continue;
            }
        }
        let next = match step(head, d, config) {
            Some(h) => h,
            None => continue,
        };
        if first_step_blocked(next) {
            continue;
        }
        // BFS：时间感知蛇身（d 步后前 d 节已弹出）
        let dist = bfs_distances(next, config, &body, &body_set);
        let nearest = foods.iter()
            .map(|&f| dist[f])
            .min()
            .unwrap_or(u32::MAX);
        let total = if nearest == u32::MAX { u32::MAX } else { 1 + nearest };
        let hug = count_obstacle_neighbors(next, config, &body_set);

        // 优先级：距离近 > 贴障碍多 > 保持原方向（减少转向）
        if total < best_dist
            || (total == best_dist && hug > best_hug)
            || (total == best_dist && hug == best_hug && Some(d) == cur_dir)
        {
            best_dist = total;
            best_hug = hug;
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
        let empty_body: Vec<usize> = vec![];
        let empty_set = vec![false; config.total_size()];
        let dist = bfs_distances(0, &config, &empty_body, &empty_set);
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
        }
    }

    /// 验证：存活期间 next_dir 永不返回 None——交规图强连通 + 时间感知蛇身保证总有路
    #[test]
    fn test_never_returns_none_while_alive() {
        for seed in 0..30 {
            let config = MapConfig::new(16, 16);
            let mut rng = SmallRng::seed_from_u64(seed);
            let mut game = SnakeGame::new(config, 3, 5, &mut rng);
            for step in 0..500 {
                let dir = next_dir(&game);
                let state = game.update(dir, &mut rng);
                match state {
                    crate::types::GameState::Running => {
                        assert!(dir.is_some(),
                            "seed={seed} step={step}: AI returned None while game is Running. \
                             head={:?} len={} foods={}",
                            game.head_position(), game.length(), game.food_count());
                    }
                    _ => break, // Over/Won — 游戏结束，不需要方向了
                }
            }
        }
    }
}
