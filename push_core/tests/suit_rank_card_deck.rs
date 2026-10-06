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
