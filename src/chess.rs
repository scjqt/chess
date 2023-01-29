use std::collections::{HashMap, HashSet};
use Colour::*;
use MoveType::*;
use Variant::*;

#[derive(Clone)]
pub struct State {
    pieces: HashMap<Position, Piece>,
    moves: HashMap<(Position, Position), MoveInfo>,
    turn: Colour,
    info: StateInfo,
    ended: Option<EndState>,
}

impl State {
    pub fn new() -> State {
        State::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1")
    }

    pub fn from_fen(fen: &str) -> State {
        let mut parts = fen.split(' ');

        let mut pieces = HashMap::new();
        let mut x = 0;
        let mut y = 7;
        for char in parts.next().unwrap().chars() {
            match char {
                '/' => {
                    x = 0;
                    y -= 1;
                }
                _ if char::is_numeric(char) => x += char.to_string().parse::<i8>().unwrap(),
                c => {
                    pieces.insert(
                        Position::from_xy(x, y).unwrap(),
                        Piece {
                            colour: if c.is_uppercase() { White } else { Black },
                            variant: match c.to_lowercase().next().unwrap() {
                                'p' => Pawn,
                                'n' => Knight,
                                'b' => Bishop,
                                'r' => Rook,
                                'q' => Queen,
                                'k' => King,
                                _ => panic!(),
                            },
                        },
                    );
                    x += 1;
                }
            }
        }

        let turn = if parts.next().unwrap() == "w" {
            White
        } else {
            Black
        };

        let castling = parts.next().unwrap();
        let en_passant = match parts.next().unwrap() {
            "-" => None,
            x => Position::from_xy(
                x.chars().nth(0).unwrap() as i8 - 97,
                x[1..=1].parse::<i8>().unwrap() - 1,
            ),
        };

        let info = StateInfo {
            white_short: castling.contains('K'),
            white_long: castling.contains('Q'),
            black_short: castling.contains('k'),
            black_long: castling.contains('q'),
            en_passant,
            promoting: None,
        };

        let mut state = State {
            pieces,
            moves: HashMap::new(),
            turn,
            info,
            ended: None,
        };
        state.gen_legal_moves();
        state
    }

    fn gen_legal_moves(&mut self) {
        self.moves = self.gen_capture_moves(self.turn).unwrap();
        self.gen_other_moves();
        self.cull_moves();

        if self.moves.len() == 0 {
            self.ended = if self.in_check() {
                Some(EndState::Checkmate(self.turn.flipped()))
            } else {
                Some(EndState::Stalemate)
            };
        } else {
            if !self.check_material() {
                self.ended = Some(EndState::InsufficientMaterial);
                self.moves = HashMap::new();
            }
        }
    }

    fn gen_capture_moves(&self, colour: Colour) -> Option<HashMap<(Position, Position), MoveInfo>> {
        const OFFSETS: [(i8, i8); 16] = [
            (1, 1),
            (-1, 1),
            (-1, -1),
            (1, -1),
            (1, 0),
            (0, 1),
            (-1, 0),
            (0, -1),
            (2, 1),
            (1, 2),
            (-1, 2),
            (-2, 1),
            (-2, -1),
            (-1, -2),
            (1, -2),
            (2, -1),
        ];
        const CORNERS: [Position; 4] = [
            Position { value: 0 },
            Position { value: 7 },
            Position { value: 56 },
            Position { value: 63 },
        ];

        let mut moves = HashSet::new();

        for (&from, &moving) in self.pieces.iter() {
            if moving.colour == colour {
                match moving.variant {
                    Pawn => {
                        let y = if colour == White { 1 } else { -1 };
                        self.pawn_capture(&mut moves, from, colour, -1, y);
                        self.pawn_capture(&mut moves, from, colour, 1, y);
                    }
                    Knight => {
                        self.non_sliding_moves(&mut moves, from, colour, &OFFSETS[8..]);
                    }
                    Bishop => {
                        self.sliding_moves(&mut moves, from, colour, &OFFSETS[..4]);
                    }
                    Rook => {
                        self.sliding_moves(&mut moves, from, colour, &OFFSETS[4..8]);
                    }
                    Queen => {
                        self.sliding_moves(&mut moves, from, colour, &OFFSETS[..8]);
                    }
                    King => {
                        self.non_sliding_moves(&mut moves, from, colour, &OFFSETS[..8]);
                    }
                }
            }
        }

        let mut full_moves = HashMap::new();

        for (from, to) in moves.iter() {
            let mut info = MoveInfo::default();

            if let Some(piece) = self.pieces.get(to) {
                if piece.variant == King {
                    return None;
                }
            }

            match self.pieces.get(from).unwrap().variant {
                King => {
                    if colour == White {
                        info.state_info.white_short = false;
                        info.state_info.white_long = false;
                    } else {
                        info.state_info.black_short = false;
                        info.state_info.black_long = false;
                    };
                }
                Pawn => {
                    if self.pieces.get(to).is_some() {
                        if (colour == White && to.get_y() == 7)
                            || (colour == Black && to.get_y() == 0)
                        {
                            info.state_info.promoting = Some(*to);
                        }
                    } else {
                        info.move_type =
                            EnPassant(Position::from_xy(to.get_x(), from.get_y()).unwrap());
                    }
                }
                _ => (),
            }

            if *from == CORNERS[0] || *to == CORNERS[0] {
                info.state_info.white_long = false;
            }
            if *from == CORNERS[1] || *to == CORNERS[1] {
                info.state_info.white_short = false;
            }
            if *from == CORNERS[2] || *to == CORNERS[2] {
                info.state_info.black_long = false;
            }
            if *from == CORNERS[3] || *to == CORNERS[3] {
                info.state_info.black_short = false;
            }

            full_moves.insert((*from, *to), info);
        }

        Some(full_moves)
    }

    fn gen_other_moves(&mut self) {
        for (&from, &moving) in self.pieces.iter() {
            if moving.colour == self.turn {
                match moving.variant {
                    Pawn => {
                        let y = if self.turn == White { 1 } else { -1 };
                        if let Some(to) = from.offset_by(0, y) {
                            if self.pieces.get(&to).is_none() {
                                self.moves.insert(
                                    (from, to),
                                    if (self.turn == White && to.get_y() == 7)
                                        || (self.turn == Black && to.get_y() == 0)
                                    {
                                        MoveInfo {
                                            state_info: StateInfo {
                                                promoting: Some(to),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        }
                                    } else {
                                        MoveInfo::default()
                                    },
                                );
                                if (self.turn == White && from.get_y() == 1)
                                    || (self.turn == Black && from.get_y() == 6)
                                {
                                    if let Some(double) = from.offset_by(0, y * 2) {
                                        if self.pieces.get(&double).is_none() {
                                            self.moves.insert(
                                                (from, double),
                                                MoveInfo {
                                                    state_info: StateInfo {
                                                        en_passant: Some(to),
                                                        ..Default::default()
                                                    },
                                                    ..Default::default()
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                    King => {
                        let short = (self.turn == White && self.info.white_short)
                            || (self.turn == Black && self.info.black_short);
                        let long = (self.turn == White && self.info.white_long)
                            || (self.turn == Black && self.info.black_long);
                        if short || long {
                            if !self.in_check() {
                                let castle_info = if self.turn == White {
                                    StateInfo {
                                        white_short: false,
                                        white_long: false,
                                        ..Default::default()
                                    }
                                } else {
                                    StateInfo {
                                        black_short: false,
                                        black_long: false,
                                        ..Default::default()
                                    }
                                };
                                if short {
                                    if let Some(between) = from.offset_by(1, 0) {
                                        if self.pieces.get(&between).is_none() {
                                            let mut state = self.clone();
                                            state.make_move(from, between, MoveInfo::default());
                                            if !state.in_check() {
                                                if let Some(to) = from.offset_by(2, 0) {
                                                    if self.pieces.get(&to).is_none() {
                                                        self.moves.insert(
                                                            (from, to),
                                                            MoveInfo {
                                                                move_type: Castle(
                                                                    from.offset_by(3, 0).unwrap(),
                                                                    from.offset_by(1, 0).unwrap(),
                                                                ),
                                                                state_info: castle_info,
                                                            },
                                                        );
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                if long {
                                    if let Some(between) = from.offset_by(-1, 0) {
                                        if self.pieces.get(&between).is_none() {
                                            let mut state = self.clone();
                                            state.make_move(from, between, MoveInfo::default());
                                            if !state.in_check() {
                                                if let Some(to) = from.offset_by(-2, 0) {
                                                    if self.pieces.get(&to).is_none() {
                                                        if let Some(between2) =
                                                            from.offset_by(-3, 0)
                                                        {
                                                            if self.pieces.get(&between2).is_none()
                                                            {
                                                                self.moves.insert(
                                                                    (from, to),
                                                                    MoveInfo {
                                                                        move_type: Castle(
                                                                            from.offset_by(-4, 0)
                                                                                .unwrap(),
                                                                            from.offset_by(-1, 0)
                                                                                .unwrap(),
                                                                        ),
                                                                        state_info: castle_info,
                                                                    },
                                                                );
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }

    fn cull_moves(&mut self) {
        let temp = self.clone();
        self.moves.retain(|&(from, to), &mut info| {
            let mut state = temp.clone();
            state.make_move(from, to, info);
            !state.in_check()
        });
    }

    fn sliding_moves(
        &self,
        moves: &mut HashSet<(Position, Position)>,
        from: Position,
        colour: Colour,
        offsets: &[(i8, i8)],
    ) {
        for offset in offsets {
            let mut i = 1;
            while let Some(to) = from.offset_by(offset.0 * i, offset.1 * i) {
                if let Some(piece) = self.pieces.get(&to) {
                    if piece.colour != colour {
                        moves.insert((from, to));
                    }
                    break;
                } else {
                    moves.insert((from, to));
                    i += 1;
                }
            }
        }
    }

    fn non_sliding_moves(
        &self,
        moves: &mut HashSet<(Position, Position)>,
        from: Position,
        colour: Colour,
        offsets: &[(i8, i8)],
    ) {
        for offset in offsets {
            if let Some(to) = from.offset_by(offset.0, offset.1) {
                if let Some(piece) = self.pieces.get(&to) {
                    if piece.colour != colour {
                        moves.insert((from, to));
                    }
                } else {
                    moves.insert((from, to));
                }
            }
        }
    }

    fn pawn_capture(
        &self,
        moves: &mut HashSet<(Position, Position)>,
        from: Position,
        colour: Colour,
        x: i8,
        y: i8,
    ) {
        if let Some(to) = from.offset_by(x, y) {
            if let Some(piece) = self.pieces.get(&to) {
                if piece.colour != colour {
                    moves.insert((from, to));
                }
            } else if let Some(pos) = self.info.en_passant {
                if pos == to {
                    moves.insert((from, to));
                }
            }
        }
    }

    pub fn try_move(&mut self, from: Position, to: Position) -> bool {
        if self.info.promoting.is_none() {
            if let Some(&info) = self.moves.get(&(from, to)) {
                self.make_move(from, to, info);
                if self.info.promoting.is_none() {
                    self.turn.flip();
                    self.gen_legal_moves();
                }
                return true;
            }
        }
        false
    }

    pub fn is_valid_move(&self, from: Position, to: Position) -> bool {
        if self.info.promoting.is_none() {
            if self.moves.get(&(from, to)).is_some() {
                return true;
            }
        }
        false
    }

    fn make_move(&mut self, from: Position, to: Position, info: MoveInfo) {
        self.move_piece(from, to);
        self.info.apply(info.state_info);
        match info.move_type {
            Castle(f, t) => self.move_piece(f, t),
            EnPassant(p) => {
                self.pieces.remove(&p);
            }
            Normal => (),
        }
    }

    fn move_piece(&mut self, from: Position, to: Position) {
        self.pieces.insert(to, self.pieces[&from]);
        self.pieces.remove(&from);
    }

    pub fn promote(&mut self, variant: Variant) -> bool {
        if let Some(pos) = self.info.promoting {
            match variant {
                Knight | Bishop | Rook | Queen => {
                    self.pieces.insert(
                        pos,
                        Piece {
                            colour: self.turn,
                            variant,
                        },
                    );
                    self.turn.flip();
                    self.gen_legal_moves();
                    self.info.promoting = None;
                    return true;
                }
                _ => (),
            }
        }
        false
    }

    fn check_material(&self) -> bool {
        let mut minors_white = 0;
        let mut minors_black = 0;
        for piece in self.pieces.values() {
            match piece.variant {
                Pawn => return true,
                Rook => return true,
                Queen => return true,
                Knight | Bishop => {
                    if piece.colour == White {
                        minors_white += 1;
                    } else {
                        minors_black += 1;
                    }
                }
                _ => (),
            }
        }
        return minors_white > 1 || minors_black > 1;
    }

    pub fn get_pieces(&self) -> &HashMap<Position, Piece> {
        &self.pieces
    }

    pub fn get_piece_moves(&self) -> HashMap<Position, HashSet<Position>> {
        let mut piece_moves = HashMap::new();
        for &(from, to) in self.moves.keys() {
            let piece = piece_moves.entry(from).or_insert(HashSet::new());
            (*piece).insert(to);
        }
        piece_moves
    }

    pub fn get_turn(&self) -> Colour {
        self.turn
    }

    pub fn king_in_check(&self) -> Option<Position> {
        if self.in_check() {
            for (pos, piece) in self.pieces.iter() {
                if piece.colour == self.turn && piece.variant == King {
                    return Some(*pos);
                }
            }
        }
        None
    }

    fn in_check(&self) -> bool {
        self.gen_capture_moves(self.turn.flipped()).is_none()
    }

    pub fn promoting(&self) -> bool {
        self.info.promoting.is_some()
    }

    pub fn ended(&self) -> &Option<EndState> {
        &self.ended
    }

    pub fn threefold_repetition(&mut self) {
        self.ended = Some(EndState::ThreefoldRepetition);
        self.moves = HashMap::new();
    }
}

#[derive(Clone)]
pub enum EndState {
    Checkmate(Colour),
    Stalemate,
    InsufficientMaterial,
    ThreefoldRepetition,
}

#[derive(Clone, Copy, Default)]
struct MoveInfo {
    state_info: StateInfo,
    move_type: MoveType,
}

#[derive(Clone, Copy)]
enum MoveType {
    Normal,
    Castle(Position, Position),
    EnPassant(Position),
}

impl Default for MoveType {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Clone, Copy)]
struct StateInfo {
    white_short: bool,
    white_long: bool,
    black_short: bool,
    black_long: bool,
    en_passant: Option<Position>,
    promoting: Option<Position>,
}

impl StateInfo {
    fn apply(&mut self, other: StateInfo) {
        self.white_short &= other.white_short;
        self.white_long &= other.white_long;
        self.black_short &= other.black_short;
        self.black_long &= other.black_long;
        self.en_passant = other.en_passant;
        self.promoting = other.promoting;
    }
}

impl Default for StateInfo {
    fn default() -> Self {
        StateInfo {
            white_short: true,
            white_long: true,
            black_short: true,
            black_long: true,
            en_passant: None,
            promoting: None,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Position {
    value: u8,
}

impl Position {
    pub fn from_xy(x: i8, y: i8) -> Option<Position> {
        if x >= 0 && y >= 0 && x < 8 && y < 8 {
            return Some(Position {
                value: ((y as u8) << 3) | (x as u8),
            });
        }
        None
    }

    pub fn offset_by(&self, x: i8, y: i8) -> Option<Position> {
        Position::from_xy((self.value & 0b111) as i8 + x, (self.value >> 3) as i8 + y)
    }

    pub fn get_x(&self) -> i8 {
        (self.value & 0b111) as i8
    }

    pub fn get_y(&self) -> i8 {
        (self.value >> 3) as i8
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub struct Piece {
    pub colour: Colour,
    pub variant: Variant,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Colour {
    White,
    Black,
}

impl Colour {
    fn flip(&mut self) {
        *self = if self == &White { Black } else { White };
    }

    fn flipped(&self) -> Colour {
        if self == &White {
            Black
        } else {
            White
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Variant {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl PartialEq for State {
    fn eq(&self, other: &State) -> bool {
        self.pieces == other.pieces
            && self.turn == other.turn
            && self.info.white_short == other.info.white_short
            && self.info.white_long == other.info.white_long
            && self.info.black_short == other.info.black_short
            && self.info.black_long == other.info.black_long
            && self.info.en_passant == other.info.en_passant
    }
}
impl Eq for State {}
