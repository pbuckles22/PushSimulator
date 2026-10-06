//! Chain: Suit + Rank + Card, then Deck::new.
//! Two standard decks, two jokers in each deck.

use push_core::card::{Rank, Suit};
use push_core::deck::Deck;

const DECKS: usize = 2;
const JOKERS_PER_DECK: usize = 2;
const SUITS_PER_DECK: usize = 4;

#[test]
fn test_suit_rank_card_deck_new_two_decks_two_jokers_each() {
    let deck = Deck::new();
    let cards = &deck.cards;

    assert_eq!(cards.len(), 52 * DECKS + JOKERS_PER_DECK * DECKS);

    let jokers: Vec<_> = cards
        .iter()
        .filter(|card| card.rank == Rank::Joker)
        .collect();
    assert_eq!(jokers.len(), DECKS * JOKERS_PER_DECK);
    assert!(
        jokers.iter().all(|card| card.suit == Suit::None),
        "jokers use Suit::None"
    );

    let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
    let ranks = [
        Rank::Two,
        Rank::Three,
        Rank::Four,
        Rank::Five,
        Rank::Six,
        Rank::Seven,
        Rank::Eight,
        Rank::Nine,
        Rank::Ten,
        Rank::Jack,
        Rank::Queen,
        Rank::King,
        Rank::Ace,
    ];
    for suit in suits {
        for rank in ranks {
            let count = cards
                .iter()
                .filter(|card| card.suit == suit && card.rank == rank)
                .count();
            assert_eq!(
                count, DECKS,
                "{rank:?} of {suit:?} should appear once per deck"
            );
        }
    }

    let suited = cards.iter().filter(|card| card.rank != Rank::Joker).count();
    assert_eq!(suited, ranks.len() * SUITS_PER_DECK * DECKS);

    let ids: std::collections::HashSet<_> = cards.iter().map(|card| card.id).collect();
    assert_eq!(ids.len(), cards.len(), "each copy has its own id");
    assert!(cards.iter().all(|card| card.locked_until_turn == 0));
}

#[test]
fn test_suit_rank_card_deck_new_is_wild_twelve_wilds() {
    let deck = Deck::new();
    let wilds: Vec<_> = deck.cards.iter().filter(|card| card.is_wild()).collect();

    assert_eq!(
        wilds.len(),
        DECKS * JOKERS_PER_DECK + DECKS * SUITS_PER_DECK
    );
    assert!(wilds
        .iter()
        .all(|card| card.rank == Rank::Joker || card.rank == Rank::Two));
    assert!(deck.cards.iter().all(|card| {
        let expect = card.rank == Rank::Joker || card.rank == Rank::Two;
        card.is_wild() == expect
    }));
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw() {
    let mut deck = Deck::new();
    let before_ids: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    assert_eq!(deck.cards.len(), 52 * DECKS + JOKERS_PER_DECK * DECKS);
    assert_eq!(
        deck.cards.iter().filter(|card| card.is_wild()).count(),
        DECKS * JOKERS_PER_DECK + DECKS * SUITS_PER_DECK
    );

    deck.shuffle();

    let after_ids: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    assert_ne!(before_ids, after_ids, "shuffle changes card order");
    let mut before_sorted = before_ids.clone();
    let mut after_sorted = after_ids.clone();
    before_sorted.sort_unstable();
    after_sorted.sort_unstable();
    assert_eq!(before_sorted, after_sorted, "shuffle keeps the same cards");
    assert_eq!(
        deck.cards.iter().filter(|card| card.is_wild()).count(),
        DECKS * JOKERS_PER_DECK + DECKS * SUITS_PER_DECK
    );
    assert!(deck.cards.iter().all(|card| {
        let expect = card.rank == Rank::Joker || card.rank == Rank::Two;
        card.is_wild() == expect
    }));

    let drawn = deck.draw().expect("shuffled deck still has cards");
    assert!(before_ids.contains(&drawn.id));
    assert_eq!(deck.cards.len(), 107);
    assert!(deck.cards.iter().all(|card| card.id != drawn.id));
    assert_eq!(
        drawn.is_wild(),
        drawn.rank == Rank::Joker || drawn.rank == Rank::Two
    );

    while deck.draw().is_some() {}
    assert!(deck.draw().is_none());
    assert!(deck.cards.is_empty());
}
