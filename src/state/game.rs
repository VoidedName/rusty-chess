use crate::state::board::PieceKind;
use std::cmp::{max, min};
use std::collections::HashMap;

#[derive(Copy, Clone, Debug)]
pub struct CastlingAvailability {
    long_side_available: bool,
    short_side_available: bool,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum PlayerColor {
    Black,
    White,
}

impl PlayerColor {
    pub fn oppnent(&self) -> PlayerColor {
        match self {
            PlayerColor::Black => PlayerColor::White,
            PlayerColor::White => PlayerColor::Black,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Piece {
    pub kind: PieceKind,
    pub color: PlayerColor,
}

#[derive(Clone, Debug)]
pub enum Interaction {
    StartMovingPiece(Position),
    PlacedPiece(Position),
    PickingPromotion(Position, Position, Vec<Piece>),
    PickedPromotion(Piece),
}

#[derive(Clone, Debug)]
pub struct GameState {
    castling_white: CastlingAvailability,
    castling_black: CastlingAvailability,
    white_king: Position,
    black_king: Position,
    board: Vec<Option<Piece>>,
    state: GamePhase,
    // previous_state: Option<Box<GameState>>,
    previous_en_passe_move: Option<Position>,
    interaction: Option<Interaction>,
    moves_since_interesting: u8,
    position_counter: HashMap<Vec<Option<Piece>>, u8>,
}

#[derive(Copy, Clone, Debug)]
pub enum DrawReason {
    Fifty,
    Repeat,
    Stalemate,
}

#[derive(Copy, Clone, Debug)]
pub enum GamePhase {
    Won(PlayerColor),
    Draw(DrawReason),
    Turn(PlayerColor),
}

#[derive(Copy, Clone)]
pub enum CastleType {
    Long,
    Short,
}

pub struct CastlingMovement {
    pub rook_start: Position,
    pub king_start: Position,

    pub rook_end: Position,
    pub king_end: Position,
}

impl CastleType {
    pub fn positions(&self, color: PlayerColor) -> CastlingMovement {
        let rank = match color {
            PlayerColor::Black => 0,
            PlayerColor::White => 7,
        };
        match self {
            CastleType::Long => CastlingMovement {
                rook_start: Position(0, rank),
                king_start: Position(4, rank),
                rook_end: Position(3, rank),
                king_end: Position(2, rank),
            },
            CastleType::Short => CastlingMovement {
                rook_start: Position(7, rank),
                king_start: Position(4, rank),
                rook_end: Position(5, rank),
                king_end: Position(6, rank),
            },
        }
    }
}

#[derive(Copy, Clone)]
pub enum Move {
    Move(Position),
    // to
    Take(Position, Position),
    // to, take on ( pawns en passe -_- )
    Promote(Position, Piece),
    // to, transform to
    Castle(CastleType), // rook
}

impl GameState {
    pub fn new() -> Self {
        let mut board = vec![None; 64];

        board[0] = Some(Piece {
            kind: PieceKind::Rook,
            color: PlayerColor::Black,
        });
        board[1] = Some(Piece {
            kind: PieceKind::Knight,
            color: PlayerColor::Black,
        });
        board[2] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PlayerColor::Black,
        });
        board[3] = Some(Piece {
            kind: PieceKind::Queen,
            color: PlayerColor::Black,
        });
        board[4] = Some(Piece {
            kind: PieceKind::King,
            color: PlayerColor::Black,
        });
        board[5] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PlayerColor::Black,
        });
        board[6] = Some(Piece {
            kind: PieceKind::Knight,
            color: PlayerColor::Black,
        });
        board[7] = Some(Piece {
            kind: PieceKind::Rook,
            color: PlayerColor::Black,
        });

        for x in 8..16 {
            board[x] = Some(Piece {
                kind: PieceKind::Pawn,
                color: PlayerColor::Black,
            })
        }

        for x in 48..56 {
            board[x] = Some(Piece {
                kind: PieceKind::Pawn,
                color: PlayerColor::White,
            })
        }

        board[56] = Some(Piece {
            kind: PieceKind::Rook,
            color: PlayerColor::White,
        });
        board[57] = Some(Piece {
            kind: PieceKind::Knight,
            color: PlayerColor::White,
        });
        board[58] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PlayerColor::White,
        });
        board[59] = Some(Piece {
            kind: PieceKind::Queen,
            color: PlayerColor::White,
        });
        board[60] = Some(Piece {
            kind: PieceKind::King,
            color: PlayerColor::White,
        });
        board[61] = Some(Piece {
            kind: PieceKind::Bishop,
            color: PlayerColor::White,
        });
        board[62] = Some(Piece {
            kind: PieceKind::Knight,
            color: PlayerColor::White,
        });
        board[63] = Some(Piece {
            kind: PieceKind::Rook,
            color: PlayerColor::White,
        });

        Self {
            castling_white: CastlingAvailability {
                long_side_available: true,
                short_side_available: true,
            },
            castling_black: CastlingAvailability {
                long_side_available: true,
                short_side_available: true,
            },
            white_king: Position(4, 7),
            black_king: Position(4, 0),
            board,
            state: GamePhase::Turn(PlayerColor::White),
            // previous_state: None,
            interaction: None,
            previous_en_passe_move: None,
            moves_since_interesting: 0,
            position_counter: HashMap::new(),
        }
    }

    pub fn interact(mut self, interaction: Interaction) -> Self {
        let player = match self.state {
            GamePhase::Won(_) => return self,
            GamePhase::Draw(_) => return self,
            GamePhase::Turn(player) => player,
        };

        match &interaction {
            Interaction::StartMovingPiece(start) => {
                if let Some(piece) = self.piece_at(*start) {
                    if piece.color == player {
                        self.interaction = Some(interaction);
                    }
                }
                self
            }
            Interaction::PlacedPiece(onto) => match &self.interaction {
                Some(Interaction::StartMovingPiece(from)) if from != onto => {
                    if let Some(piece) = self.board[from.idx()] {
                        let moves = piece.moves(*from, &self);
                        let moves = moves.iter().filter(|m| match m {
                            Move::Move(p) => p == onto,
                            Move::Take(p, _) => p == onto,
                            Move::Promote(p, _) => p == onto,
                            Move::Castle(side) => side.positions(player).king_end == *onto,
                        }).collect::<Vec<_>>();

                        if moves.len() <= 1 {
                            if let Some(m) = moves.get(0) {
                                let mut next = self.next(*from, **m);
                                next.interaction = None;
                                next
                            } else {
                                self.interaction = None;
                                self
                            }
                        } else {
                            let promotions = moves.iter().filter_map(|m| {
                                if let Move::Promote(_, piece) = m {
                                    Some(*piece)
                                } else {
                                    None
                                }
                            }).collect();

                            self.interaction = Some(Interaction::PickingPromotion(*from, *onto, promotions));
                            self
                        }
                    } else {
                        self
                    }
                }
                Some(Interaction::StartMovingPiece(_)) => {
                    self.interaction = None;
                    self
                }
                _ => self,
            },
            Interaction::PickedPromotion(choice) => {
                if let Some(Interaction::PickingPromotion(from, onto, ..)) = self.interaction {
                    self.interaction = None;
                    self.next(from, Move::Promote(onto, *choice))
                } else {
                    self.interaction = None;
                    self
                }
            }
            _ => self,
        }
    }

    pub fn position_is_attacked_by(&self, position: Position, player: PlayerColor) -> bool {
        for x in 0..8 {
            for y in 0..8 {
                let pos = Position(x, y);
                if let Some(piece) = self.board[pos.idx()] {
                    if piece.color == player
                        && piece
                            .attacks(pos, self)
                            .iter()
                            .any(|target| *target == position)
                    {
                        return true;
                    }
                }
            }
        }

        false
    }

    fn next(&self, piece: Position, m: Move) -> GameState {
        let player = match self.state {
            GamePhase::Won(_) => {
                return self.clone();
            }
            GamePhase::Draw(_) => {
                return self.clone();
            }
            GamePhase::Turn(player) => player,
        };

        let mut new = Self {
            castling_white: self.castling_white,
            castling_black: self.castling_black,
            white_king: self.white_king,
            black_king: self.black_king,
            board: self.board.clone(),
            state: self.state,
            // previous_state: Some(Box::new(self.clone())),
            interaction: self.interaction.clone(),
            previous_en_passe_move: self.previous_en_passe_move,
            moves_since_interesting: self.moves_since_interesting,
            position_counter: self.position_counter.clone(),
        };

        let castling = match player {
            PlayerColor::Black => &mut new.castling_black,
            PlayerColor::White => &mut new.castling_white,
        };

        let from = piece.idx();
        let moved_piece = match self.board[from] {
            None => {
                return self.clone();
            }
            Some(piece) => piece,
        };

        if moved_piece.kind == PieceKind::King {
            castling.long_side_available = false;
            castling.short_side_available = false;
        }

        if moved_piece.kind == PieceKind::Rook {
            if piece.0 == 0 {
                castling.long_side_available = false;
            } else if piece.0 == 7 {
                castling.short_side_available = false;
            }
        }

        new.previous_en_passe_move = None;
        new.moves_since_interesting += 1;

        match m {
            Move::Move(to) => {
                if let Some(Piece {
                    kind: PieceKind::Pawn,
                    ..
                }) = self.board[from]
                {
                    if (to.1 - piece.1).abs() == 2 {
                        new.previous_en_passe_move = Some(to);
                    }
                    new.moves_since_interesting = 0;
                }

                if let Some(Piece {
                    kind: PieceKind::King,
                    ..
                }) = self.board[from]
                {
                    match player {
                        PlayerColor::Black => new.black_king = to,
                        PlayerColor::White => new.white_king = to,
                    }
                }

                let to = to.idx();

                new.board[to] = new.board[from];
                new.board[from] = None;
            }
            Move::Take(to, victim) => {
                if let Some(Piece {
                    kind: PieceKind::King,
                    ..
                }) = self.board[from]
                {
                    match player {
                        PlayerColor::Black => new.black_king = to,
                        PlayerColor::White => new.white_king = to,
                    }
                }

                let to = to.idx();
                let victim = victim.idx();
                new.board[victim] = None;
                new.board[to] = new.board[from];
                new.board[from] = None;

                new.moves_since_interesting = 0;
            }
            Move::Promote(to, butterfly) => {
                let to = to.idx();
                new.board[to] = Some(butterfly);
                new.board[from] = None;
                new.moves_since_interesting = 0;
            }
            Move::Castle(side) => {
                let CastlingMovement {
                    rook_start,
                    rook_end,
                    king_end,
                    ..
                } = side.positions(player);

                match player {
                    PlayerColor::Black => new.black_king = king_end,
                    PlayerColor::White => new.white_king = king_end,
                }

                new.board[rook_end.idx()] = new.board[rook_start.idx()];
                new.board[king_end.idx()] = new.board[from];

                new.board[rook_start.idx()] = None;
                new.board[from] = None;
            }
        };

        new.state = GamePhase::Turn(player.oppnent());
        let mut counter: u8 = new.position_counter.get(&new.board).map_or(0, |x| *x);
        counter += 1;
        new.position_counter.insert(new.board.clone(), counter);

        let mut legal_moves = false;
        for x in 0..8 {
            for y in 0..8 {
                let pos = Position(x, y);
                if let Some(piece) = new.board[pos.idx()] {
                    if piece.color != player && piece.moves(pos, &new).len() > 0 {
                        legal_moves = true;
                    }
                }
            }
        }

        if !legal_moves {
            if match player {
                PlayerColor::Black => new.position_is_attacked_by(new.white_king, player),
                PlayerColor::White => new.position_is_attacked_by(new.black_king, player),
            } {
                new.state = GamePhase::Won(player);
            } else {
                new.state = GamePhase::Draw(DrawReason::Stalemate);
            }
        }
        if counter == 3 {
            new.state = GamePhase::Draw(DrawReason::Repeat);
        }
        if new.moves_since_interesting >= 50 {
            new.state = GamePhase::Draw(DrawReason::Fifty);
        }

        new
    }

    pub fn piece_at(&self, position: Position) -> Option<&Piece> {
        self.board[position.idx()].as_ref()
    }

    pub fn phase(&self) -> GamePhase {
        self.state
    }

    pub fn interaction(&self) -> Option<&Interaction> {
        self.interaction.as_ref()
    }

    pub fn is_valid_position_on_board(pos: Position) -> bool {
        pos.0 >= 0 && pos.0 < 8 && pos.1 >= 0 && pos.1 < 8
    }
}

const ROOK_MOVEMENT: [(i32, i32); 4] = [(-1, 0), (1, 0), (0, -1), (0, 1)];
const QUEEN_MOVEMENT: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];
const KING_MOVEMENT: [(i32, i32); 8] = [
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
    (0, -1),
    (0, 1),
    (-1, 0),
    (1, 0),
];
const KNIGHT_MOVEMENT: [(i32, i32); 8] = [
    (-1, -2),
    (-1, 2),
    (1, -2),
    (1, 2),
    (-2, -1),
    (-2, 1),
    (2, -1),
    (2, 1),
];
const BISHOP_MOVEMENT: [(i32, i32); 4] = [(-1, -1), (-1, 1), (1, -1), (1, 1)];

impl Piece {
    fn add_attacks_single_step(start: Position, step: (i32, i32), positions: &mut Vec<Position>) {
        let pos = Position(start.0 + step.0, start.1 + step.1);
        if GameState::is_valid_position_on_board(pos) {
            positions.push(pos)
        }
    }

    fn add_attacks_pattern(
        start: Position,
        step: (i32, i32),
        board: &GameState,
        positions: &mut Vec<Position>,
    ) {
        let mut pos = Position(start.0 + step.0, start.1 + step.1);
        loop {
            if !GameState::is_valid_position_on_board(pos) {
                break;
            }

            positions.push(pos);

            if let Some(_) = board.piece_at(pos) {
                break;
            }

            pos = Position(pos.0 + step.0, pos.1 + step.1);
        }
    }

    pub fn attacks(&self, position: Position, board: &GameState) -> Vec<Position> {
        let mut attacks = vec![];

        match self.kind {
            PieceKind::Pawn => {
                let dir = match self.color {
                    PlayerColor::Black => 1,
                    PlayerColor::White => -1,
                };
                for side in [-1, 1] {
                    Self::add_attacks_single_step(position, (side, dir), &mut attacks)
                }
            }
            PieceKind::Rook => {
                for dir in ROOK_MOVEMENT {
                    Self::add_attacks_pattern(position, dir, board, &mut attacks)
                }
            }
            PieceKind::Knight => {
                for dir in KNIGHT_MOVEMENT {
                    Self::add_attacks_single_step(position, dir, &mut attacks)
                }
            }
            PieceKind::Bishop => {
                for dir in BISHOP_MOVEMENT {
                    Self::add_attacks_pattern(position, dir, board, &mut attacks)
                }
            }
            PieceKind::King => {
                for dir in KING_MOVEMENT {
                    Self::add_attacks_single_step(position, dir, &mut attacks)
                }
            }
            PieceKind::Queen => {
                for dir in QUEEN_MOVEMENT {
                    Self::add_attacks_pattern(position, dir, board, &mut attacks)
                }
            }
        }

        attacks
    }

    pub fn moves(&self, position: Position, board: &GameState) -> Vec<Move> {
        let mut local_board = board.clone();
        let mut moves = vec![];
        let opponent = self.color.oppnent();
        let king_position = match self.color {
            PlayerColor::Black => board.black_king,
            PlayerColor::White => board.white_king,
        };

        match self.kind {
            PieceKind::Pawn => {
                // remove from board, i.e. detect pins
                local_board.board[position.idx()] = None;

                let dir = match self.color {
                    PlayerColor::Black => 1,
                    PlayerColor::White => -1,
                };
                let start_rank = match self.color {
                    PlayerColor::Black => 1,
                    PlayerColor::White => 6,
                };
                let promotion_rank = match self.color {
                    PlayerColor::Black => 7,
                    PlayerColor::White => 0,
                };

                // pawn forward
                let forward = Position(position.0, position.1 + dir);
                if let None = board.piece_at(forward) {
                    local_board.board[forward.idx()] = Some(*self);

                    if !local_board.position_is_attacked_by(king_position, opponent) {
                        if forward.1 == promotion_rank {
                            for kind in [
                                PieceKind::Queen,
                                PieceKind::Rook,
                                PieceKind::Knight,
                                PieceKind::Bishop,
                            ] {
                                moves.push(Move::Promote(
                                    forward,
                                    Piece {
                                        color: self.color,
                                        kind,
                                    },
                                ));
                            }
                        } else {
                            moves.push(Move::Move(forward));
                        }
                    }
                    local_board.board[forward.idx()] = None;

                    let forward = Position(forward.0, forward.1 + dir);
                    if start_rank == position.1 {
                        if let None = board.piece_at(forward) {
                            local_board.board[forward.idx()] = Some(*self);
                            if !local_board.position_is_attacked_by(king_position, opponent) {
                                moves.push(Move::Move(forward));
                            }
                            local_board.board[forward.idx()] = None;
                        }
                    }
                }

                // pawn take
                for pos in self.attacks(position, board) {
                    let target_piece = local_board.board[pos.idx()];

                    // placing, i.e. detect blocking lines / removing attacks
                    local_board.board[pos.idx()] = Some(*self);
                    match target_piece {
                        Some(Piece { color, .. }) => {
                            if color == opponent {
                                if !local_board.position_is_attacked_by(king_position, opponent) {
                                    if pos.1 == promotion_rank {
                                        for kind in [
                                            PieceKind::Queen,
                                            PieceKind::Rook,
                                            PieceKind::Knight,
                                            PieceKind::Bishop,
                                        ] {
                                            moves.push(Move::Promote(
                                                pos,
                                                Piece {
                                                    color: self.color,
                                                    kind,
                                                },
                                            ));
                                        }
                                    } else {
                                        moves.push(Move::Take(pos, pos))
                                    }
                                }
                            }
                        }
                        None => {
                            let en_passe_pos = Position(pos.0, pos.1 - dir);
                            if Some(en_passe_pos) == board.previous_en_passe_move {
                                let en_passe_pawn = local_board.board[en_passe_pos.idx()];
                                local_board.board[en_passe_pos.idx()] = None;

                                if !local_board.position_is_attacked_by(king_position, opponent) {
                                    moves.push(Move::Take(pos, en_passe_pos))
                                }

                                local_board.board[en_passe_pos.idx()] = en_passe_pawn;
                            }
                        }
                    }

                    local_board.board[pos.idx()] = target_piece;
                }
            }
            PieceKind::Queen | PieceKind::Rook | PieceKind::Bishop | PieceKind::Knight => {
                // remove from board, i.e. detect pins
                local_board.board[position.idx()] = None;

                for pos in self.attacks(position, board).into_iter() {
                    let target_piece = local_board.board[pos.idx()];

                    // placing, i.e. detect blocking lines / removing attacks
                    local_board.board[pos.idx()] = Some(*self);

                    match target_piece {
                        None => {
                            if !local_board.position_is_attacked_by(king_position, opponent) {
                                moves.push(Move::Move(pos))
                            }
                        }
                        Some(Piece { color, .. }) if color == opponent => {
                            if !local_board.position_is_attacked_by(king_position, opponent) {
                                moves.push(Move::Take(pos, pos))
                            }
                        }
                        _ => {}
                    }

                    local_board.board[pos.idx()] = target_piece;
                }
            }
            PieceKind::King => {
                for pos in self.attacks(position, board).into_iter() {
                    if board.position_is_attacked_by(pos, opponent) {
                        continue;
                    }

                    match board.piece_at(pos) {
                        None => moves.push(Move::Move(pos)),
                        Some(other) if other.color == opponent => moves.push(Move::Take(pos, pos)),
                        _ => {}
                    };
                }

                let castling = match self.color {
                    PlayerColor::Black => board.castling_black,
                    PlayerColor::White => board.castling_white,
                };

                let mut castles = vec![];
                if castling.short_side_available {
                    castles.push(CastleType::Short)
                }
                if castling.long_side_available {
                    castles.push(CastleType::Long)
                }

                for castle in castles {
                    let CastlingMovement {
                        rook_start,
                        king_start,
                        ..
                    } = castle.positions(self.color);
                    let mut legal = true;
                    let start = min(rook_start.0, king_start.0);
                    let end = max(rook_start.0, king_start.0);
                    for x in start..=end {
                        let test_position = Position(x, rook_start.1);

                        if x != start && x != end {
                            if let Some(_) = board.piece_at(test_position) {
                                legal = false;
                                break;
                            }
                        }

                        if board.position_is_attacked_by(test_position, opponent) {
                            legal = false;
                            break;
                        }
                    }

                    if legal {
                        moves.push(Move::Castle(castle))
                    }
                }
            }
        }

        moves
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct Position(pub i32, pub i32);

impl Position {
    pub fn idx(&self) -> usize {
        (self.0 + self.1 * 8) as usize
    }
}
