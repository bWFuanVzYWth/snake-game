const MAP_SIDE_LENGTH: usize = 16;
const MAP_SIZE: usize = MAP_SIDE_LENGTH * MAP_SIDE_LENGTH;

const DIRECTION_NONE: Position = Position { x: 0, y: 0 };
const DIRECTION_RIGHT: Position = Position { x: 1, y: 0 };
const DIRECTION_UP: Position = Position { x: 0, y: -1 };
const DIRECTION_LEFT: Position = Position { x: -1, y: 0 };
const DIRECTION_DOWN: Position = Position { x: 0, y: 1 };

const CELL_EMPTY: u8 = 0;
const CELL_FOOD: u8 = 1;
const CELL_SNAKE: u8 = 2;

const STATE_OVER: u8 = 0;
const STATE_READY: u8 = 1;
const STATE_RUN: u8 = 2;

#[derive(Copy, Clone, PartialEq, Debug)]
struct Position {
    x: i8,
    y: i8,
}

impl Position {
    pub const fn from_hash(hash: usize) -> Self {
        Self {
            x: (hash % MAP_SIDE_LENGTH) as i8,
            y: (hash / MAP_SIDE_LENGTH) as i8,
        }
    }

    pub const fn as_hash(&self) -> usize {
        self.y as usize * MAP_SIDE_LENGTH + self.x as usize
    }
}

#[derive(Debug)]
struct SnakeGame {
    direction: Position,                 // 记住前进方向
    map: [u8; MAP_SIZE],                 // 游戏地图
    tail_index: usize,                   // 蛇尾在`positions`的索引
    length: usize,                       // 蛇的长度
    hashed_positions: [usize; MAP_SIZE], // 用于O(1)复杂度维护蛇身与空位
    indices: [usize; MAP_SIZE],          // 用于O(1)复杂度查找`positions`中元素的`index`
}

impl Default for SnakeGame {
    /// 返回贪吃蛇的默认上下文。其实就是初始化游戏。
    fn default() -> Self {
        const SNAKE_POSITION: Position = Position {
            x: (MAP_SIDE_LENGTH / 2) as i8,
            y: (MAP_SIDE_LENGTH / 2) as i8,
        };

        let mut tmp = Self {
            direction: DIRECTION_NONE,
            map: [CELL_EMPTY; MAP_SIZE],
            tail_index: 0,
            length: 0,
            hashed_positions: std::array::from_fn(|i| i),
            indices: std::array::from_fn(|i| i),
        };

        // 生成初始蛇

        tmp.tail_index = SNAKE_POSITION.as_hash();
        tmp.push_snake_head(SNAKE_POSITION.as_hash());

        // 生成初始食物
        tmp.generate_food();

        tmp
    }
}

impl SnakeGame {
    /// 用于维护环形队列索引。
    ///
    /// 当`MAP_SIZE`是2的整数次幂时，除法可用位运算取代，甚至直接依赖数据类型的溢出截断
    const fn wrapping_offset(base: usize, offset: usize) -> usize {
        (base + offset) % MAP_SIZE
    }

    /// 在空位中随机选择一个位置
    fn random_food_hashed_position(&self) -> usize {
        let mut rng = rand::rng();
        let empty_indices_base = Self::wrapping_offset(self.tail_index, self.length);
        let empty_indices_length = MAP_SIZE - self.length;
        let empty_indices_random = Self::wrapping_offset(
            empty_indices_base,
            rand::Rng::random_range(&mut rng, 0..empty_indices_length),
        );
        self.hashed_positions[empty_indices_random]
    }

    /// 生成食物
    fn generate_food(&mut self) {
        let food_hash = self.random_food_hashed_position();
        self.map[food_hash] = CELL_FOOD;
    }

    /// 弹出蛇尾，更新相关数据结构
    const fn pop_snake_tail(&mut self) -> usize {
        // 仅弹出蛇尾，不涉及positions中元素的变动
        let tail_hash = self.hashed_positions[self.tail_index];

        // 移动指向蛇尾的索引
        self.tail_index = Self::wrapping_offset(self.tail_index, 1);

        // 杂项
        self.map[tail_hash] = CELL_EMPTY;
        self.length -= 1;

        tail_hash
    }

    /// 压入蛇头，更新相关的数据结构
    const fn push_snake_head(&mut self, head_hash: usize) {
        // 找到新的蛇头对应的元素
        let new_head_hash = head_hash;
        let new_head_index = self.indices[new_head_hash];

        // 找到因为会被覆写，所以需要迁移的元素
        let relocate_from_index = Self::wrapping_offset(self.tail_index, self.length);
        let relocate_from_hash = self.hashed_positions[relocate_from_index];

        // 交换元素
        self.hashed_positions[new_head_index] = self.hashed_positions[relocate_from_index];
        self.hashed_positions[relocate_from_index] = head_hash;

        // 维护因交换元素变化的索引
        self.indices[new_head_hash] = relocate_from_index;
        self.indices[relocate_from_hash] = new_head_index;

        // 杂项
        self.map[new_head_hash] = CELL_SNAKE;
        self.length += 1;
    }

    /// 游戏更新的主要逻辑
    ///
    /// 以下事件的先后顺序不可变更，否则可能产生严重的逻辑错误
    /// 1. 删除蛇尾
    /// 2. 插入蛇头
    /// 3. 生成食物
    fn update(&mut self, direction: Position) -> u8 {
        // 处理方向输入
        if direction != DIRECTION_NONE {
            self.direction = direction;
        }

        if self.direction == DIRECTION_NONE {
            return STATE_READY;
        }

        // 根据当前蛇头位置和方向输入，计算新的蛇头位置
        let head_index = Self::wrapping_offset(self.tail_index, self.length - 1);
        let head_position = Position::from_hash(self.hashed_positions[head_index]);
        let new_head_position = Position {
            x: (head_position.x + self.direction.x),
            y: (head_position.y + self.direction.y),
        };
        if new_head_position.x >= MAP_SIDE_LENGTH as i8
            || new_head_position.x < 0
            || new_head_position.y >= MAP_SIDE_LENGTH as i8
            || new_head_position.y < 0
        {
            return STATE_OVER;
        }

        let new_head_hash = new_head_position.as_hash();

        // 碰撞测试
        match self.map[new_head_hash] {
            CELL_EMPTY => {
                self.pop_snake_tail();
                self.push_snake_head(new_head_hash);

                STATE_RUN
            }
            CELL_FOOD => {
                if self.length >= MAP_SIZE - 1 {
                    return STATE_OVER;
                }

                self.push_snake_head(new_head_hash);
                self.generate_food();

                STATE_RUN
            }
            CELL_SNAKE => STATE_OVER,
            _ => {
                panic!("Invalid cell value");
            }
        }
    }

    /// 控制台输出
    ///
    /// 不参与核心逻辑，不必关心复杂度，此处仅使用最朴素的实现
    fn render(&self) {
        use std::fmt::Write;

        let border_line = "-".repeat(MAP_SIDE_LENGTH + 2);
        let mut output = String::with_capacity(MAP_SIZE + 4 * MAP_SIDE_LENGTH);

        let _ = write!(output, "\x1B[2J\x1B[1;1H");
        let _ = writeln!(output, "{border_line}");

        for i in 0..MAP_SIZE {
            let pos = i % MAP_SIDE_LENGTH;
            if pos == 0 {
                output.push('|');
            }

            let ch = match self.map[i] {
                CELL_EMPTY => ' ',
                CELL_SNAKE => '#',
                CELL_FOOD => 'F',
                _ => panic!("Invalid cell value"),
            };
            output.push(ch);

            if (pos + 1) == MAP_SIDE_LENGTH {
                let _ = writeln!(output, "|");
            }
        }

        let _ = writeln!(output, "{border_line}");

        print!("{output}");
    }
}

const UPDATE_INTERVAL_MILLIS: u64 = 250;

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let mut content = SnakeGame::default();
    let mut moves_count: usize = 0;

    content.render();

    loop {
        let direction = {
            let mut dir = DIRECTION_NONE;
            while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                    use crossterm::event::KeyCode;
                    dir = match key_event.code {
                        KeyCode::Up => DIRECTION_UP,
                        KeyCode::Down => DIRECTION_DOWN,
                        KeyCode::Left => DIRECTION_LEFT,
                        KeyCode::Right => DIRECTION_RIGHT,
                        _ => DIRECTION_NONE,
                    };
                }
            }
            dir
        };

        match content.update(direction) {
            STATE_OVER => {
                break;
            }
            STATE_READY => {}
            STATE_RUN => {
                moves_count += 1;
            }
            _ => panic!("Invalid Game State!"),
        }

        content.render();

        std::thread::sleep(std::time::Duration::from_millis(UPDATE_INTERVAL_MILLIS));
    }

    println!("Game over after {moves_count} moves");

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
