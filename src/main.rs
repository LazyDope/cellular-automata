use std::time::Instant;

use nannou::prelude::*;

fn main() {
    nannou::app(model)
        .update(update)
        .event(event)
        .simple_window(view)
        .run();
}

const GRID_SIZE: usize = 20;

struct Model {
    active: [[State; GRID_SIZE]; GRID_SIZE],
    rules: Vec<Rule>,
    last: Instant,
    last_pos: Option<Point2>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum State {
    Full,
    Empty,
}

enum Rule {
    Axial {
        in_state: Vec<Vec<State>>,
        out_state: Vec<Vec<State>>,
    },
    Radial {
        current_state: State,
        surroundings: Vec<(usize, State, Comparison)>,
        final_state: State,
    },
}

#[derive(PartialEq, Eq)]
enum Comparison {
    Equal,
    NotEqual,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl Comparison {
    fn compare<T>(&self, lhs: T, rhs: T) -> bool
    where
        T: PartialOrd,
    {
        match lhs.partial_cmp(&rhs) {
            Some(ord) => match ord {
                std::cmp::Ordering::Less => {
                    self == &Self::LessThan
                        || self == &Self::LessThanOrEqual
                        || self == &Self::NotEqual
                }
                std::cmp::Ordering::Equal => {
                    self == &Self::Equal
                        || self == &Self::LessThanOrEqual
                        || self == &Self::GreaterThanOrEqual
                }
                std::cmp::Ordering::Greater => {
                    self == &Self::GreaterThan
                        || self == &Self::GreaterThanOrEqual
                        || self == &Self::NotEqual
                }
            },
            None => false,
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::Empty
    }
}

fn model(_app: &App) -> Model {
    let active = [[Default::default(); GRID_SIZE]; GRID_SIZE];
    Model {
        active,
        rules: vec![Rule::Axial {
            // if there is a full cell above an empty cell, swap them
            in_state: vec![vec![State::Full, State::Empty]],
            out_state: vec![vec![State::Empty, State::Full]],
        }],
        last: Instant::now(),
        last_pos: Default::default(),
    }
}

fn event(app: &App, model: &mut Model, event: Event) {
    match event {
        Event::WindowEvent {
            simple: Some(event),
            ..
        // clicking or tapping in a cell to swap it's 'fullness'
        } => match event {
            MousePressed(_) if model.last_pos.is_some() => {
                let last_pos = model.last_pos.clone().unwrap();
                update_grid(&app.main_window().rect(), model, &last_pos)
            }
            Touch(touch_event) => {
                update_grid(&app.main_window().rect(), model, &touch_event.position)
            }
            MouseMoved(pos) => model.last_pos = Some(pos),
            _ => (),
        },
        _ => (),
    }
}

// changes the state of the cell that was interacted with
fn update_grid(win: &Rect, model: &mut Model, win_pos: &Point2) {
    let w = win.x.len() / (GRID_SIZE as f32);
    let h = win.y.len() / (GRID_SIZE as f32);
    let x = ((win_pos.x - win.x.start) / w) as usize;
    let y = ((win.y.end - win_pos.y) / h) as usize;
    model.active[x][y] = match model.active[x][y] {
        State::Full => State::Empty,
        State::Empty => State::Full,
    }
}

fn update(_app: &App, model: &mut Model, _update: Update) {
    if model.last.elapsed().as_millis() < 50 {
        return;
    } else {
        model.last = Instant::now();
    }
    let mut inactive = model.active.clone();
    for (i, col) in model.active.iter().enumerate() {
        for (j, cell) in col.iter().enumerate() {
            for rule in model.rules.iter() {
                match rule {
                    Rule::Axial {
                        in_state,
                        out_state,
                    } => model.axial((i, j), &mut inactive, in_state, out_state),
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
                }
            }
        }
    }
    model.active = inactive;
}

impl Model {
    fn radial(
        &self,
        cell: &State,
        cell_cords: (usize, usize),
        inactive: &mut [[State; GRID_SIZE]; GRID_SIZE],
        current_state: &State,
        surroundings: &[(usize, State, Comparison)],
        final_state: &State,
    ) {
        if current_state == cell {
            let cells: Vec<_> = (-1i64..=1)
                .flat_map(|ri| {
                    (-1i64..=1).map(move |rj| (ri, rj)).map(|(ri, rj)| {
                        self.active
                            .get((cell_cords.0 as i64 + ri) as usize)
                            .map(|col| col.get((cell_cords.1 as i64 + rj) as usize))
                            .flatten()
                    })
                })
                .filter_map(|cell| cell)
                .collect();
            if surroundings.iter().all(|(count, req_state, comp)| {
                comp.compare(
                    *count,
                    cells.iter().filter(|val| val == &&req_state).count(),
                )
            }) {
                inactive[cell_cords.0][cell_cords.1] = final_state.clone();
            }
        }
    }

    fn axial(
        &self,
        cell_cords: (usize, usize),
        inactive: &mut [[State; GRID_SIZE]; GRID_SIZE],
        in_state: &Vec<Vec<State>>,
        out_state: &Vec<Vec<State>>,
    ) {
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
            .all(|(ri, rj, rule_cell)| {
                self.active.get(cell_cords.0 + ri).is_some_and(|col| {
                    col.get(cell_cords.1 + rj)
                        .is_some_and(|cell| cell == rule_cell)
                })
            })
        {
            // if cells match expected, perform the swaps to the new layout
            for (ri, rj, state) in out_state.iter().enumerate().flat_map(|(ri, out_col)| {
                out_col
                    .iter()
                    .enumerate()
                    .map(move |(rj, state)| (ri, rj, state))
            }) {
                inactive[cell_cords.0 + ri][cell_cords.1 + rj] = state.clone();
            }
        }
    }
}

fn view(app: &App, model: &Model, frame: Frame) {
    let draw = app.draw();
    let window = app.main_window();
    let win = window.rect();
    draw.background().color(BLUE);

    draw_grid(&draw, &win, &model);

    draw.to_frame(app, &frame).unwrap();
}

fn draw_grid(draw: &Draw, win: &Rect, model: &Model) {
    let w = win.x.len() / (GRID_SIZE as f32);
    let h = win.y.len() / (GRID_SIZE as f32);
    let x0 = win.x.start + w / 2.;
    let y0 = win.y.end - h / 2.;
    for (i, col) in model.active.iter().enumerate() {
        for (j, cell) in col.iter().enumerate() {
            draw.rect()
                .x_y(x0 + (i as f32) * w, y0 - (j as f32) * h)
                .w_h(w, h)
                .stroke_weight(0.5)
                .stroke(GRAY)
                .color(match cell {
                    State::Full => WHITE,
                    State::Empty => BLACK,
                });
        }
    }
}
