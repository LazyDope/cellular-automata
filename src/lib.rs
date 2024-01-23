use std::{
    ops::{Index, IndexMut},
    time::Instant,
};

use nannou::{
    color::encoding::Srgb,
    prelude::{rgb::Rgb, *},
    state::mouse::ButtonPosition,
};

pub struct Model<const GRID_SIZE: usize> {
    active: Grid<GRID_SIZE>,
    rules: Vec<Rule>,
    last: Instant,
    paused: bool,
    fill_state: State,
}

#[derive(Clone)]
pub struct Grid<const GRID_SIZE: usize> {
    grid: [[State; GRID_SIZE]; GRID_SIZE],
}

impl<const GRID_SIZE: usize> Grid<GRID_SIZE> {
    fn get_cell(&self, x: usize, y: usize) -> Option<&State> {
        self.get_col(x).map(|col| col.get(y)).flatten()
    }

    fn get_col(&self, x: usize) -> Option<&[State; GRID_SIZE]> {
        self.grid.get(x)
    }

    fn indexed_iter(&self) -> impl Iterator<Item = (usize, usize, &State)> {
        self.grid
            .iter()
            .enumerate()
            .flat_map(|(i, col)| col.iter().enumerate().map(move |(j, cell)| (i, j, cell)))
    }
}

impl<const GRID_SIZE: usize> From<[[State; GRID_SIZE]; GRID_SIZE]> for Grid<GRID_SIZE> {
    fn from(value: [[State; GRID_SIZE]; GRID_SIZE]) -> Self {
        Grid { grid: value }
    }
}

impl<const GRID_SIZE: usize> Index<(usize, usize)> for Grid<GRID_SIZE> {
    type Output = State;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.grid[index.0][index.1]
    }
}

impl<const GRID_SIZE: usize> IndexMut<(usize, usize)> for Grid<GRID_SIZE> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.grid[index.0][index.1]
    }
}

impl<const GRID_SIZE: usize> Default for Grid<GRID_SIZE> {
    fn default() -> Self {
        Self::from([[Default::default(); GRID_SIZE]; GRID_SIZE])
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Full,
    Empty,
}

impl State {
    fn color(&self) -> Rgb<Srgb, u8> {
        match self {
            State::Full => WHITE,
            State::Empty => BLACK,
        }
    }

    fn next(self) -> Self {
        use State::*;
        match self {
            Full => Empty,
            Empty => Full,
        }
    }

    fn prev(self) -> Self {
        use State::*;
        match self {
            Full => Empty,
            Empty => Full,
        }
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            State::Full => write!(f, "X"),
            State::Empty => write!(f, "O"),
        }
    }
}

pub enum Rule {
    Linear {
        in_state: Vec<Vec<Option<State>>>,
        out_state: Vec<Vec<Option<State>>>,
    },
    Radial {
        current_state: State,
        surroundings: Vec<(State, Comparison<usize>)>,
        final_state: State,
    },
}

pub enum Comparison<T> {
    Equal(T),
    NotEqual(T),
    GreaterThan(T),
    LessThan(T),
    GreaterThanOrEqual(T),
    LessThanOrEqual(T),
    BetweenExclusive(T, T),
    BetweenInclusive(T, T),
}

impl<T: PartialOrd> Comparison<T> {
    fn compare(&self, other: T) -> bool {
        match self {
            Comparison::Equal(value) => &other == value,
            Comparison::NotEqual(value) => &other != value,
            Comparison::GreaterThan(value) => &other > value,
            Comparison::LessThan(value) => &other < value,
            Comparison::GreaterThanOrEqual(value) => &other >= value,
            Comparison::LessThanOrEqual(value) => &other <= value,
            Comparison::BetweenExclusive(lower, upper) => lower < &other && &other < upper,
            Comparison::BetweenInclusive(lower, upper) => lower <= &other && &other <= upper,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Empty
    }
}

impl<const GRID_SIZE: usize> Model<GRID_SIZE> {
    pub fn model(
        _app: &App,
        starting_state: Option<Grid<GRID_SIZE>>,
        rules: Vec<Rule>,
        paused: bool,
    ) -> Model<GRID_SIZE> {
        let active = starting_state.unwrap_or_default();
        Model {
            active,
            rules,
            last: Instant::now(),
            paused,
            fill_state: Default::default(),
        }
    }
}

pub fn event<const GRID_SIZE: usize>(app: &App, model: &mut Model<GRID_SIZE>, event: Event) {
    if let ButtonPosition::Down(_) = app.mouse.buttons.left() {
        update_grid(
            &app.main_window().rect(),
            model,
            &Point2::new(app.mouse.x, app.mouse.y),
        );
    }
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        // clicking or tapping in a cell to swap it's 'fullness'
        } => match event {
            Touch(touch_event) => {
                update_grid(&app.main_window().rect(), model, &touch_event.position)
            }
            KeyPressed(key) => {
                match key {
                    Key::P => model.paused = !model.paused,
                    Key::C => model.active = Default::default(),
                    _ => (),
                }
            }
            MouseWheel(delta, phase) => {
                match phase {
                    TouchPhase::Moved => match delta {
                        MouseScrollDelta::LineDelta(_, y) => if y > 0. {
                            model.fill_state = model.fill_state.next();
                        } else if y < 0. {
                            model.fill_state = model.fill_state.prev();
                        },
                        MouseScrollDelta::PixelDelta(pos) => if pos.y > 0. {
                            model.fill_state = model.fill_state.next();
                        } else if pos.y < 0. {
                            model.fill_state = model.fill_state.prev();
                        },
                    },
                    _ => (),
                }
            }
            _ => (),
        },
        _ => (),
    }
}

// changes the state of the cell that was interacted with
fn update_grid<const GRID_SIZE: usize>(win: &Rect, model: &mut Model<GRID_SIZE>, pos: &Point2) {
    let w = win.x.len() / (GRID_SIZE as f32);
    let h = win.y.len() / (GRID_SIZE as f32);
    let x = ((pos.x - win.x.start) / w) as usize;
    let y = ((win.y.end - pos.y) / h) as usize;
    model.active[(x.min(GRID_SIZE - 1).max(0), y.min(GRID_SIZE - 1).max(0))] =
        model.fill_state.clone();
}

pub fn update<const GRID_SIZE: usize>(_app: &App, model: &mut Model<GRID_SIZE>, _update: Update) {
    if model.paused || model.last.elapsed().as_millis() < 50 {
        return;
    } else {
        model.last = Instant::now();
    }
    let mut inactive = model.active.clone();
    for rule in model.rules.iter() {
        for (i, j, cell) in model.active.indexed_iter() {
            if match rule {
                Rule::Linear {
                    in_state,
                    out_state,
                } => model.linear((i, j), &mut inactive, in_state, out_state),
                Rule::Radial {
                    current_state,
                    surroundings,
                    final_state,
                } => model.radial(
                    &cell,
                    (i, j),
                    &mut inactive,
                    current_state,
                    surroundings,
                    final_state,
                ),
            } {
                continue;
            }
        }
    }
    model.active = inactive;
}

impl<const GRID_SIZE: usize> Model<GRID_SIZE> {
    fn radial(
        &self,
        cell: &State,
        cell_cords: (usize, usize),
        inactive: &mut Grid<GRID_SIZE>,
        current_state: &State,
        surroundings: &[(State, Comparison<usize>)],
        final_state: &State,
    ) -> bool {
        if current_state == cell {
            let cells: Vec<_> = (-1i64..=1)
                .flat_map(|ri| {
                    (-1i64..=1)
                        .map(move |rj| (ri, rj))
                        .filter(|r| r != &(0, 0))
                        .map(|(ri, rj)| {
                            self.active.get_cell(
                                (cell_cords.0 as i64 + ri) as usize,
                                (cell_cords.1 as i64 + rj) as usize,
                            )
                        })
                })
                .filter_map(|cell| cell)
                .collect();
            if surroundings.iter().all(|(req_state, comp)| {
                comp.compare(cells.iter().filter(|val| val == &&req_state).count())
            }) {
                inactive[(cell_cords.0, cell_cords.1)] = final_state.clone();
                return true;
            }
        }
        return false;
    }

    fn linear(
        &self,
        cell_cords: (usize, usize),
        inactive: &mut Grid<GRID_SIZE>,
        in_state: &[Vec<Option<State>>],
        out_state: &[Vec<Option<State>>],
    ) -> bool {
        // check all the states relative to the given cell
        if in_state
            .iter()
            .enumerate()
            .flat_map(|(ri, rule_col)| {
                rule_col
                    .iter()
                    .enumerate()
                    .map(move |(rj, rule_cell)| (ri, rj, rule_cell))
            })
            .filter_map(|(ri, rj, rule_cell)| rule_cell.map(|cell| (ri, rj, cell)))
            .all(|(ri, rj, rule_cell)| {
                self.active
                    .get_cell(cell_cords.0 + ri, cell_cords.1 + rj)
                    .is_some_and(|cell| cell == &rule_cell)
                    && inactive
                        .get_cell(cell_cords.0 + ri, cell_cords.1 + rj)
                        .is_some_and(|cell| cell == &rule_cell)
            })
        {
            // if cells match expected, perform the swaps to the new layout
            for (ri, rj, state) in out_state.iter().enumerate().flat_map(|(ri, out_col)| {
                out_col
                    .iter()
                    .enumerate()
                    .filter_map(move |(rj, state)| state.map(|state| (ri, rj, state)))
            }) {
                inactive[(cell_cords.0 + ri, cell_cords.1 + rj)] = state.clone();
            }
            return true;
        }
        return false;
    }
}

pub fn view<const GRID_SIZE: usize>(app: &App, model: &Model<GRID_SIZE>, frame: Frame) {
    let draw = app.draw();
    let window = app.main_window();
    let win = window.rect();
    draw.background().color(BLUE);

    draw_grid(&draw, &win, &model);

    draw.to_frame(app, &frame).unwrap();
}

fn draw_grid<const GRID_SIZE: usize>(draw: &Draw, win: &Rect, model: &Model<GRID_SIZE>) {
    let w = win.x.len() / (GRID_SIZE as f32);
    let h = win.y.len() / (GRID_SIZE as f32);
    let x0 = win.x.start + w / 2.;
    let y0 = win.y.end - h / 2.;
    for (i, j, cell) in model.active.indexed_iter() {
        draw.rect()
            .x_y(x0 + (i as f32) * w, y0 - (j as f32) * h)
            .w_h(w, h)
            .stroke_weight(0.5)
            .stroke(GRAY)
            .color(cell.color());
    }
    draw.rect()
        .x_y(x0 + w / 2., y0 - h / 2.)
        .w_h(w.min(h), w.min(h))
        .stroke_weight(0.5)
        .stroke(GRAY)
        .color(model.fill_state.color());
}
