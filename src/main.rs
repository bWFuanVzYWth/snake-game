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

const STATE_GAME_OVER: u8 = 0;
const STATE_READY: u8 = 1;
const STATE_GO: u8 = 2;

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

const fn offset(base: usize, offset: usize) -> usize {
    (base + offset) % MAP_SIZE
}

struct Content {
    current_dir: Position, // 记住前进方向
    map: [u8; MAP_SIZE],   // 地图
    snake_tail_index: usize,
    snake_length: usize,
    positions_list: [Position; MAP_SIZE],
    indices_list: [usize; MAP_SIZE],
}

impl Default for Content {
    fn default() -> Self {
        Self {
            current_dir: NONE,
            map: [0; MAP_SIZE],
            snake_tail_index: 0,
            snake_length: 1,
            positions_list: std::array::from_fn(|i| Position::from_index(i)),
            indices_list: std::array::from_fn(|i| i),
            // TODO 检查初始化是否正确
        }
    }
}

impl Content {
    fn food_position(&self) -> Position {
        let mut rng = rand::rng();
        let empty_index_from = offset(self.snake_tail_index, self.snake_length + 1);
        let empty_length = MAP_SIZE - self.snake_length - 1;
        let index = offset(
            empty_index_from,
            rand::Rng::random_range(&mut rng, 0..empty_length),
        );
        self.positions_list[index]
    }

    fn generate_food(&mut self) {
        let food_position = self.food_position();
        let food_index = food_position.as_index();

        self.map[food_index] = FOOD;
        // FIXME 这里没有创建空位，要创建吗？
        // 想想应该不用，食物本来就不会撞到人。每次先让push蛇头，再生成食物，就不会错了
    }

    // 生成初始的食物
    fn init(&mut self) {
        self.generate_food();
    }

    fn pop_snake_tail(&mut self) -> Position {
        let tail_position = self.positions_list[self.snake_tail_index];
        self.map[self.snake_tail_index] = EMPTY;
        self.snake_tail_index = offset(self.snake_tail_index, 1);
        self.snake_length -= 1;

        tail_position
    }

    fn push_snake_head(&mut self, head_pos: Position) {
        // 计算蛇头位置，插入蛇，维护蛇长
        let head_index = offset(self.snake_tail_index, self.snake_length);
        self.positions_list[head_index] = head_pos;
        self.map[head_index] = SNAKE;
        self.snake_length += 1;

        // 移动empty，维护被移动的empty的index
        let empty_index_to = head_pos.as_index();
        let empty_index_from = offset(head_index, 1);
        self.indices_list[self.positions_list[empty_index_from].as_index()] = empty_index_to;
        self.positions_list[empty_index_to] = self.positions_list[empty_index_from];
    }

    fn update(&mut self, direction: Position) -> u8 {
        if direction != NONE {
            self.current_dir = direction;
        }

        if self.current_dir == NONE {
            return STATE_READY;
        }

        // 计算新的蛇头位置
        let head_index = offset(self.snake_tail_index, self.snake_length);
        let head_position = self.positions_list[head_index];
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
        let new_head_index = new_head_position.as_index();

        match self.map[new_head_index] {
            EMPTY => {
                // 空位：蛇头前进一格，蛇尾收缩一格
                self.push_snake_head(new_head_position);
                self.pop_snake_tail();

                STATE_GO
            }
            FOOD => {
                // 检查是否还有空间生成食物
                if self.snake_length >= 255 {
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
