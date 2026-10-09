//! Records the screen draws. The shoe size and the melds come from `push_core`.

use push_core::card::{Card, Rank, Suit};
use push_core::game_state::GameState;

/// Suit on a card the screen can draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum CardSuit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
    None,
}

/// Rank on a card the screen can draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum CardRank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
    Joker,
}

/// One card on the table the screen draws.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct CardSnapshot {
    pub id: u32,
    pub suit: CardSuit,
    pub rank: CardRank,
    pub locked_until_turn: u32,
}

/// One meld, in the order the cards sit on the board.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct MeldSnapshot {
    pub cards: Vec<CardSnapshot>,
}

/// The shoe size, the round, and the melds. Hands stay off this record.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct TableSnapshot {
    pub deck_size: u32,
    pub round_number: u32,
    pub board: Vec<MeldSnapshot>,
}

/// A new table: the full shoe, round 1, and no melds.
pub fn empty_snapshot(deck_size: u32) -> TableSnapshot {
    TableSnapshot {
        deck_size,
        round_number: 1,
        board: Vec::new(),
    }
}

/// The set of fours and the heart run the screen lays out.
///
/// The shoe size stays the undealt shoe. These cards are the board, not a deal.
pub fn exhibit_snapshot(deck_size: u32) -> TableSnapshot {
    TableSnapshot {
        deck_size,
        round_number: 2,
        board: vec![
            meld(vec![
                card(1, Suit::Hearts, Rank::Four),
                card(2, Suit::Spades, Rank::Four),
                card(3, Suit::Clubs, Rank::Four),
            ]),
            meld(vec![
                card(10, Suit::Hearts, Rank::Four),
                card(11, Suit::Hearts, Rank::Five),
                card(12, Suit::Hearts, Rank::Six),
                card(13, Suit::Hearts, Rank::Seven),
            ]),
        ],
    }
}

/// Copies the draw pile, the round, and `state.board` in the order they sit.
pub fn snapshot_from_state(state: &GameState) -> TableSnapshot {
    TableSnapshot {
        deck_size: state.deck.cards.len() as u32,
        round_number: u32::from(state.round_number),
        board: state
            .board
            .iter()
            .map(|cards| meld(cards.clone()))
            .collect(),
    }
}

fn meld(cards: Vec<Card>) -> MeldSnapshot {
    MeldSnapshot {
        cards: cards.iter().copied().map(card_snapshot).collect(),
    }
}

fn card(id: u32, suit: Suit, rank: Rank) -> Card {
    Card {
        id,
        suit,
        rank,
        locked_until_turn: 0,
    }
}

fn card_snapshot(card: Card) -> CardSnapshot {
    CardSnapshot {
        id: card.id,
        suit: suit_name(card.suit),
        rank: rank_name(card.rank),
        locked_until_turn: card.locked_until_turn,
    }
}

fn suit_name(suit: Suit) -> CardSuit {
    match suit {
        Suit::Hearts => CardSuit::Hearts,
        Suit::Diamonds => CardSuit::Diamonds,
        Suit::Clubs => CardSuit::Clubs,
        Suit::Spades => CardSuit::Spades,
        Suit::None => CardSuit::None,
    }
}

fn rank_name(rank: Rank) -> CardRank {
    match rank {
        Rank::Two => CardRank::Two,
        Rank::Three => CardRank::Three,
        Rank::Four => CardRank::Four,
        Rank::Five => CardRank::Five,
        Rank::Six => CardRank::Six,
        Rank::Seven => CardRank::Seven,
        Rank::Eight => CardRank::Eight,
        Rank::Nine => CardRank::Nine,
        Rank::Ten => CardRank::Ten,
        Rank::Jack => CardRank::Jack,
        Rank::Queen => CardRank::Queen,
        Rank::King => CardRank::King,
        Rank::Ace => CardRank::Ace,
        Rank::Joker => CardRank::Joker,
    }
}
