use crate::config::MapConfig;
use crate::types::{CellState, Direction, GameState, Position};
use rand::Rng;
use std::collections::VecDeque;

/// 贪吃蛇游戏的核心数据结构
///
/// 所有逐帧操作均为 O(1) 时间复杂度：
///
/// - **蛇移动**: VecDeque 的 push_back / pop_front
/// - **碰撞检测**: map[hash] 直接查表
/// - **食物生成**: 从 `empty_cells` 随机选一个再 swap-remove
///   （通过 `empty_indices` 反向查找实现 O(1) 移除）
/// - **食物被吃**: 从 `food_hashes` swap-remove，蛇头进入蛇身
///
/// 内存占用约 3×total_size×sizeof(usize) + total_size×1 字节。
/// 1000×1000 地图约 25 MB。
#[derive(Debug)]
pub struct SnakeGame {
    /// 地图配置（宽度、高度、哈希工具）
    config: MapConfig,
    /// 当前移动方向；None 表示尚未收到方向输入
    direction: Option<Direction>,
    /// 地图格子状态数组
    map: Vec<CellState>,
    /// 蛇身队列：队首为蛇尾，队尾为蛇头
    snake_body: VecDeque<usize>,
    /// 空格列表：所有当前为 Empty 的格子 hash
    empty_cells: Vec<usize>,
    /// 反向查找表：hash → 在 empty_cells 中的索引
    /// 值为 usize::MAX 表示该格子不在 empty_cells 中
    empty_indices: Vec<usize>,
    /// 当前食物位置的 hash 列表
    food_hashes: Vec<usize>,
}

impl SnakeGame {
    /// 创建一个新的贪吃蛇游戏实例
    ///
    /// # 参数
    /// - `config`: 地图配置
    /// - `initial_length`: 蛇的初始长度（至少为 1）
    /// - `food_count`: 地图上维持的食物数量
    /// - `rng`: 随机数生成器（用于初始食物放置）
    ///
    /// # Panics
    /// - `initial_length` 为 0
    /// - `initial_length + food_count > total_size`
    pub fn new(
        config: MapConfig,
        initial_length: usize,
        food_count: usize,
        rng: &mut impl Rng,
    ) -> Self {
        assert!(initial_length > 0, "蛇的初始长度必须大于 0");
        let total = config.total_size();
        assert!(
            initial_length + food_count <= total,
            "初始蛇身({initial_length}) + 食物({food_count}) 超过地图总格数({total})"
        );

        // 预先计算蛇的初始位置（在 config 被 move 之前）
        let center_x = config.width / 2;
        let center_y = config.height / 2;
        let tail_x = center_x.saturating_sub(initial_length as u32 - 1);

        let mut game = Self {
            config,
            direction: Some(Direction::Right),
            map: vec![CellState::Empty; total],
            snake_body: VecDeque::with_capacity(initial_length + 100),
            empty_cells: (0..total).collect(),
            empty_indices: (0..total).collect(),
            food_hashes: Vec::with_capacity(food_count),
        };

        // 放置蛇身
        for i in 0..initial_length as u32 {
            let seg_hash = game.config.to_hash(Position {
                x: tail_x + i,
                y: center_y,
            });
            game.map[seg_hash] = CellState::Snake;
            game.remove_from_empty(seg_hash);
            game.snake_body.push_back(seg_hash);
        }

        // 生成初始食物
        for _ in 0..food_count {
            game.spawn_food(rng);
        }

        debug_check_invariants(&game);
        game
    }

    // ========================================================================
    // 访问器
    // ========================================================================

    /// 返回地图配置的引用
    pub fn config(&self) -> &MapConfig {
        &self.config
    }

    /// 返回当前蛇的长度
    pub fn length(&self) -> usize {
        self.snake_body.len()
    }

    /// 返回当前食物数量
    pub fn food_count(&self) -> usize {
        self.food_hashes.len()
    }

    /// 返回蛇头的位置
    pub fn head_position(&self) -> Option<Position> {
        self.snake_body.back().map(|&h| self.config.from_hash(h))
    }

    /// 返回蛇身所有段的迭代器（从尾到头）
    pub fn snake_hashes(&self) -> impl DoubleEndedIterator<Item = &usize> + '_ {
        self.snake_body.iter()
    }

    /// 返回当前方向
    pub fn direction(&self) -> Option<Direction> {
        self.direction
    }

    /// 获取指定 hash 位置的格子状态
    pub fn cell_state(&self, hash: usize) -> CellState {
        self.map[hash]
    }

    /// 返回所有食物 hash 的切片
    pub fn food_hashes(&self) -> &[usize] {
        &self.food_hashes
    }

    // ========================================================================
    // 核心操作（均为 O(1)）
    // ========================================================================

    /// 从空格列表中移除指定 hash（O(1) swap-remove）
    fn remove_from_empty(&mut self, hash: usize) {
        let idx = self.empty_indices[hash];
        if idx == usize::MAX {
            return; // 已经被移除
        }
        let last_idx = self.empty_cells.len() - 1;
        let last_hash = self.empty_cells[last_idx];
        // swap-remove
        self.empty_cells.swap_remove(idx);
        if idx != last_idx {
            // 更新被移动元素的索引
            self.empty_indices[last_hash] = idx;
        }
        self.empty_indices[hash] = usize::MAX;
    }

    /// 将一个 hash 添加到空格列表中
    fn add_to_empty(&mut self, hash: usize) {
        self.empty_indices[hash] = self.empty_cells.len();
        self.empty_cells.push(hash);
    }

    /// 从空格列表中随机选一个位置生成食物（O(1)）
    fn spawn_food(&mut self, rng: &mut impl Rng) {
        if self.empty_cells.is_empty() {
            return;
        }
        let idx = rand::Rng::random_range(rng, 0..self.empty_cells.len());
        let food_hash = self.empty_cells[idx];
        self.remove_from_empty(food_hash);
        self.map[food_hash] = CellState::Food;
        self.food_hashes.push(food_hash);
    }

    /// 吃掉指定位置的食物：线性查找后 swap-remove
    ///
    /// 查找是 O(food_count)，对于典型食物数量（< 100）足够快。
    /// 如需严格 O(1)，可额外维护 food_indices 反向查找表。
    fn consume_food(&mut self, food_hash: usize) {
        if let Some(idx) = self.food_hashes.iter().position(|&h| h == food_hash) {
            self.food_hashes.swap_remove(idx);
        }
    }

    /// 蛇尾前进一步（正常移动时调用）
    fn advance_tail(&mut self) {
        if let Some(tail_hash) = self.snake_body.pop_front() {
            self.map[tail_hash] = CellState::Empty;
            self.add_to_empty(tail_hash);
        }
    }

    /// 蛇头前进一步（移动和吃食时均调用）
    fn advance_head(&mut self, head_hash: usize) {
        self.remove_from_empty(head_hash);
        self.map[head_hash] = CellState::Snake;
        self.snake_body.push_back(head_hash);
    }

    /// 游戏更新的主逻辑
    ///
    /// 处理方向输入、蛇移动、碰撞检测、食物处理。
    /// 返回更新后的游戏状态。
    pub fn update(
        &mut self,
        direction: Option<Direction>,
        rng: &mut impl Rng,
    ) -> GameState {
        // 处理方向输入：更新方向，防止 180 度掉头
        if let Some(dir) = direction
            && self.direction.is_none_or(|d| dir != d.opposite())
        {
            self.direction = Some(dir);
        }

        let dir = match self.direction {
            None => return GameState::Ready,
            Some(d) => d,
        };

        // 计算新蛇头位置（用 i64 做带符号运算避免 u32 下溢）
        let head_hash = *self.snake_body.back().unwrap();
        let head_pos = self.config.from_hash(head_hash);
        let (dx, dy) = dir.delta();

        let new_x = head_pos.x as i64 + dx as i64;
        let new_y = head_pos.y as i64 + dy as i64;

        // 边界检查
        if new_x < 0
            || new_x >= self.config.width as i64
            || new_y < 0
            || new_y >= self.config.height as i64
        {
            return GameState::Over;
        }

        let new_pos = Position {
            x: new_x as u32,
            y: new_y as u32,
        };
        let new_hash = self.config.to_hash(new_pos);

        // 碰撞检测与处理
        match self.map[new_hash] {
            CellState::Empty => {
                self.advance_tail();
                self.advance_head(new_hash);
                debug_check_invariants(self);
                GameState::Running
            }
            CellState::Food => {
                // 检查是否即将填满地图
                if self.snake_body.len() >= self.config.total_size() - 1 {
                    // 蛇吃掉最后一份食物后填满全图
                    self.consume_food(new_hash);
                    self.advance_head(new_hash);
                    debug_check_invariants(self);
                    return GameState::Won;
                }

                self.consume_food(new_hash);
                self.advance_head(new_hash);
                // 补充食物以维持目标数量
                self.spawn_food(rng);
                debug_check_invariants(self);
                GameState::Running
            }
            CellState::Snake => {
                debug_check_invariants(self);
                GameState::Over
            }
        }
    }
}

// ============================================================================
// Debug 不变式检查
// ============================================================================

/// Debug 模式下的数据结构不变式检查
///
/// 仅在 debug_assertions 启用时编译，release 构建中完全移除。
#[cfg(debug_assertions)]
fn debug_check_invariants(game: &SnakeGame) {
    let total = game.config.total_size();

    // 1. map 中蛇身格数 == snake_body 长度
    let snake_in_map = game.map.iter().filter(|&&c| c == CellState::Snake).count();
    debug_assert_eq!(
        snake_in_map,
        game.snake_body.len(),
        "map 蛇身格数 ({snake_in_map}) ≠ snake_body 长度 ({})",
        game.snake_body.len(),
    );

    // 2. map 中食物格数 == food_hashes 长度
    let food_in_map = game.map.iter().filter(|&&c| c == CellState::Food).count();
    debug_assert_eq!(
        food_in_map,
        game.food_hashes.len(),
        "map 食物格数 ({food_in_map}) ≠ food_hashes 长度 ({})",
        game.food_hashes.len(),
    );

    // 3. snake_body 中的每个 hash 在 map 中都标记为 Snake
    for &h in &game.snake_body {
        debug_assert_eq!(
            game.map[h],
            CellState::Snake,
            "蛇身 hash {h} 在 map 中为 {:?}，应为 Snake",
            game.map[h],
        );
    }

    // 4. food_hashes 中的每个 hash 在 map 中都标记为 Food
    for &h in &game.food_hashes {
        debug_assert_eq!(
            game.map[h],
            CellState::Food,
            "食物 hash {h} 在 map 中为 {:?}，应为 Food",
            game.map[h],
        );
    }

    // 5. empty_cells 中的每个 hash 在 map 中都标记为 Empty
    for &h in &game.empty_cells {
        debug_assert_eq!(
            game.map[h],
            CellState::Empty,
            "空格 hash {h} 在 map 中为 {:?}，应为 Empty",
            game.map[h],
        );
    }

    // 6. 三种分类不重叠，且覆盖所有格子
    let mut seen = vec![0u8; total];
    for &h in &game.snake_body {
        seen[h] += 1;
    }
    for &h in &game.food_hashes {
        seen[h] += 1;
    }
    for &h in &game.empty_cells {
        seen[h] += 1;
    }
    for (i, &count) in seen.iter().enumerate() {
        debug_assert_eq!(
            count, 1,
            "hash {i} 被分类了 {count} 次（应为恰好 1 次：蛇身/食物/空格）"
        );
    }
}

#[cfg(not(debug_assertions))]
#[inline(always)]
fn debug_check_invariants(_game: &SnakeGame) {}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::SmallRng;
    use rand::SeedableRng;

    /// 使用固定种子的 RNG 创建测试用游戏
    fn make_game(
        width: u32,
        height: u32,
        initial_length: usize,
        food_count: usize,
    ) -> (SnakeGame, SmallRng) {
        let config = MapConfig::new(width, height);
        let mut rng = SmallRng::seed_from_u64(42);
        let game = SnakeGame::new(config, initial_length, food_count, &mut rng);
        (game, rng)
    }

    #[test]
    fn test_initial_state() {
        let (game, _) = make_game(16, 16, 3, 1);
        assert_eq!(game.length(), 3);
        assert_eq!(game.food_count(), 1);
        assert_eq!(game.direction(), Some(Direction::Right));
        // 蛇应位于地图中央
        let head = game.head_position().unwrap();
        assert_eq!(head.y, 8);
        assert_eq!(head.x, 8); // 中心 x=8，初始长度3: tail=6,7, head=8
    }

    #[test]
    fn test_move_empty_cell() {
        let config = MapConfig::new(16, 16);
        let mut rng = SmallRng::seed_from_u64(42);
        // 不放食物，避免蛇出生就碰到食物
        let mut game = SnakeGame::new(config, 3, 0, &mut rng);
        let initial_len = game.length();
        let state = game.update(Some(Direction::Right), &mut rng);
        assert_eq!(state, GameState::Running);
        assert_eq!(game.length(), initial_len); // 移动不改变长度
    }

    #[test]
    fn test_eat_food_grows_snake() {
        let config = MapConfig::new(10, 10);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut game = SnakeGame::new(config, 3, 20, &mut rng);
        let initial_len = game.length();
        let mut grew = false;
        for _ in 0..20 {
            let state = game.update(Some(Direction::Right), &mut rng);
            if state != GameState::Running {
                break;
            }
            if game.length() > initial_len {
                grew = true;
                break;
            }
        }
        assert!(grew, "蛇碰到食物后应该增长");
    }

    #[test]
    fn test_wall_collision_up() {
        let config = MapConfig::new(10, 10);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut game = SnakeGame::new(config, 3, 0, &mut rng);
        for _ in 0..20 {
            let state = game.update(Some(Direction::Up), &mut rng);
            if state == GameState::Over {
                return;
            }
        }
        panic!("蛇应该撞墙而死");
    }

    #[test]
    fn test_wall_collision_left() {
        let config = MapConfig::new(10, 10);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut game = SnakeGame::new(config, 3, 0, &mut rng);
        for _ in 0..20 {
            let state = game.update(Some(Direction::Left), &mut rng);
            if state == GameState::Over {
                return;
            }
        }
        panic!("蛇应该撞墙而死");
    }

    #[test]
    fn test_no_180_turn() {
        let (mut game, mut rng) = make_game(16, 16, 3, 0);
        let state = game.update(Some(Direction::Left), &mut rng);
        assert_eq!(state, GameState::Running);
        assert_eq!(game.direction(), Some(Direction::Right));
    }

    #[test]
    fn test_food_count_maintained() {
        let config = MapConfig::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut game = SnakeGame::new(config, 3, 3, &mut rng);
        assert_eq!(game.food_count(), 3);

        for _ in 0..50 {
            let state = game.update(Some(Direction::Right), &mut rng);
            if state != GameState::Running {
                break;
            }
            assert_eq!(
                game.food_count(),
                3,
                "食物被吃后应立即补充以维持目标数量"
            );
        }
    }

    #[test]
    fn test_multiple_foods_no_overlap() {
        let config = MapConfig::new(50, 50);
        let mut rng = SmallRng::seed_from_u64(42);
        let game = SnakeGame::new(config, 3, 100, &mut rng);

        let foods = game.food_hashes();
        assert_eq!(foods.len(), 100);

        // 验证无重复
        let mut sorted = foods.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 100, "食物位置不应重复");

        // 验证食物不与蛇身重叠
        for &f in foods {
            assert_eq!(game.cell_state(f), CellState::Food);
        }
    }

    #[test]
    fn test_food_positions_are_valid() {
        let config = MapConfig::new(20, 20);
        let mut rng = SmallRng::seed_from_u64(99);
        let game = SnakeGame::new(config.clone(), 5, 10, &mut rng);

        let foods = game.food_hashes();
        assert_eq!(foods.len(), 10);

        for &f in foods {
            let pos = config.from_hash(f);
            assert!(config.contains(pos), "食物位置应在边界内");
        }
    }

    #[test]
    fn test_random_spawn_no_panic() {
        let configs = [
            (5, 5, 3, 2),
            (10, 10, 5, 10),
            (50, 50, 10, 100),
            (100, 100, 20, 50),
        ];

        for (w, h, len, food) in configs {
            let config = MapConfig::new(w, h);
            let mut rng = SmallRng::seed_from_u64(12345);
            let mut game = SnakeGame::new(config, len, food, &mut rng);
            for _ in 0..10 {
                let dirs = [
                    Direction::Right,
                    Direction::Down,
                    Direction::Left,
                    Direction::Up,
                ];
                let dir = dirs[rand::Rng::random_range(&mut rng, 0..4)];
                let state = game.update(Some(dir), &mut rng);
                if state != GameState::Running {
                    break;
                }
            }
        }
    }

    #[test]
    fn test_self_collision_small_map_long_snake() {
        // 在 6×6 地图上放长度 5 的蛇，走环路迫使蛇头撞到身体
        let config = MapConfig::new(6, 6);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut game = SnakeGame::new(config, 5, 0, &mut rng);

        // 蛇初始朝右，头在中心 (3,3)，尾在 (-1,3)→saturate到(0,3)
        // 身体: (0,3),(1,3),(2,3),(3,3),(4,3) 实际上 center_x=3, tail_x=-1→0
        // 重新算: center(3,3), tail(0,3), 身体(0..5,3) head=(4,3)
        // 移动: Right→(5,3), Down→(5,4), Left→(4,4), Left→(3,4), Up→(3,3)
        // 此时 (3,3) 是否还在蛇身中？初始 body 包含 (3,3) 但 tail 已弹了 4 次
        // 用更简单的方法：疯狂绕圈，蛇长填满大半地图，必撞

        let cycle = [
            Direction::Right,
            Direction::Down,
            Direction::Left,
            Direction::Up,
        ];
        for i in 0..200 {
            let dir = cycle[i % 4];
            let state = game.update(Some(dir), &mut rng);
            if state != GameState::Running {
                return; // 预期：撞墙或撞到自己
            }
        }
        panic!("长度5的蛇在6×6地图上绕圈200步后应已结束游戏");
    }

    #[test]
    fn test_game_state_ready() {
        let config = MapConfig::new(10, 10);
        let total = config.total_size();
        let mut game = SnakeGame {
            config,
            direction: None,
            map: vec![CellState::Empty; total],
            snake_body: VecDeque::new(),
            empty_cells: (0..total).collect(),
            empty_indices: (0..total).collect(),
            food_hashes: Vec::new(),
        };
        let mut rng = SmallRng::seed_from_u64(42);
        let state = game.update(None, &mut rng);
        assert_eq!(state, GameState::Ready);
    }

    #[test]
    fn test_empty_cells_coverage() {
        let (game, _) = make_game(16, 16, 5, 3);
        let total = game.config.total_size();
        // snake + food + empty = total
        assert_eq!(
            game.snake_body.len() + game.food_hashes.len() + game.empty_cells.len(),
            total,
            "蛇身 + 食物 + 空格 应覆盖所有格子"
        );
    }

    #[test]
    fn test_head_position_consistency() {
        let (game, _) = make_game(20, 20, 5, 10);
        let head = game.head_position().unwrap();
        let head_hash = game.config.to_hash(head);
        assert_eq!(*game.snake_body.back().unwrap(), head_hash);
        assert_eq!(game.map[head_hash], CellState::Snake);
    }
}
