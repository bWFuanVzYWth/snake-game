const SIDE_LENGTH: usize = 16;
const MAP_SIZE: usize = SIDE_LENGTH * SIDE_LENGTH;

const NONE: Position = Position { x: 0, y: 0 };
const RIGHT: Position = Position { x: 1, y: 0 };
const UP: Position = Position { x: 0, y: -1 };
const LEFT: Position = Position { x: -1, y: 0 };
const DOWN: Position = Position { x: 0, y: 1 };

const EMPTY: u8 = 0;
const SNAKE: u8 = 1;
const FOOD: u8 = 2;

#[derive(Copy, Clone, Default, PartialEq)]
struct Position {
    x: i8,
    y: i8,
}

impl Position {
    pub const fn from_index(index: usize) -> Self {
        Self {
            x: (index % SIDE_LENGTH) as i8,
            y: (index / SIDE_LENGTH) as i8,
        }
    }

    pub const fn as_index(&self) -> usize {
        self.y as usize * SIDE_LENGTH + self.x as usize
    }
}

struct Snake {
    tail_index: usize,                     // 蛇尾坐标索引
    length: usize,                         // 蛇的长度
    positions_queue: [Position; MAP_SIZE], // 蛇坐标队列，仅`snake_length`个连续元素有效
}

impl Default for Snake {
    fn default() -> Self {
        Self {
            tail_index: 0,
            length: 0,
            positions_queue: [Position::default(); MAP_SIZE],
        }
    }
}

impl Snake {
    fn push(&mut self, position: Position) {
        let head_index = (self.tail_index + self.length) % MAP_SIZE;
        self.positions_queue[head_index] = position;
        self.length += 1;
    }

    fn pop(&mut self) -> Position {
        let position = self.positions_queue[self.tail_index];
        self.length -= 1;
        self.tail_index = (self.tail_index + 1) % MAP_SIZE;
        position
    }
}

struct Empty {
    length: usize,                        // 空格坐标长度
    positions_set: [Position; MAP_SIZE],  // 空格坐标，仅前`empty_length`个有效
    positions_indices: [usize; MAP_SIZE], // 空格坐标索引，仅当对应坐标是空格时才有效
}

impl Default for Empty {
    fn default() -> Self {
        Self {
            length: MAP_SIZE,
            positions_set: std::array::from_fn(Position::from_index),
            positions_indices: std::array::from_fn(|i| i),
        }
    }
}

impl Empty {
    fn pop(&mut self, position: Position) {
        // 维护空格坐标长度
        self.length -= 1;

        // 把待删除的空格坐标替换为最后一个空格坐标
        let remove_index = self.positions_indices[position.as_index()];
        let last_empty_position = self.positions_set[self.length];
        self.positions_set[remove_index] = last_empty_position;

        // 维护空格坐标索引
        self.positions_indices[last_empty_position.as_index()] = remove_index;
    }

    fn push(&mut self, position: Position) {
        assert!(self.length < MAP_SIZE, "Snake length exceeds map size");

        // 维护空格坐标索引
        self.positions_indices[position.as_index()] = self.length;

        // 在列表最后插入新的空格坐标
        self.positions_set[self.length] = position;

        // 维护空格坐标长度
        self.length += 1;
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
            current_dir: NONE,
            map: [0; MAP_SIZE],
            empty: Empty::default(),
            snake: Snake::default(),
        }
    }
}

impl Content {
    fn food_position(&self) -> Position {
        let mut rng = rand::rng();
        let index = rand::Rng::random_range(&mut rng, 0..self.empty.length);
        self.empty.positions_set[index]
    }

    fn generate_food(&mut self) {
        let food_position = self.food_position();
        self.map[food_position.as_index()] = FOOD;
        self.empty.pop(food_position);
    }

    // 生成初始的蛇与食物
    fn init(&mut self) {
        let snake_position = Position {
            x: (SIDE_LENGTH / 2) as i8,
            y: (SIDE_LENGTH / 2) as i8,
        };

        self.map[snake_position.as_index()] = 1;
        self.snake.push(snake_position);
        self.empty.pop(snake_position);

        self.generate_food();
    }

    fn update(&mut self, direction: Position) -> bool {
        if direction != NONE {
            self.current_dir = direction;
        }

        if self.current_dir != NONE {
            // 计算新的蛇头位置
            let head_position = self.snake.positions_queue
                [(self.snake.tail_index + self.snake.length - 1) % MAP_SIZE];
            let new_head_position = Position {
                x: (head_position.x + self.current_dir.x),
                y: (head_position.y + self.current_dir.y),
            };

            // 检查蛇是否越界
            if new_head_position.x >= SIDE_LENGTH as i8
                || new_head_position.y >= SIDE_LENGTH as i8
                || new_head_position.x < 0
                || new_head_position.y < 0
            {
                return false;
            }

            match self.map[new_head_position.as_index()] {
                EMPTY => {
                    // 空位：蛇头前进一格，蛇尾收缩一格
                    self.map[new_head_position.as_index()] = SNAKE;
                    self.snake.push(new_head_position);
                    self.empty.pop(new_head_position);

                    let tail_position = self.snake.pop();
                    self.map[tail_position.as_index()] = EMPTY;
                    self.empty.push(tail_position);

                    return true;
                }
                FOOD => {
                    // 食物：蛇头前进一格，生成新的食物
                    self.map[new_head_position.as_index()] = SNAKE;
                    self.snake.push(new_head_position);
                    self.empty.pop(new_head_position);
                    self.generate_food();
                    return true;
                }
                SNAKE => {
                    // 蛇身：游戏结束
                    return false;
                }
                _ => {
                    panic!("Invalid cell value");
                }
            }
        }

        true
    }

    fn print_board(&self) {
        const BORDER: &str = "-";
        const BORDER_LENGTH: usize = SIDE_LENGTH + 2;

        let border_line = BORDER.repeat(BORDER_LENGTH);
        let mut output = String::with_capacity(MAP_SIZE + 4 * SIDE_LENGTH);

        output.push_str("\x1B[2J\x1B[1;1H");
        output.push_str(&format!("{border_line}\n"));

        for i in 0..MAP_SIZE {
            let pos = i % SIDE_LENGTH;
            if pos == 0 {
                output.push('|');
            }

            let ch = match self.map[i] {
                EMPTY => ' ',
                SNAKE => '#',
                FOOD => 'F',
                _ => panic!("Invalid cell value"),
            };
            output.push(ch);

            if (pos + 1) == SIDE_LENGTH {
                output.push_str("|\n");
            }
        }

        output.push_str(&format!("{border_line}\n"));

        print!("{output}");
    }
}

const UPDATE_INTERVAL_MILLIS: u64 = 250;

fn main() -> std::io::Result<()> {
    crossterm::terminal::enable_raw_mode()?;

    let mut content = Content::default();
    let mut moves_count: usize = 0;

    content.init();
    content.print_board();

    loop {
        // 非阻塞式输入处理
        let direction = {
            let mut dir = NONE;
            while crossterm::event::poll(std::time::Duration::from_millis(0))? {
                if let Ok(crossterm::event::Event::Key(key_event)) = crossterm::event::read() {
                    use crossterm::event::KeyCode;
                    dir = match key_event.code {
                        KeyCode::Up => UP,
                        KeyCode::Down => DOWN,
                        KeyCode::Left => LEFT,
                        KeyCode::Right => RIGHT,
                        _ => NONE,
                    };
                }
            }
            dir
        };

        if !content.update(direction) {
            break;
        }

        moves_count += 1;
        content.print_board();

        // 固定时间休眠
        std::thread::sleep(std::time::Duration::from_millis(UPDATE_INTERVAL_MILLIS));
    }

    println!("Game over after {moves_count} moves");

    crossterm::terminal::disable_raw_mode()?;
    Ok(())
}
