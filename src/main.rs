const SIDE_LENGTH: usize = 16;
const MAP_SIZE: usize = SIDE_LENGTH * SIDE_LENGTH;

const NONE: Position = Position { x: 0, y: 0 };
const RIGHT: Position = Position { x: 1, y: 0 };
const UP: Position = Position { x: 0, y: -1 };
const LEFT: Position = Position { x: -1, y: 0 };
const DOWN: Position = Position { x: 0, y: 1 };

const EMPTY: u8 = 0;
const FOOD: u8 = 1;
const SNAKE: u8 = 2;

const STATE_GAME_OVER: u8 = 0;
const STATE_READY: u8 = 1;
const STATE_GO: u8 = 2;

#[derive(Copy, Clone, Default, PartialEq, Debug)]
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

    pub const fn as_hash(&self) -> usize {
        self.y as usize * SIDE_LENGTH + self.x as usize
    }
}

const fn offset(base: usize, offset: usize) -> usize {
    (base + offset) % MAP_SIZE
}

#[derive(Debug)]
struct Content {
    current_dir: Position, // 记住前进方向
    map: [u8; MAP_SIZE],   // 地图
    tail_index: usize,
    length: usize,
    positions: [Position; MAP_SIZE],
    indices: [usize; MAP_SIZE],
}

impl Default for Content {
    fn default() -> Self {
        let mut tmp = Self {
            current_dir: NONE,
            map: [EMPTY; MAP_SIZE],
            tail_index: 0,
            length: 0,
            positions: std::array::from_fn(Position::from_index),
            indices: std::array::from_fn(|i| i),
        };

        // 生成初始蛇
        const SNAKE_POSITION: Position = Position { x: 8, y: 8 };
        const SNAKE_POSITION_AS_INDEX: usize = SNAKE_POSITION.as_hash();
        tmp.tail_index = SNAKE_POSITION_AS_INDEX;
        tmp.push_snake_head(SNAKE_POSITION);

        // 生成初始的食物
        tmp.generate_food();

        tmp
    }
}

impl Content {
    fn food_position(&self) -> Position {
        let mut rng = rand::rng();
        let empty_index_from = offset(self.tail_index, self.length);
        let empty_length = MAP_SIZE - self.length;
        let index = offset(
            empty_index_from,
            rand::Rng::random_range(&mut rng, 0..empty_length),
        );
        self.positions[index]
    }

    fn generate_food(&mut self) {
        let food_position = self.food_position();
        let food_index = food_position.as_hash();

        self.map[food_index] = FOOD;
    }

    fn pop_snake_tail(&mut self) -> Position {
        // 只是弹出蛇尾，似乎不用移动positions中的元素？
        let tail_position = self.positions[self.tail_index];

        // 修改tail_index
        self.tail_index = offset(self.tail_index, 1);

        // 修改map
        let tail_hash = tail_position.as_hash();
        self.map[tail_hash] = EMPTY;

        // 蛇长--
        self.length -= 1;

        tail_position
    }

    fn push_snake_head(&mut self, head_position: Position) {
        // 找到新的蛇头对应的元素
        let new_head_hash = head_position.as_hash();
        let new_head_index = self.indices[new_head_hash];

        // 找到因为会被覆写，所以需要迁移的元素
        let relocate_from_index = offset(self.tail_index, self.length);
        let relocate_from_hash = self.positions[relocate_from_index].as_hash();

        // 交换元素
        self.positions[new_head_index] = self.positions[relocate_from_index];
        self.positions[relocate_from_index] = head_position;

        // 维护因交换元素变化的索引
        self.indices[new_head_hash] = relocate_from_index;
        self.indices[relocate_from_hash] = new_head_index;

        // 修改map
        self.map[new_head_hash] = SNAKE;

        // 蛇长++
        self.length += 1;
    }
    // 理论上应该先删蛇尾，再判断蛇头碰撞，再插入蛇头，再生成食物
    fn update(&mut self, direction: Position) -> u8 {
        if direction != NONE {
            self.current_dir = direction;
        }

        if self.current_dir == NONE {
            return STATE_READY;
        }

        // 计算新的蛇头位置
        let head_index = offset(self.tail_index, self.length - 1);
        let head_position = self.positions[head_index];
        let new_head_position = Position {
            x: (head_position.x + self.current_dir.x),
            y: (head_position.y + self.current_dir.y),
        };
        if new_head_position.x >= SIDE_LENGTH as i8
            || new_head_position.x < 0
            || new_head_position.y >= SIDE_LENGTH as i8
            || new_head_position.y < 0
        {
            return STATE_GAME_OVER;
        }

        // 预计算索引
        let new_head_index = new_head_position.as_hash();

        match self.map[new_head_index] {
            EMPTY => {
                // 空位：蛇头前进一格，蛇尾收缩一格
                self.pop_snake_tail();
                self.push_snake_head(new_head_position);

                STATE_GO
            }
            FOOD => {
                // 检查是否还有空间生成食物
                if self.length >= MAP_SIZE - 1 {
                    return STATE_GAME_OVER;
                }

                // 食物：蛇头前进一格，生成新的食物
                self.push_snake_head(new_head_position);
                self.generate_food();

                STATE_GO
            }
            SNAKE => {
                // 蛇身：游戏结束
                STATE_GAME_OVER
            }
            _ => {
                panic!("Invalid cell value");
            }
        }
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

        // dbg!(&content);

        match content.update(direction) {
            STATE_GAME_OVER => {
                break;
            }
            STATE_READY => {}
            STATE_GO => {
                moves_count += 1;
            }
            _ => panic!("Invalid Game State!"),
        }

        content.print_board();

        // 固定时间休眠
        std::thread::sleep(std::time::Duration::from_millis(UPDATE_INTERVAL_MILLIS));
    }

    println!("Game over after {moves_count} moves");

    crossterm::terminal::disable_raw_mode()?;

    Ok(())
}
