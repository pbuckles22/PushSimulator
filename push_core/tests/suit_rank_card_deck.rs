//! Chain: Suit + Rank + Card, then Deck::new.
//! Two standard decks, two jokers in each deck.

use push_core::actions::Action;
use push_core::card::{Card, Rank, Suit};
use push_core::deck::{Deck, TurnDraw};
use push_core::player::{deal_initial_hands, Player};
use push_core::validation::{validate_run, validate_set};

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

    let drawn = ordinary(deck.draw());
    assert!(before_ids.contains(&drawn.id));
    assert_eq!(deck.cards.len(), 107);
    assert!(deck.cards.iter().all(|card| card.id != drawn.id));
    assert_eq!(
        drawn.is_wild(),
        drawn.rank == Rank::Joker || drawn.rank == Rank::Two
    );

    loop {
        match deck.draw() {
            TurnDraw::One(_) => {}
            TurnDraw::Empty => break,
            other => panic!("an empty discard drains with ordinary draws, got {other:?}"),
        }
    }
    assert_eq!(deck.draw(), TurnDraw::Empty);
    assert!(deck.cards.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffles() {
    let mut deck = Deck::new();
    let before_ids: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    assert_eq!(deck.cards.len(), 52 * DECKS + JOKERS_PER_DECK * DECKS);
    assert!(deck.discard.is_empty());
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

    let first = ordinary(deck.draw());
    assert!(before_ids.contains(&first.id));
    assert_eq!(
        first.is_wild(),
        first.rank == Rank::Joker || first.rank == Rank::Two
    );

    let mut recycled = Vec::new();
    loop {
        match deck.draw() {
            TurnDraw::One(card) => recycled.push(card),
            TurnDraw::Empty => break,
            other => panic!("an empty discard drains with ordinary draws, got {other:?}"),
        }
    }
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    assert_eq!(recycled.len() + 1, before_ids.len());

    let top = recycled
        .pop()
        .expect("drained deck still has a discard top");
    deck.discard = recycled;
    deck.discard.push(top);

    let under_ids: Vec<u32> = deck.discard[..deck.discard.len() - 1]
        .iter()
        .map(|card| card.id)
        .collect();
    let drawn = ordinary(deck.draw());
    assert_ne!(drawn.id, top.id);
    assert!(under_ids.contains(&drawn.id));
    assert_eq!(deck.discard.len(), 1);
    assert_eq!(deck.discard[0].id, top.id);

    let mut left: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    left.push(drawn.id);
    left.sort_unstable();
    let mut expect = under_ids;
    expect.sort_unstable();
    assert_eq!(left, expect);
}

fn deck_size() -> usize {
    52 * DECKS + JOKERS_PER_DECK * DECKS
}

fn wild_count() -> usize {
    DECKS * JOKERS_PER_DECK + DECKS * SUITS_PER_DECK
}

fn assert_wilds(cards: &[Card]) {
    assert_eq!(
        cards.iter().filter(|card| card.is_wild()).count(),
        wild_count()
    );
    assert!(cards.iter().all(|card| {
        let expect = card.rank == Rank::Joker || card.rank == Rank::Two;
        card.is_wild() == expect
    }));
}

fn ids_of(cards: &[Card]) -> Vec<u32> {
    cards.iter().map(|card| card.id).collect()
}

fn same_ids(left: &[u32], right: &[u32]) {
    let mut left_sorted = left.to_vec();
    let mut right_sorted = right.to_vec();
    left_sorted.sort_unstable();
    right_sorted.sort_unstable();
    assert_eq!(left_sorted, right_sorted);
}

/// Suit → Rank → Card → Deck::new → is_wild → shuffle.
fn shuffled_deck() -> Deck {
    let mut deck = Deck::new();
    assert_eq!(deck.cards.len(), deck_size());
    assert!(deck.discard.is_empty());
    assert_wilds(&deck.cards);
    let before = ids_of(&deck.cards);
    deck.shuffle();
    let after = ids_of(&deck.cards);
    assert_ne!(before, after, "shuffle changes card order");
    same_ids(&before, &after);
    assert_wilds(&deck.cards);
    deck
}

fn ordinary(draw: TurnDraw) -> Card {
    match draw {
        TurnDraw::One(card) => card,
        other => panic!("expected an ordinary draw, got {other:?}"),
    }
}

fn drain_without_discard(deck: &mut Deck) -> Vec<Card> {
    assert!(deck.discard.is_empty());
    let mut drawn = Vec::new();
    loop {
        match deck.draw() {
            TurnDraw::One(card) => drawn.push(card),
            TurnDraw::Empty => break,
            other => panic!("draining a draw pile should be ordinary draws, got {other:?}"),
        }
    }
    assert_eq!(deck.draw(), TurnDraw::Empty);
    assert!(deck.cards.is_empty());
    drawn
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw, discard present.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_nonempty_leaves_discard() {
    let mut deck = shuffled_deck();
    let mut pile = Vec::new();
    for _ in 0..5 {
        pile.push(ordinary(deck.draw()));
    }
    deck.discard = pile;
    let discard_before = deck.discard.clone();
    let draw_len = deck.cards.len();

    let drawn = ordinary(deck.draw());

    assert_eq!(deck.cards.len(), draw_len - 1);
    assert!(discard_before.iter().all(|card| card.id != drawn.id));
    assert_eq!(deck.discard, discard_before);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → last draw → empty reshuffle.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_last_card_then_empty_reshuffles() {
    let mut deck = shuffled_deck();
    let mut held = Vec::new();
    while deck.cards.len() > 1 {
        held.push(ordinary(deck.draw()));
    }
    let last = deck.cards[0];
    let top = held.pop().expect("held cards supply a discard top");
    let mid = held.pop().expect("held cards supply a middle discard");
    let under = held.pop().expect("held cards supply an under discard");
    deck.discard = vec![under, mid, top];
    let discard_before = deck.discard.clone();

    let drawn = ordinary(deck.draw());
    assert_eq!(drawn, last);
    assert!(deck.cards.is_empty());
    assert_eq!(deck.discard, discard_before);

    let recycled = ordinary(deck.draw());
    assert_ne!(recycled.id, top.id);
    assert!(recycled.id == under.id || recycled.id == mid.id);
    assert_eq!(deck.discard, vec![top]);
    assert_eq!(deck.cards.len(), 1);
    assert_ne!(deck.cards[0].id, recycled.id);
    assert_ne!(deck.cards[0].id, top.id);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → last card.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_last_card_goes_to_current() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let last = drawn.pop().expect("drained deck still has one card");
    deck.discard = vec![last];

    assert_eq!(deck.draw(), TurnDraw::LastCard(last));
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    assert_eq!(last.locked_until_turn, 0);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → last two cards.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_last_two_to_current_and_next() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let second = drawn.pop().expect("drained deck still has a card");
    let first = drawn.pop().expect("drained deck still has a second card");
    let side = drawn;
    deck.discard = vec![first, second];

    let TurnDraw::LastTwo { current, next } = deck.draw() else {
        panic!("the last two cards are shuffled and split");
    };
    same_ids(&ids_of(&[current, next]), &ids_of(&[first, second]));
    assert_ne!(current.id, next.id);
    assert_eq!(
        current.is_wild(),
        current.rank == Rank::Joker || current.rank == Rank::Two
    );
    assert_eq!(
        next.is_wild(),
        next.rank == Rank::Joker || next.rank == Rank::Two
    );
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    assert!(side
        .iter()
        .all(|card| card.id != current.id && card.id != next.id));
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle → drain.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_drains_to_none() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let top = drawn.pop().expect("drained deck still has a discard top");
    let under_ids = ids_of(&drawn);
    deck.discard = drawn;
    deck.discard.push(top);

    let mut got = Vec::new();
    loop {
        match deck.draw() {
            TurnDraw::One(card) => {
                assert_ne!(card.id, top.id);
                assert_eq!(
                    card.is_wild(),
                    card.rank == Rank::Joker || card.rank == Rank::Two
                );
                assert_eq!(card.locked_until_turn, 0);
                got.push(card);
            }
            TurnDraw::LastCard(card) => {
                assert_eq!(card, top);
                break;
            }
            other => panic!("expected recycled draws, then the last card, got {other:?}"),
        }
    }

    same_ids(&ids_of(&got), &under_ids);
    let mut unique = ids_of(&got);
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), got.len(), "reshuffle keeps each card id once");
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle order.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_reorders() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let top = drawn.pop().expect("drained deck still has a discard top");
    let under_ids = ids_of(&drawn);
    deck.discard = drawn;
    deck.discard.push(top);

    let first = ordinary(deck.draw());
    let mut after = ids_of(&deck.cards);
    after.push(first.id);

    assert_ne!(after, under_ids, "reshuffle changes card order");
    same_ids(&after, &under_ids);
    assert_eq!(deck.discard, vec![top]);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle wilds.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_keeps_wilds() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let top_index = drawn
        .iter()
        .position(|card| !card.is_wild())
        .expect("the deck has a non-wild card");
    let top = drawn.remove(top_index);
    let wilds_under = drawn.iter().filter(|card| card.is_wild()).count();
    assert_eq!(wilds_under, wild_count());
    deck.discard = drawn;
    deck.discard.push(top);

    let mut got = Vec::new();
    loop {
        match deck.draw() {
            TurnDraw::One(card) => got.push(card),
            TurnDraw::LastCard(card) => {
                assert_eq!(card, top);
                break;
            }
            other => panic!("expected recycled draws, then the last card, got {other:?}"),
        }
    }

    assert_eq!(
        got.iter().filter(|card| card.is_wild()).count(),
        wilds_under
    );
    assert!(got.iter().all(|card| {
        let expect = card.rank == Rank::Joker || card.rank == Rank::Two;
        card.is_wild() == expect
    }));
    assert!(!top.is_wild());
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle, side cards out.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_leaves_side_cards_out() {
    let mut deck = shuffled_deck();
    let mut side = Vec::new();
    for _ in 0..20 {
        side.push(ordinary(deck.draw()));
    }
    let mut discard_src = Vec::new();
    for _ in 0..10 {
        discard_src.push(ordinary(deck.draw()));
    }
    let top = discard_src.pop().expect("discard source has a top");
    let under_ids = ids_of(&discard_src);
    let left_out = std::mem::take(&mut deck.cards);
    assert!(!left_out.is_empty());
    deck.discard = discard_src;
    deck.discard.push(top);

    let drawn = ordinary(deck.draw());
    let mut back = ids_of(&deck.cards);
    back.push(drawn.id);

    same_ids(&back, &under_ids);
    assert!(side.iter().all(|card| !back.contains(&card.id)));
    assert!(left_out.iter().all(|card| !back.contains(&card.id)));
    assert_eq!(deck.discard, vec![top]);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle, twin copy.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_keeps_top_copy() {
    let mut deck = shuffled_deck();
    let drawn = drain_without_discard(&mut deck);
    let top_index = drawn
        .iter()
        .position(|card| {
            drawn.iter().any(|other| {
                other.id != card.id && other.suit == card.suit && other.rank == card.rank
            })
        })
        .expect("two decks share a suit and rank");
    let top = drawn[top_index];
    let twin = drawn
        .iter()
        .find(|card| card.id != top.id && card.suit == top.suit && card.rank == top.rank)
        .copied()
        .expect("the matching copy stays in the drained cards");
    let under: Vec<Card> = drawn
        .into_iter()
        .enumerate()
        .filter(|(index, _)| *index != top_index)
        .map(|(_, card)| card)
        .collect();
    deck.discard = under;
    deck.discard.push(top);

    let mut got = Vec::new();
    let last = loop {
        match deck.draw() {
            TurnDraw::One(card) => got.push(card),
            TurnDraw::LastCard(card) => break card,
            other => panic!("expected recycled draws, then the top copy, got {other:?}"),
        }
    };

    assert!(got.iter().any(|card| card.id == twin.id));
    assert!(got.iter().all(|card| card.id != top.id));
    assert_eq!(last, top);
    assert_eq!(last.suit, top.suit);
    assert_eq!(last.rank, top.rank);
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one draw-pile card, then the last discard.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_one_then_last_discard_card() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let discard_card = drawn.pop().expect("drained deck still has a discard card");
    let draw_card = drawn.pop().expect("drained deck still has a draw card");
    deck.cards = vec![draw_card];
    deck.discard = vec![discard_card];

    assert_eq!(deck.draw(), TurnDraw::One(draw_card));
    assert_eq!(deck.discard.as_slice(), &[discard_card]);
    assert_eq!(deck.draw(), TurnDraw::LastCard(discard_card));
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → two ordinary draw-pile cards.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_two_pile_cards_stay_ordinary() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let second = drawn.pop().expect("drained deck still has a card");
    let first = drawn.pop().expect("drained deck still has a second card");
    deck.cards = vec![first, second];

    assert_eq!(deck.draw(), TurnDraw::One(second));
    assert_eq!(deck.draw(), TurnDraw::One(first));
    assert!(deck.discard.is_empty());
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → Player::new.
/// A new player does not take cards from that deck.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_player_new() {
    let mut deck = Deck::new();
    let before_ids: Vec<u32> = deck.cards.iter().map(|card| card.id).collect();
    assert_eq!(deck.cards.len(), 52 * DECKS + JOKERS_PER_DECK * DECKS);
    assert_eq!(
        deck.cards.iter().filter(|card| card.is_wild()).count(),
        DECKS * JOKERS_PER_DECK + DECKS * SUITS_PER_DECK
    );

    deck.shuffle();

    let drawn = ordinary(deck.draw());
    assert!(before_ids.contains(&drawn.id));
    assert_eq!(deck.cards.len(), 107);

    let current = Player::new(1, 0);
    let next = Player::new(2, 1);
    assert_eq!(current.id, 1);
    assert_eq!(current.seat_index, 0);
    assert_eq!(current.points, 0);
    assert!(!current.is_on_board);
    assert!(current.hand.is_empty());
    assert_eq!(next.id, 2);
    assert_eq!(next.seat_index, 1);
    assert_eq!(next.points, 0);
    assert!(!next.is_on_board);
    assert!(next.hand.is_empty());
    assert_eq!(deck.cards.len(), 107);
    assert!(deck.cards.iter().all(|card| card.id != drawn.id));
    assert!(current.hand.iter().all(|card| card.id != drawn.id));
    assert!(next.hand.iter().all(|card| card.id != drawn.id));
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → last card → Player::new.
/// The leftover card is a draw result. Creating the current player does not take it.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_last_card_player_starts_empty() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let last = drawn.pop().expect("drained deck still has one card");
    deck.discard = vec![last];

    assert_eq!(deck.draw(), TurnDraw::LastCard(last));
    let current = Player::new(1, 0);
    assert_eq!(current.points, 0);
    assert!(!current.is_on_board);
    assert!(current.hand.is_empty());
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → last two → Player::new.
/// The split cards stay out of both new hands.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_last_two_players_start_empty() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let second = drawn.pop().expect("drained deck still has a card");
    let first = drawn.pop().expect("drained deck still has a second card");
    deck.discard = vec![first, second];

    let TurnDraw::LastTwo {
        current: current_card,
        next: next_card,
    } = deck.draw()
    else {
        panic!("the last two cards are shuffled and split");
    };
    let current = Player::new(1, 0);
    let next = Player::new(2, 1);
    assert!(current.hand.is_empty());
    assert!(next.hand.is_empty());
    assert_ne!(current_card.id, next_card.id);
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
}

fn assert_fresh(player: &Player, id: u32, seat: u32) {
    assert_eq!(player.id, id);
    assert_eq!(player.seat_index, seat);
    assert_eq!(player.points, 0);
    assert!(!player.is_on_board);
    assert!(player.hand.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → Player::new.
/// Building a player leaves the 108-card deck and its 12 wilds alone.
#[test]
fn test_suit_rank_card_deck_new_is_wild_player_new() {
    let deck = Deck::new();
    assert_eq!(deck.cards.len(), deck_size());
    assert_wilds(&deck.cards);
    assert_eq!(
        deck.cards
            .iter()
            .filter(|card| card.rank == Rank::Joker)
            .count(),
        DECKS * JOKERS_PER_DECK
    );

    let player = Player::new(4, 3);
    assert_fresh(&player, 4, 3);
    assert_eq!(deck.cards.len(), deck_size());
    assert_wilds(&deck.cards);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty → Player::new.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_player_starts_empty() {
    let mut deck = shuffled_deck();
    let _drawn = drain_without_discard(&mut deck);
    assert_eq!(deck.draw(), TurnDraw::Empty);

    let player = Player::new(1, 0);
    assert_fresh(&player, 1, 0);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → draw → empty reshuffle → Player::new.
/// The recycled card and the discard top stay out of the new hand.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_empty_reshuffle_player_starts_empty() {
    let mut deck = shuffled_deck();
    let mut held = Vec::new();
    while deck.cards.len() > 1 {
        held.push(ordinary(deck.draw()));
    }
    let last = deck.cards[0];
    let top = held.pop().expect("held cards supply a discard top");
    let mid = held.pop().expect("held cards supply a middle discard");
    let under = held.pop().expect("held cards supply an under discard");
    deck.discard = vec![under, mid, top];

    let drawn = ordinary(deck.draw());
    assert_eq!(drawn, last);
    let recycled = ordinary(deck.draw());
    assert_ne!(recycled.id, top.id);
    assert!(recycled.id == under.id || recycled.id == mid.id);
    assert_eq!(
        recycled.is_wild(),
        recycled.rank == Rank::Joker || recycled.rank == Rank::Two
    );

    let player = Player::new(1, 0);
    assert_fresh(&player, 1, 0);
    assert!(player
        .hand
        .iter()
        .all(|card| card.id != recycled.id && card.id != top.id));
    assert_eq!(deck.discard.as_slice(), &[top]);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one draw, discard stays → Player::new.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_nonempty_discard_player_starts_empty() {
    let mut deck = shuffled_deck();
    let mut pile = Vec::new();
    for _ in 0..5 {
        pile.push(ordinary(deck.draw()));
    }
    deck.discard = pile;
    let discard_before = deck.discard.clone();
    let drawn = ordinary(deck.draw());

    let player = Player::new(3, 2);
    assert_fresh(&player, 3, 2);
    assert!(player.hand.iter().all(|card| card.id != drawn.id));
    assert_eq!(deck.discard, discard_before);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one draw-pile card, then the last discard → Player::new.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_one_then_last_player_starts_empty() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let discard_card = drawn.pop().expect("drained deck still has a discard card");
    let draw_card = drawn.pop().expect("drained deck still has a draw card");
    deck.cards = vec![draw_card];
    deck.discard = vec![discard_card];

    assert_eq!(deck.draw(), TurnDraw::One(draw_card));
    assert_eq!(deck.draw(), TurnDraw::LastCard(discard_card));

    let player = Player::new(1, 0);
    assert_fresh(&player, 1, 0);
    assert!(player
        .hand
        .iter()
        .all(|card| card.id != draw_card.id && card.id != discard_card.id));
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → two ordinary draw-pile cards → Player::new.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_draw_two_pile_cards_player_starts_empty() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let second = drawn.pop().expect("drained deck still has a card");
    let first = drawn.pop().expect("drained deck still has a second card");
    deck.cards = vec![first, second];

    assert_eq!(deck.draw(), TurnDraw::One(second));
    assert_eq!(deck.draw(), TurnDraw::One(first));

    let current = Player::new(1, 0);
    let next = Player::new(2, 1);
    assert_fresh(&current, 1, 0);
    assert_fresh(&next, 2, 1);
    assert!(current
        .hand
        .iter()
        .all(|card| card.id != first.id && card.id != second.id));
    assert!(next
        .hand
        .iter()
        .all(|card| card.id != first.id && card.id != second.id));
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

fn top_cards(cards: &[Card], count: usize) -> Vec<Card> {
    cards.iter().rev().take(count).copied().collect()
}

fn pop_card(drawn: &mut Vec<Card>) -> Card {
    drawn.pop().expect("drained deck still has a card")
}

/// Cards taken from the drained pile, with the first taken card on top.
fn pile_from_end(drawn: &mut Vec<Card>, count: usize) -> Vec<Card> {
    let mut cards = Vec::with_capacity(count);
    for _ in 0..count {
        cards.push(pop_card(drawn));
    }
    cards.reverse();
    cards
}

fn assert_hand_kept(player: &Player, hand: &[Card]) {
    assert_eq!(player.points, 0);
    assert!(!player.is_on_board);
    assert_eq!(player.hand.as_slice(), hand);
}

fn hands_around(tops: &[Card], player_count: usize) -> Vec<Vec<Card>> {
    let mut hands = vec![Vec::new(); player_count];
    for (index, card) in tops.iter().copied().enumerate() {
        hands[index % player_count].push(card);
    }
    hands
}

fn deal_table(deck: &mut Deck, player_count: usize) -> Vec<Player> {
    let dealt = 10 * player_count;
    let tops = top_cards(&deck.cards, dealt);
    let expect = hands_around(&tops, player_count);
    let mut players: Vec<Player> = (0..player_count)
        .map(|seat| {
            let player = Player::new((seat + 1) as u32, seat as u32);
            assert_fresh(&player, player.id, player.seat_index);
            player
        })
        .collect();
    deal_initial_hands(&mut players, deck);
    for (player, hand) in players.iter().zip(&expect) {
        assert_hand_kept(player, hand);
    }
    players
}

fn hands_of(players: &[Player]) -> Vec<Vec<Card>> {
    players.iter().map(|player| player.hand.clone()).collect()
}

fn assert_hands_kept(players: &[Player], hands: &[Vec<Card>]) {
    for (player, hand) in players.iter().zip(hands) {
        assert_hand_kept(player, hand);
    }
}

fn none_hold(players: &[Player], id: u32) {
    assert!(players
        .iter()
        .all(|player| player.hand.iter().all(|card| card.id != id)));
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal one card at a time to two players.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_initial_hand() {
    let mut deck = shuffled_deck();
    let before = ids_of(&deck.cards);
    let players = deal_table(&mut deck, 2);

    assert_eq!(deck.cards.len(), deck_size() - 20);
    assert!(deck.discard.is_empty());
    let mut left = ids_of(&deck.cards);
    for player in &players {
        left.extend(player.hand.iter().map(|card| card.id));
        for card in &player.hand {
            assert_eq!(
                card.is_wild(),
                card.rank == Rank::Joker || card.rank == Rank::Two
            );
        }
    }
    same_ids(&before, &left);
    assert_eq!(
        players
            .iter()
            .flat_map(|player| player.hand.iter())
            .chain(deck.cards.iter())
            .filter(|card| card.is_wild())
            .count(),
        wild_count()
    );
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one card at a time around three players.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands() {
    let mut deck = shuffled_deck();
    let tops = top_cards(&deck.cards, 30);
    let players = deal_table(&mut deck, 3);

    assert_eq!(players.len(), 3);
    assert!(players.iter().all(|player| player.hand.len() == 10));
    assert_eq!(players[0].hand[0], tops[0]);
    assert_eq!(players[1].hand[0], tops[1]);
    assert_eq!(players[2].hand[0], tops[2]);
    assert_eq!(players[0].hand[1], tops[3]);
    assert_eq!(deck.cards.len(), deck_size() - 30);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → ordinary draw.
/// The draw stays out of every dealt hand.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let next_top = top_cards(&deck.cards, 1)[0];

    let drawn = ordinary(deck.draw());

    assert_eq!(drawn, next_top);
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays, then an ordinary draw.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_then_draw() {
    let mut deck = shuffled_deck();
    let discard: Vec<Card> = deck.cards.drain(0..4).collect();
    deck.discard = discard.clone();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert_eq!(deck.discard, discard);
    for card in &discard {
        none_hold(&players, card.id);
    }
    let drawn = ordinary(deck.draw());
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    assert_eq!(deck.discard, discard);
    assert_eq!(deck.cards.len(), deck_size() - 4 - 20 - 1);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal the last 20 around → empty reshuffle.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_empty_reshuffle() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let top = pop_card(&mut drawn);
    let mid = pop_card(&mut drawn);
    let under = pop_card(&mut drawn);
    deck.cards = pile_from_end(&mut drawn, 20);
    deck.discard = vec![under, mid, top];

    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert!(deck.cards.is_empty());
    assert_eq!(deck.discard.as_slice(), &[under, mid, top]);
    let recycled = ordinary(deck.draw());
    assert_ne!(recycled.id, top.id);
    assert!(recycled.id == under.id || recycled.id == mid.id);
    assert_hands_kept(&players, &hands);
    none_hold(&players, recycled.id);
    assert_eq!(deck.discard.as_slice(), &[top]);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal the last 20 around → last discard card.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_last_card() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let last = pop_card(&mut drawn);
    deck.cards = pile_from_end(&mut drawn, 20);
    deck.discard = vec![last];

    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert!(deck.cards.is_empty());
    assert_eq!(deck.discard.as_slice(), &[last]);
    assert_eq!(deck.draw(), TurnDraw::LastCard(last));
    assert_hands_kept(&players, &hands);
    none_hold(&players, last.id);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal the last 20 around → last two split.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_last_two() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let second = pop_card(&mut drawn);
    let first = pop_card(&mut drawn);
    deck.cards = pile_from_end(&mut drawn, 20);
    deck.discard = vec![first, second];

    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert!(deck.cards.is_empty());
    assert_eq!(deck.discard.as_slice(), &[first, second]);
    let TurnDraw::LastTwo { current, next } = deck.draw() else {
        panic!("the last two cards are shuffled and split");
    };
    let mut got = [current.id, next.id];
    got.sort_unstable();
    let mut expect = [first.id, second.id];
    expect.sort_unstable();
    assert_eq!(got, expect);
    assert_hands_kept(&players, &hands);
    none_hold(&players, current.id);
    none_hold(&players, next.id);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → one draw-pile card, then the last discard.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_one_then_last() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let discard_card = pop_card(&mut drawn);
    let pile = pile_from_end(&mut drawn, 21);
    let leftover = pile[0];
    deck.cards = pile;
    deck.discard = vec![discard_card];

    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert_eq!(deck.cards.as_slice(), &[leftover]);
    assert_eq!(deck.discard.as_slice(), &[discard_card]);
    assert_eq!(deck.draw(), TurnDraw::One(leftover));
    assert_eq!(deck.draw(), TurnDraw::LastCard(discard_card));
    assert_hands_kept(&players, &hands);
    none_hold(&players, leftover.id);
    none_hold(&players, discard_card.id);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → two ordinary draw-pile cards.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_two_pile_cards() {
    let mut deck = shuffled_deck();
    let mut drawn = drain_without_discard(&mut deck);
    let pile = pile_from_end(&mut drawn, 22);
    let first = pile[0];
    let second = pile[1];
    deck.cards = pile;

    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);

    assert_eq!(deck.cards.as_slice(), &[first, second]);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.draw(), TurnDraw::One(second));
    assert_eq!(deck.draw(), TurnDraw::One(first));
    assert_hands_kept(&players, &hands);
    none_hold(&players, first.id);
    none_hold(&players, second.id);
    assert_eq!(deck.draw(), TurnDraw::Empty);
}

const PIP_COUNT: usize = 7 * SUITS_PER_DECK * DECKS;

fn is_pip(rank: Rank) -> bool {
    matches!(
        rank,
        Rank::Three | Rank::Four | Rank::Five | Rank::Six | Rank::Seven | Rank::Eight | Rank::Nine
    )
}

fn take_rank(cards: &mut Vec<Card>, rank: Rank) -> Card {
    let index = cards
        .iter()
        .position(|card| card.rank == rank)
        .expect("the deck contains this rank");
    cards.remove(index)
}

fn cards_in_play(players: &[Player], deck: &Deck) -> Vec<Card> {
    let mut cards = Vec::new();
    for player in players {
        cards.extend(player.hand.iter().copied());
    }
    cards.extend(deck.cards.iter().copied());
    cards.extend(deck.discard.iter().copied());
    cards
}

fn take_suited(cards: &mut Vec<Card>, suit: Suit, rank: Rank) -> Card {
    let index = cards
        .iter()
        .position(|card| card.suit == suit && card.rank == rank)
        .expect("the shoe contains this suit and rank");
    cards.remove(index)
}

/// Next draws come off the end, so the last pushed card is dealt first.
fn stack_next_draws(cards: &mut Vec<Card>, ranks: &[Rank]) {
    let stacked: Vec<Card> = ranks
        .iter()
        .rev()
        .map(|rank| take_rank(cards, *rank))
        .collect();
    cards.extend(stacked);
}

fn count_pip_penalty(cards: &[Card]) -> usize {
    let pips: Vec<&Card> = cards.iter().filter(|card| is_pip(card.rank)).collect();
    for card in &pips {
        assert_eq!(card.get_penalty_value(), 5);
    }
    pips.len()
}

fn pip_scores_in_play(players: &[Player], deck: &Deck) -> usize {
    let mut scored = 0;
    for player in players {
        assert_eq!(player.points, 0);
        scored += count_pip_penalty(&player.hand);
    }
    scored += count_pip_penalty(&deck.cards);
    scored += count_pip_penalty(&deck.discard);
    scored
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal one card at a time → pip penalty.
/// The first three cards dealt are a 3, a 5, and a 9. Each scores 5. Every other rank 3–9 scores 5.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_pip_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(&mut deck.cards, &[Rank::Three, Rank::Five, Rank::Nine]);
    let players = deal_table(&mut deck, 2);

    assert_eq!(players[0].hand[0].rank, Rank::Three);
    assert_eq!(players[0].hand[0].get_penalty_value(), 5);
    assert_eq!(players[1].hand[0].rank, Rank::Five);
    assert_eq!(players[1].hand[0].get_penalty_value(), 5);
    assert_eq!(players[0].hand[1].rank, Rank::Nine);
    assert_eq!(players[0].hand[1].get_penalty_value(), 5);
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one card at a time around three players → pip penalty.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_pip_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(&mut deck.cards, &[Rank::Three, Rank::Five, Rank::Nine]);
    let players = deal_table(&mut deck, 3);

    assert_eq!(players[0].hand[0].rank, Rank::Three);
    assert_eq!(players[1].hand[0].rank, Rank::Five);
    assert_eq!(players[2].hand[0].rank, Rank::Nine);
    for player in &players {
        assert_eq!(player.hand[0].get_penalty_value(), 5);
        assert_eq!(player.hand.len(), 10);
        assert_eq!(player.points, 0);
    }
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 30);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → ordinary draw → pip penalty.
/// Scoring the dealt cards leaves the hands and the draw pile where the deal and draw put them.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_pip_penalty() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let next_top = top_cards(&deck.cards, 1)[0];

    let drawn = ordinary(deck.draw());

    assert_eq!(drawn, next_top);
    if is_pip(drawn.rank) {
        assert_eq!(drawn.get_penalty_value(), 5);
    }
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    let scored = pip_scores_in_play(&players, &deck);
    let drawn_pips = usize::from(is_pip(drawn.rank));
    assert_eq!(scored + drawn_pips, PIP_COUNT);
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → pip penalty.
/// A pip left on the discard pile still scores 5 and stays there.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_pip_penalty() {
    let mut deck = shuffled_deck();
    let pip = take_rank(&mut deck.cards, Rank::Three);
    let mut discard: Vec<Card> = deck.cards.drain(0..3).collect();
    discard.push(pip);
    deck.discard = discard.clone();
    let players = deal_table(&mut deck, 2);

    assert_eq!(deck.discard, discard);
    assert_eq!(pip.get_penalty_value(), 5);
    for card in &discard {
        none_hold(&players, card.id);
    }
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(deck.cards.len(), deck_size() - 4 - 20);
}

const FACE_COUNT: usize = 4 * SUITS_PER_DECK * DECKS;
const ACE_COUNT: usize = SUITS_PER_DECK * DECKS;

fn is_face(rank: Rank) -> bool {
    matches!(rank, Rank::Ten | Rank::Jack | Rank::Queen | Rank::King)
}

fn count_face_penalty(cards: &[Card]) -> usize {
    let faces: Vec<&Card> = cards.iter().filter(|card| is_face(card.rank)).collect();
    for card in &faces {
        assert_eq!(card.get_penalty_value(), 10);
    }
    faces.len()
}

fn count_ace_penalty(cards: &[Card]) -> usize {
    let aces: Vec<&Card> = cards.iter().filter(|card| card.rank == Rank::Ace).collect();
    for card in &aces {
        assert_eq!(card.get_penalty_value(), 15);
    }
    aces.len()
}

fn face_ace_scores_in_play(players: &[Player], deck: &Deck) -> (usize, usize) {
    let mut faces = 0;
    let mut aces = 0;
    for player in players {
        assert_eq!(player.points, 0);
        faces += count_face_penalty(&player.hand);
        aces += count_ace_penalty(&player.hand);
    }
    faces += count_face_penalty(&deck.cards);
    aces += count_ace_penalty(&deck.cards);
    faces += count_face_penalty(&deck.discard);
    aces += count_ace_penalty(&deck.discard);
    (faces, aces)
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal one card at a time → face and ace penalty.
/// The first five cards dealt are a 10, Jack, Queen, King, and Ace. Faces score 10. The ace scores 15.
/// Every other 10 through King scores 10, and every other ace scores 15. Pips still score 5.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_face_ace_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(
        &mut deck.cards,
        &[Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace],
    );
    let players = deal_table(&mut deck, 2);

    assert_eq!(players[0].hand[0].rank, Rank::Ten);
    assert_eq!(players[0].hand[0].get_penalty_value(), 10);
    assert_eq!(players[1].hand[0].rank, Rank::Jack);
    assert_eq!(players[1].hand[0].get_penalty_value(), 10);
    assert_eq!(players[0].hand[1].rank, Rank::Queen);
    assert_eq!(players[0].hand[1].get_penalty_value(), 10);
    assert_eq!(players[1].hand[1].rank, Rank::King);
    assert_eq!(players[1].hand[1].get_penalty_value(), 10);
    assert_eq!(players[0].hand[2].rank, Rank::Ace);
    assert_eq!(players[0].hand[2].get_penalty_value(), 15);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one card at a time around three players → face and ace penalty.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_face_ace_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(
        &mut deck.cards,
        &[Rank::Ten, Rank::Jack, Rank::Queen, Rank::King, Rank::Ace],
    );
    let players = deal_table(&mut deck, 3);

    assert_eq!(players[0].hand[0].rank, Rank::Ten);
    assert_eq!(players[1].hand[0].rank, Rank::Jack);
    assert_eq!(players[2].hand[0].rank, Rank::Queen);
    assert_eq!(players[0].hand[1].rank, Rank::King);
    assert_eq!(players[1].hand[1].rank, Rank::Ace);
    assert_eq!(players[0].hand[0].get_penalty_value(), 10);
    assert_eq!(players[1].hand[0].get_penalty_value(), 10);
    assert_eq!(players[2].hand[0].get_penalty_value(), 10);
    assert_eq!(players[0].hand[1].get_penalty_value(), 10);
    assert_eq!(players[1].hand[1].get_penalty_value(), 15);
    for player in &players {
        assert_eq!(player.hand.len(), 10);
        assert_eq!(player.points, 0);
    }
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 30);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → ordinary draw → face and ace penalty.
/// Scoring the dealt cards leaves the hands and the draw pile where the deal and draw put them.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_face_ace_penalty() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let next_top = top_cards(&deck.cards, 1)[0];

    let drawn = ordinary(deck.draw());

    assert_eq!(drawn, next_top);
    if is_face(drawn.rank) {
        assert_eq!(drawn.get_penalty_value(), 10);
    } else if drawn.rank == Rank::Ace {
        assert_eq!(drawn.get_penalty_value(), 15);
    } else if is_pip(drawn.rank) {
        assert_eq!(drawn.get_penalty_value(), 5);
    }
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    let (faces, aces) = face_ace_scores_in_play(&players, &deck);
    let drawn_faces = usize::from(is_face(drawn.rank));
    let drawn_aces = usize::from(drawn.rank == Rank::Ace);
    assert_eq!(faces + drawn_faces, FACE_COUNT);
    assert_eq!(aces + drawn_aces, ACE_COUNT);
    let scored = pip_scores_in_play(&players, &deck);
    let drawn_pips = usize::from(is_pip(drawn.rank));
    assert_eq!(scored + drawn_pips, PIP_COUNT);
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → face and ace penalty.
/// A king and an ace left on the discard pile still score and stay there.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_face_ace_penalty() {
    let mut deck = shuffled_deck();
    let king = take_rank(&mut deck.cards, Rank::King);
    let ace = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard: Vec<Card> = deck.cards.drain(0..3).collect();
    discard.push(king);
    discard.push(ace);
    deck.discard = discard.clone();
    let players = deal_table(&mut deck, 2);

    assert_eq!(deck.discard, discard);
    assert_eq!(king.get_penalty_value(), 10);
    assert_eq!(ace.get_penalty_value(), 15);
    for card in &discard {
        none_hold(&players, card.id);
    }
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 5 - 20);
}

const TWO_COUNT: usize = SUITS_PER_DECK * DECKS;
const JOKER_COUNT: usize = JOKERS_PER_DECK * DECKS;

fn count_wild_penalty(cards: &[Card]) -> (usize, usize) {
    let mut twos = 0;
    let mut jokers = 0;
    for card in cards {
        match card.rank {
            Rank::Two => {
                assert_eq!(card.get_penalty_value(), 20);
                assert!(card.is_wild());
                twos += 1;
            }
            Rank::Joker => {
                assert_eq!(card.get_penalty_value(), 20);
                assert!(card.is_wild());
                jokers += 1;
            }
            _ => {}
        }
    }
    (twos, jokers)
}

fn wild_scores_in_play(players: &[Player], deck: &Deck) -> (usize, usize) {
    let mut twos = 0;
    let mut jokers = 0;
    for player in players {
        assert_eq!(player.points, 0);
        let (hand_twos, hand_jokers) = count_wild_penalty(&player.hand);
        twos += hand_twos;
        jokers += hand_jokers;
    }
    let (draw_twos, draw_jokers) = count_wild_penalty(&deck.cards);
    let (discard_twos, discard_jokers) = count_wild_penalty(&deck.discard);
    (
        twos + draw_twos + discard_twos,
        jokers + draw_jokers + discard_jokers,
    )
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal one card at a time → wild penalty.
/// The first two cards dealt are a two and a joker. Each scores 20. Every other two and joker scores 20.
/// Pips still score 5. A 10 through King still scores 10. An ace still scores 15.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_wild_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(&mut deck.cards, &[Rank::Two, Rank::Joker]);
    let players = deal_table(&mut deck, 2);

    assert_eq!(players[0].hand[0].rank, Rank::Two);
    assert_eq!(players[0].hand[0].get_penalty_value(), 20);
    assert!(players[0].hand[0].is_wild());
    assert_eq!(players[1].hand[0].rank, Rank::Joker);
    assert_eq!(players[1].hand[0].suit, Suit::None);
    assert_eq!(players[1].hand[0].get_penalty_value(), 20);
    assert!(players[1].hand[0].is_wild());
    assert_eq!(
        wild_scores_in_play(&players, &deck),
        (TWO_COUNT, JOKER_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one card at a time around three players → wild penalty.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_wild_penalty() {
    let mut deck = shuffled_deck();
    stack_next_draws(&mut deck.cards, &[Rank::Two, Rank::Joker, Rank::Two]);
    let players = deal_table(&mut deck, 3);

    assert_eq!(players[0].hand[0].rank, Rank::Two);
    assert_eq!(players[1].hand[0].rank, Rank::Joker);
    assert_eq!(players[1].hand[0].suit, Suit::None);
    assert_eq!(players[2].hand[0].rank, Rank::Two);
    assert_ne!(players[0].hand[0].id, players[2].hand[0].id);
    assert_eq!(players[0].hand[0].get_penalty_value(), 20);
    assert_eq!(players[1].hand[0].get_penalty_value(), 20);
    assert_eq!(players[2].hand[0].get_penalty_value(), 20);
    for player in &players {
        assert_eq!(player.hand.len(), 10);
        assert_eq!(player.points, 0);
    }
    assert_eq!(
        wild_scores_in_play(&players, &deck),
        (TWO_COUNT, JOKER_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 30);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → draw a two → wild penalty.
/// Scoring the drawn two leaves the hands where the deal put them and does not change player points.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_two_penalty() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let two = take_rank(&mut deck.cards, Rank::Two);
    deck.cards.push(two);

    let drawn = ordinary(deck.draw());

    assert_eq!(drawn.id, two.id);
    assert_eq!(drawn.rank, Rank::Two);
    assert_eq!(drawn.get_penalty_value(), 20);
    assert_eq!(drawn.get_penalty_value(), 20);
    assert!(drawn.is_wild());
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    let (twos, jokers) = wild_scores_in_play(&players, &deck);
    assert_eq!(twos + 1, TWO_COUNT);
    assert_eq!(jokers, JOKER_COUNT);
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → draw a joker → wild penalty.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_joker_penalty() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let joker = take_rank(&mut deck.cards, Rank::Joker);
    deck.cards.push(joker);

    let drawn = ordinary(deck.draw());

    assert_eq!(drawn.id, joker.id);
    assert_eq!(drawn.rank, Rank::Joker);
    assert_eq!(drawn.suit, Suit::None);
    assert_eq!(drawn.get_penalty_value(), 20);
    assert!(drawn.is_wild());
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    let (twos, jokers) = wild_scores_in_play(&players, &deck);
    assert_eq!(twos, TWO_COUNT);
    assert_eq!(jokers + 1, JOKER_COUNT);
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → wild penalty.
/// A two and a joker left on the discard pile still score 20 and stay there.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_wild_penalty() {
    let mut deck = shuffled_deck();
    let two = take_rank(&mut deck.cards, Rank::Two);
    let joker = take_rank(&mut deck.cards, Rank::Joker);
    let mut discard: Vec<Card> = deck.cards.drain(0..3).collect();
    discard.push(two);
    discard.push(joker);
    deck.discard = discard.clone();
    let players = deal_table(&mut deck, 2);

    assert_eq!(deck.discard, discard);
    assert_eq!(two.get_penalty_value(), 20);
    assert_eq!(joker.get_penalty_value(), 20);
    assert_eq!(joker.suit, Suit::None);
    for card in &discard {
        none_hold(&players, card.id);
    }
    assert_eq!(
        wild_scores_in_play(&players, &deck),
        (TWO_COUNT, JOKER_COUNT)
    );
    assert_eq!(pip_scores_in_play(&players, &deck), PIP_COUNT);
    assert_eq!(
        face_ace_scores_in_play(&players, &deck),
        (FACE_COUNT, ACE_COUNT)
    );
    for player in &players {
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 5 - 20);
}

fn penalty_sum(cards: &[Card]) -> u32 {
    cards.iter().map(|card| card.get_penalty_value()).sum()
}

fn assert_dealt_hand_totals(players: &[Player]) {
    for player in players {
        let total = player.calculate_hand_penalty();
        assert_eq!(total, penalty_sum(&player.hand));
        assert_eq!(player.calculate_hand_penalty(), total);
        assert_eq!(player.points, 0);
        assert_eq!(player.total_score, 0);
        assert_eq!(player.hand.len(), 10);
    }
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal one card at a time → hand total.
/// Player 0's first four cards are a 4, Jack, Ace, and Joker (50). The hand total is that hand only.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_hand_total() {
    let mut deck = shuffled_deck();
    stack_next_draws(
        &mut deck.cards,
        &[
            Rank::Four,
            Rank::Three,
            Rank::Jack,
            Rank::Five,
            Rank::Ace,
            Rank::Six,
            Rank::Joker,
            Rank::Seven,
        ],
    );
    let players = deal_table(&mut deck, 2);

    assert_eq!(players[0].hand[0].rank, Rank::Four);
    assert_eq!(players[0].hand[1].rank, Rank::Jack);
    assert_eq!(players[0].hand[2].rank, Rank::Ace);
    assert_eq!(players[0].hand[3].rank, Rank::Joker);
    assert_eq!(players[0].hand[3].suit, Suit::None);
    assert_eq!(penalty_sum(&players[0].hand[..4]), 50);
    assert_dealt_hand_totals(&players);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → one card at a time around three players → hand total.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_three_hands_hand_total() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 3);

    assert_dealt_hand_totals(&players);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 30);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → draw → hand total.
/// The drawn card stays out of both hands, so each hand total stays the dealt sum.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_then_draw_hand_total() {
    let mut deck = shuffled_deck();
    let players = deal_table(&mut deck, 2);
    let before: Vec<u32> = players
        .iter()
        .map(|player| player.calculate_hand_penalty())
        .collect();
    let hands = hands_of(&players);

    let drawn = ordinary(deck.draw());

    assert_eq!(
        players
            .iter()
            .map(|player| player.calculate_hand_penalty())
            .collect::<Vec<_>>(),
        before
    );
    assert_hands_kept(&players, &hands);
    none_hold(&players, drawn.id);
    for player in &players {
        assert_eq!(player.total_score, 0);
        assert_eq!(player.points, 0);
    }
    assert_eq!(deck.cards.len(), deck_size() - 21);
    assert!(deck.discard.is_empty());
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → hand total.
/// Cards left on the discard pile are not part of either hand total.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_hand_total() {
    let mut deck = shuffled_deck();
    let two = take_rank(&mut deck.cards, Rank::Two);
    let joker = take_rank(&mut deck.cards, Rank::Joker);
    let mut discard: Vec<Card> = deck.cards.drain(0..3).collect();
    discard.push(two);
    discard.push(joker);
    deck.discard = discard.clone();
    let players = deal_table(&mut deck, 2);

    assert_eq!(deck.discard, discard);
    assert_eq!(two.get_penalty_value(), 20);
    assert_eq!(joker.get_penalty_value(), 20);
    assert_dealt_hand_totals(&players);
    for card in &discard {
        none_hold(&players, card.id);
    }
    assert_eq!(deck.cards.len(), deck_size() - 5 - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around → add the hand penalty to the total score.
/// Only the player who records the hand moves off 0. The hand and the other seat stay put.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_add_to_total_score() {
    let mut deck = shuffled_deck();
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let penalty = players[0].calculate_hand_penalty();
    assert_eq!(penalty, penalty_sum(&players[0].hand));
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);

    players[0].add_hand_penalty_to_total();

    assert_eq!(players[0].total_score, penalty);
    assert_eq!(players[0].points, 0);
    assert_eq!(
        players[1].calculate_hand_penalty(),
        penalty_sum(&players[1].hand)
    );
    assert_eq!(players[1].total_score, 0);
    assert_hands_kept(&players, &hands);
    assert!(deck.discard.is_empty());
    assert_eq!(deck.cards.len(), deck_size() - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → take the top discard.
/// The top card joins player 0's hand. Cards under it stay. The other hand stays the dealt 10.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_take_top() {
    let mut deck = shuffled_deck();
    let under: Vec<Card> = deck.cards.drain(0..3).collect();
    let top = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard = under.clone();
    discard.push(top);
    deck.discard = discard;
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let before_penalty = players[0].calculate_hand_penalty();
    let other_penalty = players[1].calculate_hand_penalty();
    let draw_len = deck.cards.len();

    Action::TakeDiscard.apply(&mut players, 0, &mut deck);

    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(&players[0].hand[..10], hands[0].as_slice());
    assert_eq!(players[0].hand[10], top);
    assert_eq!(deck.discard, under);
    assert_eq!(players[1].hand, hands[1]);
    assert_eq!(
        players[0].calculate_hand_penalty(),
        before_penalty + top.get_penalty_value()
    );
    assert_eq!(players[1].calculate_hand_penalty(), other_penalty);
    assert_eq!(top.get_penalty_value(), 15);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[1].total_score, 0);
    assert!(!players[0].is_on_board);
    assert!(!players[1].is_on_board);
    assert_eq!(deck.cards.len(), draw_len);
    assert_eq!(deck.cards.len(), deck_size() - 4 - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → push the top discard.
/// Player 1 pushes the top ace to Player 2. Player 2 also draws a four. Player 1 then draws a jack.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top() {
    let mut deck = shuffled_deck();
    let penalty = take_rank(&mut deck.cards, Rank::Four);
    let actor_draw = take_rank(&mut deck.cards, Rank::Jack);
    let under: Vec<Card> = deck.cards.drain(0..3).collect();
    let top = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard = under.clone();
    discard.push(top);
    deck.discard = discard;
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let before = players[0].calculate_hand_penalty();
    let other_before = players[1].calculate_hand_penalty();
    deck.cards.push(actor_draw);
    deck.cards.push(penalty);
    let draw_len = deck.cards.len();

    Action::PushDiscard.apply(&mut players, 0, &mut deck);

    assert_eq!(players[0].id, 1);
    assert_eq!(players[1].id, 2);
    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(&players[0].hand[..10], hands[0].as_slice());
    assert_eq!(players[0].hand[10], actor_draw);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(&players[1].hand[..10], hands[1].as_slice());
    assert_eq!(players[1].hand[10], top);
    assert_eq!(players[1].hand[11], penalty);
    assert_eq!(deck.discard, under);
    assert_eq!(top.get_penalty_value(), 15);
    assert_eq!(penalty.get_penalty_value(), 5);
    assert_eq!(actor_draw.get_penalty_value(), 10);
    assert_eq!(players[0].calculate_hand_penalty(), before + 10);
    assert_eq!(players[1].calculate_hand_penalty(), other_before + 15 + 5);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);
    assert!(!players[0].is_on_board);
    assert!(!players[1].is_on_board);
    assert_eq!(deck.cards.len(), draw_len - 2);
    assert_eq!(deck.cards.len(), deck_size() - 2 - 3 - 1 - 20);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → take the top, then push the new top.
/// The take still adds the ace to Player 1. The push then moves the uncovered card to Player 2 with a penalty draw.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_take_top_then_push() {
    let mut deck = shuffled_deck();
    let penalty = take_rank(&mut deck.cards, Rank::Three);
    let actor_draw = take_rank(&mut deck.cards, Rank::King);
    let buried: Vec<Card> = deck.cards.drain(0..2).collect();
    let pushed = take_rank(&mut deck.cards, Rank::Five);
    let taken = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard = buried.clone();
    discard.push(pushed);
    discard.push(taken);
    deck.discard = discard;
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    deck.cards.push(actor_draw);
    deck.cards.push(penalty);

    Action::TakeDiscard.apply(&mut players, 0, &mut deck);
    Action::PushDiscard.apply(&mut players, 0, &mut deck);

    assert_eq!(players[0].hand.len(), 12);
    assert_eq!(&players[0].hand[..10], hands[0].as_slice());
    assert_eq!(players[0].hand[10], taken);
    assert_eq!(players[0].hand[11], actor_draw);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(&players[1].hand[..10], hands[1].as_slice());
    assert_eq!(players[1].hand[10], pushed);
    assert_eq!(players[1].hand[11], penalty);
    assert_eq!(deck.discard, buried);
    assert_eq!(taken.get_penalty_value(), 15);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → push the top discard → validate a set.
/// The pushed ace, the penalty four, and Player 1's jack are not a set. Three fours from that shoe are. A four, a joker, and a two are.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top_validate_set() {
    let mut deck = shuffled_deck();
    let penalty = take_rank(&mut deck.cards, Rank::Four);
    let actor_draw = take_rank(&mut deck.cards, Rank::Jack);
    let under: Vec<Card> = deck.cards.drain(0..3).collect();
    let top = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard = under.clone();
    discard.push(top);
    deck.discard = discard;
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let before = players[0].calculate_hand_penalty();
    let other_before = players[1].calculate_hand_penalty();
    deck.cards.push(actor_draw);
    deck.cards.push(penalty);
    let draw_len = deck.cards.len();

    Action::PushDiscard.apply(&mut players, 0, &mut deck);

    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(&players[0].hand[..10], hands[0].as_slice());
    assert_eq!(players[0].hand[10], actor_draw);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(&players[1].hand[..10], hands[1].as_slice());
    assert_eq!(players[1].hand[10], top);
    assert_eq!(players[1].hand[11], penalty);
    assert_eq!(deck.discard, under);
    assert_eq!(players[0].calculate_hand_penalty(), before + 10);
    assert_eq!(players[1].calculate_hand_penalty(), other_before + 15 + 5);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);
    assert_eq!(deck.cards.len(), draw_len - 2);

    let mut shoe = cards_in_play(&players, &deck);
    let four_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Four);
    let four_spades = take_suited(&mut shoe, Suit::Spades, Rank::Four);
    let four_clubs = take_suited(&mut shoe, Suit::Clubs, Rank::Four);
    let five_clubs = take_suited(&mut shoe, Suit::Clubs, Rank::Five);
    let two_spades = take_suited(&mut shoe, Suit::Spades, Rank::Two);
    let jokers: Vec<Card> = shoe
        .iter()
        .copied()
        .filter(|card| card.rank == Rank::Joker)
        .take(3)
        .collect();

    assert!(!validate_set(&[top, penalty, actor_draw]));
    assert!(validate_set(&[four_hearts, four_spades, four_clubs]));
    assert!(!validate_set(&[four_hearts, four_spades, five_clubs]));
    assert!(validate_set(&[four_hearts, jokers[0], two_spades]));
    assert!(validate_set(&jokers));
    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(deck.cards.len(), draw_len - 2);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal around, discard stays → push the top discard → validate a set → validate a run.
/// Three fours are a set. 4♥ 5♥ 6♥ 7♥ is a run. Mixed suits are not. A joker and a two fill 4♥ … 6♥. Ace-low and ace-high are runs. King, ace, a two, and a three is not.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_leaves_discard_push_top_validate_set_validate_run(
) {
    let mut deck = shuffled_deck();
    let penalty = take_rank(&mut deck.cards, Rank::Four);
    let actor_draw = take_rank(&mut deck.cards, Rank::Jack);
    let under: Vec<Card> = deck.cards.drain(0..3).collect();
    let top = take_rank(&mut deck.cards, Rank::Ace);
    let mut discard = under.clone();
    discard.push(top);
    deck.discard = discard;
    let mut players = deal_table(&mut deck, 2);
    let hands = hands_of(&players);
    let before = players[0].calculate_hand_penalty();
    let other_before = players[1].calculate_hand_penalty();
    deck.cards.push(actor_draw);
    deck.cards.push(penalty);
    let draw_len = deck.cards.len();

    Action::PushDiscard.apply(&mut players, 0, &mut deck);

    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(&players[0].hand[..10], hands[0].as_slice());
    assert_eq!(players[0].hand[10], actor_draw);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(&players[1].hand[..10], hands[1].as_slice());
    assert_eq!(players[1].hand[10], top);
    assert_eq!(players[1].hand[11], penalty);
    assert_eq!(deck.discard, under);
    assert_eq!(players[0].calculate_hand_penalty(), before + 10);
    assert_eq!(players[1].calculate_hand_penalty(), other_before + 15 + 5);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);
    assert_eq!(deck.cards.len(), draw_len - 2);

    let mut shoe = cards_in_play(&players, &deck);
    let four_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Four);
    let four_spades = take_suited(&mut shoe, Suit::Spades, Rank::Four);
    let four_clubs = take_suited(&mut shoe, Suit::Clubs, Rank::Four);
    let five_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Five);
    let six_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Six);
    let seven_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Seven);
    let seven_diamonds = take_suited(&mut shoe, Suit::Diamonds, Rank::Seven);
    let eight_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Eight);
    let nine_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Nine);
    let ten_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Ten);
    let three_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Three);
    let jack_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Jack);
    let queen_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Queen);
    let king_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::King);
    let ace_hearts = take_suited(&mut shoe, Suit::Hearts, Rank::Ace);
    let two_spades = take_suited(&mut shoe, Suit::Spades, Rank::Two);
    let two_clubs = take_suited(&mut shoe, Suit::Clubs, Rank::Two);
    let joker = shoe
        .iter()
        .copied()
        .find(|card| card.rank == Rank::Joker)
        .expect("the shoe contains a joker");

    assert!(validate_set(&[four_hearts, four_spades, four_clubs]));
    assert!(!validate_set(&[top, penalty, actor_draw]));
    assert!(validate_run(&[
        four_hearts,
        five_hearts,
        six_hearts,
        seven_hearts
    ]));
    assert!(!validate_run(&[
        four_hearts,
        five_hearts,
        six_hearts,
        seven_diamonds
    ]));
    assert!(!validate_run(&[eight_hearts, nine_hearts, ten_hearts]));
    assert!(validate_run(&[four_hearts, joker, six_hearts, two_spades]));
    assert!(validate_run(&[
        ace_hearts,
        two_clubs,
        three_hearts,
        four_hearts
    ]));
    assert!(validate_run(&[
        jack_hearts,
        queen_hearts,
        king_hearts,
        ace_hearts
    ]));
    assert!(!validate_run(&[
        king_hearts,
        ace_hearts,
        two_clubs,
        three_hearts
    ]));
    assert_eq!(players[0].hand.len(), 11);
    assert_eq!(players[1].hand.len(), 12);
    assert_eq!(deck.cards.len(), draw_len - 2);
}

/// Chain: Suit → Rank → Card → Deck::new → is_wild → shuffle → deal runs out of draw-pile cards.
/// The three cards that remain are dealt. The next card panics. The discard pile stays empty.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_short_draw_pile() {
    let mut deck = shuffled_deck();
    let pile = deck.cards.split_off(deck.cards.len() - 3);
    let before = ids_of(&pile);
    deck.cards = pile.clone();
    let mut players = vec![Player::new(1, 0), Player::new(2, 1)];

    let panicked = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        deal_initial_hands(&mut players, &mut deck);
    }));

    let message = panicked.expect_err("a short draw pile cannot finish a 10-card deal");
    let text = message
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| message.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        text.contains("initial deal takes a card from the draw pile"),
        "{text}"
    );
    assert_eq!(players[0].hand, vec![pile[2], pile[0]]);
    assert_eq!(players[1].hand, vec![pile[1]]);
    assert!(deck.cards.is_empty());
    assert!(deck.discard.is_empty());
    let mut dealt = ids_of(&players[0].hand);
    dealt.extend(ids_of(&players[1].hand));
    same_ids(&dealt, &before);
    assert_eq!(players[0].points, 0);
    assert_eq!(players[1].points, 0);
    assert_eq!(players[0].total_score, 0);
    assert_eq!(players[1].total_score, 0);
}
