const SIDE_LENGTH: usize = 16;
const MAP_SIZE: usize = SIDE_LENGTH * SIDE_LENGTH;

const PAUSE: Position = Position { x: 0, y: 0 };
const RIGHT: Position = Position { x: 1, y: 0 };
const UP: Position = Position { x: 0, y: 1 };
const LEFT: Position = Position { x: u8::MAX, y: 0 };
const DOWN: Position = Position { x: 0, y: u8::MAX };

const EMPTY: u8 = 0;
const SNAKE: u8 = 1;
const FOOD: u8 = 2;

#[derive(Copy, Clone, Default, PartialEq)]
struct Position {
    x: u8,
    y: u8,
}

impl Position {
    pub const fn from_index(index: usize) -> Self {
        Self {
            x: (index % SIDE_LENGTH) as u8,
            y: (index / SIDE_LENGTH) as u8,
        }
    }

    pub const fn as_index(&self) -> usize {
        self.y as usize * SIDE_LENGTH + self.x as usize
    }
}

struct Snake {
    snake_tail_index: usize,                     // 蛇尾坐标索引
    snake_length: usize,                         // 蛇的长度
    snake_positions_queue: [Position; MAP_SIZE], // 蛇坐标队列，仅`snake_length`个连续元素有效
}

impl Default for Snake {
    fn default() -> Self {
        Self {
            snake_tail_index: 0,
            snake_length: 0,
            snake_positions_queue: [Position::default(); MAP_SIZE],
        }
    }
}

impl Snake {
    fn push(&mut self, position: Position) {
        let snake_head_index = (self.snake_tail_index + self.snake_length) % MAP_SIZE;
        self.snake_positions_queue[snake_head_index] = position;
        self.snake_length += 1;
    }

    fn pop(&mut self) -> Position {
        let position = self.snake_positions_queue[self.snake_tail_index];
        self.snake_length -= 1;
        self.snake_tail_index = (self.snake_tail_index + 1) % MAP_SIZE;
        position
    }
}

struct Empty {
    empty_length: usize,                        // 空格坐标长度
    empty_positions: [Position; MAP_SIZE],      // 空格坐标，仅前`empty_length`个有效
    empty_positions_indices: [usize; MAP_SIZE], // 空格坐标索引，仅当对应坐标是空格时才有效
}

impl Default for Empty {
    fn default() -> Self {
        Self {
            empty_length: 0,
            empty_positions: std::array::from_fn(Position::from_index),
            empty_positions_indices: std::array::from_fn(|i| i),
        }
    }
}

impl Empty {
    fn pop(&mut self, position: Position) {
        // 维护空格坐标长度
        self.empty_length -= 1;

        // 把待删除的空格坐标替换为最后一个空格坐标
        let remove_index = self.empty_positions_indices[position.as_index()];
        let last_empty_position = self.empty_positions[self.empty_length];
        self.empty_positions[remove_index] = last_empty_position;

        // 维护空格坐标索引
        self.empty_positions_indices[last_empty_position.as_index()] = remove_index;
    }

    fn push(&mut self, position: Position) {
        // 维护空格坐标索引
        self.empty_positions_indices[position.as_index()] = self.empty_length;

        // 在列表最后插入新的空格坐标
        self.empty_positions[self.empty_length] = position;

        // 维护空格坐标长度
        self.empty_length += 1;
    }
}

struct Content {
    current_dir: Position, // 蛇当前方向
    map: [u8; MAP_SIZE],   // 地图
    snake: Snake,
    empty: Empty,
}

impl Default for Content {
    fn default() -> Self {
        Self {
            current_dir: PAUSE,
            map: [0; MAP_SIZE],
            empty: Empty::default(),
            snake: Snake::default(),
        }
    }
}

impl Content {
    // 生成初始的蛇与食物
    fn init(&mut self) {
        let snake_position = Position {
            x: (SIDE_LENGTH / 2) as u8,
            y: (SIDE_LENGTH / 2) as u8,
        };

        self.map[snake_position.as_index()] = 1;
        self.snake.push(snake_position);
        self.empty.pop(snake_position);

        // TODO 生成食物
    }

    fn update(&mut self, direction: Position) -> bool {
        if direction != PAUSE {
            self.current_dir = direction;
        }

        if self.current_dir != PAUSE {
            // 蛇前进
            let snake_head_position = self.snake.snake_positions_queue[self.snake.snake_tail_index];
            let new_snake_head_position = Position {
                x: (snake_head_position.x + self.current_dir.x) % SIDE_LENGTH as u8,
                y: (snake_head_position.y + self.current_dir.y) % SIDE_LENGTH as u8,
            };

            // TODO 检查蛇是否越界

            match self.map[new_snake_head_position.as_index()] {
                EMPTY => {
                    // TODO 蛇前进
                }
                FOOD => {
                    // TODO 蛇前进，并生成新的食物
                }
                _ => {
                    // TODO 蛇死亡
                }
            }
        }
        true
    }
}

fn main() {
    let mut content = Content::default();
    let mut step: usize = 0;

    content.init();
}
