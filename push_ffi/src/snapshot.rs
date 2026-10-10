//! Records the screen draws. The shoe size, the melds, and the local hand come from `push_core`.

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

/// The shoe size, the round, the melds, and the local seat's hand.
/// Opponent hands stay off this record.
#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct TableSnapshot {
    pub deck_size: u32,
    pub round_number: u32,
    pub board: Vec<MeldSnapshot>,
    pub hand: Vec<CardSnapshot>,
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
        hand: exhibit_hand(),
    }
}

/// Copies the draw pile, the round, `state.board`, and seat 0's hand.
pub fn snapshot_from_state(state: &GameState) -> TableSnapshot {
    TableSnapshot {
        deck_size: state.deck.cards.len() as u32,
        round_number: u32::from(state.round_number),
        board: state
            .board
            .iter()
            .map(|cards| meld(cards.clone()))
            .collect(),
        hand: local_hand(state),
    }
}

/// Seat 0 is the local seat. Another seat's cards are not copied.
fn local_hand(state: &GameState) -> Vec<CardSnapshot> {
    state
        .players
        .iter()
        .find(|player| player.seat_index == 0)
        .map(|player| player.hand.iter().copied().map(card_snapshot).collect())
        .unwrap_or_default()
}

/// 8♦, K♠, a locked joker, and 2♣. These cards are the hand, not a deal.
fn exhibit_hand() -> Vec<CardSnapshot> {
    vec![
        card_snapshot(card(21, Suit::Diamonds, Rank::Eight)),
        card_snapshot(card(22, Suit::Spades, Rank::King)),
        card_snapshot(Card {
            id: 23,
            suit: Suit::None,
            rank: Rank::Joker,
            locked_until_turn: 2,
        }),
        card_snapshot(card(24, Suit::Clubs, Rank::Two)),
    ]
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

pub(crate) fn card_from_snapshot(card: CardSnapshot) -> Card {
    Card {
        id: card.id,
        suit: suit_from(card.suit),
        rank: rank_from(card.rank),
        locked_until_turn: card.locked_until_turn,
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

fn suit_from(suit: CardSuit) -> Suit {
    match suit {
        CardSuit::Hearts => Suit::Hearts,
        CardSuit::Diamonds => Suit::Diamonds,
        CardSuit::Clubs => Suit::Clubs,
        CardSuit::Spades => Suit::Spades,
        CardSuit::None => Suit::None,
    }
}

fn rank_from(rank: CardRank) -> Rank {
    match rank {
        CardRank::Two => Rank::Two,
        CardRank::Three => Rank::Three,
        CardRank::Four => Rank::Four,
        CardRank::Five => Rank::Five,
        CardRank::Six => Rank::Six,
        CardRank::Seven => Rank::Seven,
        CardRank::Eight => Rank::Eight,
        CardRank::Nine => Rank::Nine,
        CardRank::Ten => Rank::Ten,
        CardRank::Jack => Rank::Jack,
        CardRank::Queen => Rank::Queen,
        CardRank::King => Rank::King,
        CardRank::Ace => Rank::Ace,
        CardRank::Joker => Rank::Joker,
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
