use super::*;

pub struct BoardState {
    pub state: chess::State,
    pub piece_moves: HashMap<Position, HashSet<Position>>,
    pub last_move: Option<(Position, Position)>,
}

impl BoardState {
    fn new() -> BoardState {
        let state = chess::State::new();
        let piece_moves = state.get_piece_moves();
        BoardState {
            state,
            piece_moves,
            last_move: None,
        }
    }

    pub fn try_move(&self, from: Position, to: Position) -> Option<BoardState> {
        if self.state.is_valid_move(from, to) {
            let mut state = self.state.clone();
            state.try_move(from, to);
            let piece_moves = state.get_piece_moves();
            return Some(BoardState {
                state,
                piece_moves,
                last_move: Some((from, to)),
            });
        }
        None
    }

    pub fn promote(&mut self, variant: Variant) -> bool {
        if self.state.promote(variant) {
            self.piece_moves = self.state.get_piece_moves();
            return true;
        }
        false
    }
}

pub struct BoardStates {
    states: Vec<BoardState>,
    current: usize,
    end: usize,
}

impl BoardStates {
    pub fn new() -> BoardStates {
        BoardStates {
            states: vec![BoardState::new()],
            current: 0,
            end: 0,
        }
    }

    pub fn add(&mut self, state: BoardState) {
        self.current += 1;
        self.end = self.current;
        if self.states.len() > self.current {
            self.states[self.current] = state;
        } else {
            self.states.push(state);
        }

        let mut count = 0;
        for previous in &self.states[..self.current] {
            if previous.state == self.states[self.current].state {
                count += 1;
            }
        }

        if count >= 2 {
            self.states[self.current].state.threefold_repetition();
        }
    }

    pub fn undo(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
    }

    pub fn redo(&mut self) {
        if self.current < self.end {
            self.current += 1;
        }
    }

    pub fn reset(&mut self) {
        self.current = 0;
    }

    pub fn at_start(&self) -> bool {
        self.current == 0
    }

    pub fn at_end(&self) -> bool {
        self.current == self.end
    }

    pub fn active(&self) -> &BoardState {
        &self.states[self.current]
    }

    pub fn promote(&mut self, variant: Variant) -> bool {
        self.states[self.current].promote(variant)
    }
}
