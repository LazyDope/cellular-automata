use cellular_automaton::*;

fn main() {
    nannou::app(|app| Model::<50>::model(app, None, game_of_life(), false))
        .update(update)
        .event(event)
        .simple_window(view)
        .run();
}

fn falling_sand() -> Vec<Rule> {
    vec![
        Rule::Linear {
            // if there is a full cell above an empty cell, swap them
            in_state: vec![vec![Some(State::Full), Some(State::Empty)]],
            out_state: vec![vec![Some(State::Empty), Some(State::Full)]],
        },
        Rule::Linear {
            in_state: vec![
                vec![None, Some(State::Empty)],
                vec![Some(State::Full), Some(State::Full)],
            ],
            out_state: vec![
                vec![None, Some(State::Full)],
                vec![Some(State::Empty), Some(State::Full)],
            ],
        },
        Rule::Linear {
            in_state: vec![
                vec![Some(State::Full), Some(State::Full)],
                vec![None, Some(State::Empty)],
            ],
            out_state: vec![
                vec![Some(State::Empty), Some(State::Full)],
                vec![None, Some(State::Full)],
            ],
        },
    ]
}

fn game_of_life() -> Vec<Rule> {
    vec![
        Rule::Radial {
            current_state: State::Empty,
            surroundings: vec![(State::Full, Comparison::Equal(3))],
            final_state: State::Full,
        },
        Rule::Radial {
            current_state: State::Full,
            surroundings: vec![(State::Full, Comparison::LessThan(2))],
            final_state: State::Empty,
        },
        Rule::Radial {
            current_state: State::Full,
            surroundings: vec![(State::Full, Comparison::GreaterThan(3))],
            final_state: State::Empty,
        },
    ]
}
