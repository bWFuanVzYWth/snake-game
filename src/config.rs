use crate::types::Position;

/// 地图维度配置与坐标-哈希转换工具
#[derive(Debug, Clone)]
pub struct MapConfig {
    pub width: u32,
    pub height: u32,
    /// total_size = width * height，缓存以避免重复计算
    total_size: usize,
}

impl MapConfig {
    /// 创建新的地图配置
    ///
    /// # Panics
    /// width 或 height 为 0 时 panic
    pub fn new(width: u32, height: u32) -> Self {
        assert!(width > 0, "地图宽度必须大于 0");
        assert!(height > 0, "地图高度必须大于 0");
        let total_size = (width as usize)
            .checked_mul(height as usize)
            .expect("地图总格数溢出 usize");
        Self {
            width,
            height,
            total_size,
        }
    }

    /// 地图总格数
    #[inline]
    pub const fn total_size(&self) -> usize {
        self.total_size
    }

    /// 将二维坐标转换为一维哈希索引（行优先）
    #[inline]
    pub const fn to_hash(&self, pos: Position) -> usize {
        pos.y as usize * self.width as usize + pos.x as usize
    }

    /// 将一维哈希索引还原为二维坐标
    #[inline]
    #[allow(clippy::wrong_self_convention)]
    pub const fn from_hash(&self, hash: usize) -> Position {
        Position {
            x: (hash % self.width as usize) as u32,
            y: (hash / self.width as usize) as u32,
        }
    }

    /// 检查坐标是否在地图边界内
    #[inline]
    pub const fn contains(&self, pos: Position) -> bool {
        pos.x < self.width && pos.y < self.height
    }
}
