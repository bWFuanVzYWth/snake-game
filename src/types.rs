/// 地图上的二维坐标
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub x: u32,
    pub y: u32,
}

/// 蛇的移动方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    /// 返回此方向的 (dx, dy) 增量
    /// 注意：屏幕坐标系 Y 轴向下，因此 Up 对应 dy = -1
    pub const fn delta(self) -> (i32, i32) {
        match self {
            Direction::Up => (0, -1),
            Direction::Down => (0, 1),
            Direction::Left => (-1, 0),
            Direction::Right => (1, 0),
        }
    }

    /// 返回相反方向，用于防止 180 度掉头
    pub const fn opposite(self) -> Direction {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
        }
    }
}

/// 地图上单个格子的状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CellState {
    Empty = 0,
    Food = 1,
    Snake = 2,
}

/// 游戏的高级状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    /// 初始化完成，等待首次方向输入
    Ready,
    /// 游戏正在运行
    Running,
    /// 游戏结束（撞墙或撞到自己）
    Over,
    /// 蛇已填满整个地图（胜利）
    Won,
}
