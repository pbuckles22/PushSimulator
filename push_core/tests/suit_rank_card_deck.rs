//! Chain: Suit + Rank + Card, then Deck::new.
//! Two standard decks, two jokers in each deck.

use push_core::card::{Card, Rank, Suit};
use push_core::deck::{Deck, TurnDraw};
use push_core::player::Player;

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
