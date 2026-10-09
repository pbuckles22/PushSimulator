//! Two random bots play a headless game through the real engine.

use rand::rngs::StdRng;
use rand::SeedableRng;

use push_core::actions::Action;
use push_core::card::{Card, Rank, Suit};
use push_core::deck::{Deck, TurnDraw};
use push_core::game_state::{GameState, TurnPhase};
use push_core::legal_moves::generate_legal_moves;
use push_core::player::Player;
use push_core::random_bot::{new_two_seat_table, play_random_game, play_random_turn};

fn card(id: u32, suit: Suit, rank: Rank) -> Card {
    Card {
        id,
        suit,
        rank,
        locked_until_turn: 0,
    }
}

fn shoe_ids(state: &GameState) -> Vec<u32> {
    let mut ids = Vec::new();
    for player in &state.players {
        ids.extend(player.hand.iter().map(|card| card.id));
    }
    for meld in &state.board {
        ids.extend(meld.iter().map(|card| card.id));
    }
    ids.extend(state.deck.cards.iter().map(|card| card.id));
    ids.extend(state.deck.discard.iter().map(|card| card.id));
    ids.sort_unstable();
    ids
}

fn fresh_shoe_ids() -> Vec<u32> {
    let mut ids: Vec<u32> = Deck::new().cards.iter().map(|card| card.id).collect();
    ids.sort_unstable();
    ids
}

/// Two seats finish five rounds. The sixth round is dealt and not played.
/// `points` stay 0. Every card id is still on the table.
#[test]
fn test_headless_random_game() {
    let state = play_random_game(1);
    assert_eq!(state.players.len(), 2);
    assert_eq!(state.round_number, 6);
    assert!(!state.round_over);
    assert!(state.board.is_empty());
    assert_eq!(state.turn_counter, 0);
    assert_eq!(state.turn_phase, TurnPhase::Playing);
    assert_eq!(state.penalty_seat, None);
    assert_eq!(state.drawn_card_id, None);
    assert_eq!(state.deck.discard.len(), 1);
    assert_eq!(state.deck.cards.len(), 87);
    for player in &state.players {
        assert_eq!(player.hand.len(), 10);
        assert!(!player.is_on_board);
        assert_eq!(player.points, 0);
    }
    assert_eq!(state.players[0].total_score, 420);
    assert_eq!(state.players[1].total_score, 40);
    assert_eq!(shoe_ids(&state), fresh_shoe_ids());
}

/// The same seed deals the same sixth round and the same totals.
#[test]
fn test_headless_random_game_same_seed_repeats() {
    let first = play_random_game(1);
    let second = play_random_game(1);
    assert_eq!(first.players, second.players);
    assert_eq!(first.deck, second.deck);
    assert_eq!(first.round_number, second.round_number);
    assert_eq!(first.board, second.board);
    assert_eq!(first.turn_counter, second.turn_counter);
    assert_eq!(first.round_over, second.round_over);
}

/// Chain: Deck::new → shuffle → deal → generate legal moves → random turn → five rounds.
/// The shoe is still the two decks the table started with.
#[test]
fn test_suit_rank_card_deck_new_is_wild_shuffle_deal_generate_legal_moves_random_game() {
    let state = play_random_game(1);
    assert_eq!(state.round_number, 6);
    assert!(!state.round_over);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[1].points, 0);
    assert_eq!(shoe_ids(&state), fresh_shoe_ids());
}

/// A dealt shoe keeps every card through one random turn.
#[test]
fn test_random_turn_on_a_dealt_shoe_keeps_every_card() {
    let mut rng = StdRng::seed_from_u64(1);
    let mut state = new_two_seat_table(&mut rng);
    let ids = fresh_shoe_ids();
    assert_eq!(shoe_ids(&state), ids);
    assert_eq!(state.players[0].hand.len(), 10);
    assert_eq!(state.deck.discard.len(), 1);

    play_random_turn(&mut state, 0, &mut rng);

    assert_eq!(shoe_ids(&state), ids);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[1].points, 0);
    assert_eq!(state.players[0].total_score, 0);
    assert_eq!(state.players[1].total_score, 0);
    assert!(!state.round_over);
    assert_eq!(state.turn_phase, TurnPhase::Playing);
    assert_eq!(state.penalty_seat, None);
}

/// Two sets are on the table after the turn. The king that was not in those sets stays out of them.
#[test]
fn test_random_turn_lays_down_two_sets() {
    let four_hearts = card(1, Suit::Hearts, Rank::Four);
    let four_spades = card(2, Suit::Spades, Rank::Four);
    let four_clubs = card(3, Suit::Clubs, Rank::Four);
    let five_hearts = card(4, Suit::Hearts, Rank::Five);
    let five_spades = card(5, Suit::Spades, Rank::Five);
    let five_clubs = card(6, Suit::Clubs, Rank::Five);
    let king = card(7, Suit::Spades, Rank::King);
    let queen = card(8, Suit::Diamonds, Rank::Queen);
    let ace = card(9, Suit::Clubs, Rank::Ace);
    let three = card(10, Suit::Hearts, Rank::Three);
    let other = card(11, Suit::Diamonds, Rank::Three);

    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![
        four_hearts,
        four_spades,
        four_clubs,
        five_hearts,
        five_spades,
        five_clubs,
        king,
    ];
    players[1].hand = vec![other];
    let mut deck = Deck::new();
    deck.cards = vec![three, ace];
    deck.discard = vec![queen];
    let mut state = GameState::new(players, deck);
    let mut rng = StdRng::seed_from_u64(1);

    play_random_turn(&mut state, 0, &mut rng);

    assert!(state.players[0].is_on_board);
    assert!(!state.players[1].is_on_board);
    assert_eq!(state.board.len(), 2);
    assert!(!state.round_over);
    assert_eq!(state.turn_phase, TurnPhase::Playing);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[1].points, 0);
    assert_eq!(state.players[0].total_score, 0);
    assert_eq!(state.players[1].total_score, 0);
    let laid: Vec<Card> = state.board.iter().flatten().copied().collect();
    assert!(laid.contains(&four_hearts));
    assert!(laid.contains(&five_clubs));
    assert!(!laid.contains(&king));
}

/// Chain: opening PlayMeld → free PlayMeld → generate_legal_moves lists the set →
/// play_random_turn lays that set while the seat is already on the board.
#[test]
fn test_play_meld_generate_legal_moves_random_turn_free_meld_chain() {
    let fours = vec![
        card(1, Suit::Hearts, Rank::Four),
        card(2, Suit::Spades, Rank::Four),
        card(3, Suit::Clubs, Rank::Four),
    ];
    let fives = vec![
        card(4, Suit::Hearts, Rank::Five),
        card(5, Suit::Spades, Rank::Five),
        card(6, Suit::Clubs, Rank::Five),
    ];
    let sixes = vec![
        card(7, Suit::Hearts, Rank::Six),
        card(8, Suit::Spades, Rank::Six),
        card(9, Suit::Clubs, Rank::Six),
    ];
    let king = card(10, Suit::Diamonds, Rank::King);
    let queen = card(11, Suit::Diamonds, Rank::Queen);
    let ace = card(12, Suit::Clubs, Rank::Ace);
    let three = card(13, Suit::Hearts, Rank::Three);
    let other = card(14, Suit::Diamonds, Rank::Three);

    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![
        fours[0], fours[1], fours[2], fives[0], fives[1], fives[2], sixes[0], sixes[1],
        sixes[2], king,
    ];
    players[1].hand = vec![other];
    let mut deck = Deck::new();
    deck.cards = vec![three, ace];
    deck.discard = vec![queen];
    let mut state = GameState::new(players, deck);

    assert!(state.apply(Action::PlayMeld(vec![fours.clone(), fives.clone()]), 0));
    assert!(state.players[0].is_on_board);
    assert_eq!(state.board, vec![fours.clone(), fives.clone()]);
    assert!(state.players[0].hand.contains(&sixes[0]));

    let moves = generate_legal_moves(&state, 0);
    assert!(moves
        .iter()
        .any(|action| matches!(action, Action::PlayMeld(melds) if melds == &vec![sixes.clone()])));

    let mut rng = StdRng::seed_from_u64(1);
    play_random_turn(&mut state, 0, &mut rng);

    assert!(state
        .board
        .iter()
        .any(|meld| meld.iter().any(|card| card.id == sixes[0].id)
            && meld.iter().any(|card| card.id == sixes[1].id)
            && meld.iter().any(|card| card.id == sixes[2].id)));
    assert!(!state.players[0].hand.iter().any(|card| {
        card.id == sixes[0].id || card.id == sixes[1].id || card.id == sixes[2].id
    }));
}

/// On the board, a valid set in the hand is laid down during the turn.
#[test]
fn test_random_turn_lays_down_additional_melds() {
    let fours = vec![
        card(1, Suit::Hearts, Rank::Four),
        card(2, Suit::Spades, Rank::Four),
        card(3, Suit::Clubs, Rank::Four),
    ];
    let king = card(4, Suit::Spades, Rank::King);
    let queen = card(5, Suit::Diamonds, Rank::Queen);
    let ace = card(6, Suit::Clubs, Rank::Ace);
    let three = card(7, Suit::Hearts, Rank::Three);
    let other = card(8, Suit::Diamonds, Rank::Three);

    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![fours[0], fours[1], fours[2], king];
    players[0].is_on_board = true;
    players[1].hand = vec![other];
    let mut deck = Deck::new();
    deck.cards = vec![three, ace];
    deck.discard = vec![queen];
    let mut state = GameState::new(players, deck);
    state.board = vec![vec![
        card(10, Suit::Hearts, Rank::Eight),
        card(11, Suit::Spades, Rank::Eight),
        card(12, Suit::Clubs, Rank::Eight),
    ]];
    let mut rng = StdRng::seed_from_u64(1);

    play_random_turn(&mut state, 0, &mut rng);

    assert!(state.players[0].is_on_board);
    assert!(state
        .board
        .iter()
        .any(|meld| meld.iter().any(|card| card.id == fours[0].id)
            && meld.iter().any(|card| card.id == fours[1].id)
            && meld.iter().any(|card| card.id == fours[2].id)));
    assert!(!state.players[0].hand.iter().any(|card| {
        card.id == fours[0].id || card.id == fours[1].id || card.id == fours[2].id
    }));
    assert!(!state.round_over);
}

/// Off the board, the seat pushes. The next hand gains the queen and the ace.
#[test]
fn test_random_turn_pushes_while_off_the_board() {
    let king = card(1, Suit::Spades, Rank::King);
    let queen = card(2, Suit::Diamonds, Rank::Queen);
    let ace = card(3, Suit::Clubs, Rank::Ace);
    let three = card(4, Suit::Hearts, Rank::Three);
    let held: Vec<Card> = (0..11)
        .map(|index| card(20 + index, Suit::Hearts, Rank::Six))
        .collect();
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![king];
    players[1].hand = held.clone();
    let mut deck = Deck::new();
    deck.cards = vec![three, ace];
    deck.discard = vec![queen];
    let mut state = GameState::new(players, deck);
    let mut rng = StdRng::seed_from_u64(1);

    play_random_turn(&mut state, 0, &mut rng);

    assert_eq!(state.players[1].hand.len(), 13);
    assert!(state.players[1].hand.contains(&queen));
    assert!(state.players[1].hand.contains(&ace));
    assert!(state.players[1].hand.starts_with(&held));
    assert_eq!(state.players[0].hand.len(), 1);
    assert!(!state.players[0].is_on_board);
    assert_eq!(state.deck.discard.len(), 1);
    assert_eq!(state.turn_phase, TurnPhase::Playing);
    assert!(!state.round_over);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[1].points, 0);
    assert_eq!(state.players[0].total_score, 0);
    assert_eq!(state.players[1].total_score, 0);
}

/// Every card fits the eights, and the discard pile is empty, so the turn draws until the king.
#[test]
fn test_random_turn_draws_when_every_card_fits() {
    let eight = card(1, Suit::Diamonds, Rank::Eight);
    let set = vec![
        card(2, Suit::Hearts, Rank::Eight),
        card(3, Suit::Spades, Rank::Eight),
        card(4, Suit::Clubs, Rank::Eight),
    ];
    let king = card(5, Suit::Clubs, Rank::King);
    let other = card(6, Suit::Hearts, Rank::Three);
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![eight];
    players[1].hand = vec![other];
    let mut deck = Deck::new();
    deck.cards = vec![king];
    let mut state = GameState::new(players, deck);
    state.board = vec![set];
    let mut rng = StdRng::seed_from_u64(1);

    play_random_turn(&mut state, 0, &mut rng);

    assert_eq!(state.players[0].hand, vec![eight]);
    assert_eq!(state.deck.discard, vec![king]);
    assert!(state.deck.cards.is_empty());
    assert_eq!(state.turn_phase, TurnPhase::Playing);
    assert_eq!(state.penalty_seat, None);
    assert!(!state.players[0].is_on_board);
    assert!(!state.round_over);
    assert_eq!(state.players[0].points, 0);
    assert_eq!(state.players[0].total_score, 0);
    assert_eq!(state.players[1].hand, vec![other]);
}

/// A push that must reshuffle uses the bot seed for both draws.
/// The next seat receives the discard top and the first seeded draw.
#[test]
fn test_random_turn_push_reshuffle_follows_the_bot_seed() {
    let ready = bot_push_reshuffle_table();
    let (seed_a, seed_b) = seeds_whose_bot_draws_differ(&ready.deck, 2);
    let mut left = ready.clone();
    let mut again = ready.clone();
    let mut right = ready.clone();

    play_random_turn(&mut left, 0, &mut StdRng::seed_from_u64(seed_a));
    assert_eq!(left.players[1].hand, push_next_hand(&ready, seed_a));

    play_random_turn(&mut again, 0, &mut StdRng::seed_from_u64(seed_a));
    assert_eq!(left.players, again.players);
    assert_eq!(left.deck, again.deck);
    assert_eq!(left.board, again.board);
    assert_eq!(left.turn_phase, again.turn_phase);

    play_random_turn(&mut right, 0, &mut StdRng::seed_from_u64(seed_b));
    assert_eq!(right.players[1].hand, push_next_hand(&ready, seed_b));
    assert_ne!(left.players[1].hand, right.players[1].hand);
    assert_eq!(left.players[0].points, 0);
    assert_eq!(left.players[1].points, 0);
    assert_eq!(left.players[0].total_score, 0);
    assert_eq!(left.players[1].total_score, 0);
    assert!(!left.round_over);
}

/// Penalty drawing at the start of the turn uses the bot seed.
/// Cards that fit stay in the hand, in that order. The king is discarded.
#[test]
fn test_random_turn_penalty_reshuffle_follows_the_bot_seed() {
    let held = card(4, Suit::Hearts, Rank::Seven);
    let four = card(6, Suit::Hearts, Rank::Four);
    let eight = card(7, Suit::Hearts, Rank::Eight);
    let another = card(8, Suit::Hearts, Rank::Seven);
    let king = card(9, Suit::Clubs, Rank::King);
    let bystander = card(10, Suit::Diamonds, Rank::Nine);
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![held];
    players[1].hand = vec![bystander];
    let mut state = GameState::new(
        players,
        Deck {
            cards: Vec::new(),
            discard: vec![four, eight, another, king],
        },
    );
    state.board = vec![vec![
        card(1, Suit::Hearts, Rank::Five),
        card(2, Suit::Hearts, Rank::Six),
        card(3, Suit::None, Rank::Joker),
    ]];
    state.turn_phase = TurnPhase::PenaltyDrawing;
    state.penalty_seat = Some(0);
    let ready = state;
    let (seed_a, seed_b) = seeds_whose_bot_draws_differ(&ready.deck, 3);
    let mut left = ready.clone();
    let mut again = ready.clone();
    let mut right = ready.clone();

    play_random_turn(&mut left, 0, &mut StdRng::seed_from_u64(seed_a));
    let drawn = bot_draw_n(&ready.deck, seed_a, 4);
    assert_eq!(
        left.players[0].hand,
        vec![held, drawn[0], drawn[1], drawn[2]]
    );
    assert_eq!(left.deck.discard, vec![drawn[3]]);
    assert_eq!(drawn[3], king);
    assert!(left.deck.cards.is_empty());
    assert_eq!(left.turn_phase, TurnPhase::Playing);
    assert_eq!(left.penalty_seat, None);
    assert_eq!(left.players[1].hand, vec![bystander]);

    play_random_turn(&mut again, 0, &mut StdRng::seed_from_u64(seed_a));
    assert_eq!(left.players, again.players);
    assert_eq!(left.deck, again.deck);

    play_random_turn(&mut right, 0, &mut StdRng::seed_from_u64(seed_b));
    assert_ne!(left.players[0].hand, right.players[0].hand);
    assert_eq!(left.players[0].points, 0);
    assert_eq!(left.players[1].points, 0);
    assert!(!left.round_over);
    assert_eq!(left.board, ready.board);
}

fn bot_push_reshuffle_table() -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![card(20, Suit::Spades, Rank::King)];
    players[1].hand = vec![card(21, Suit::Diamonds, Rank::Nine)];
    GameState::new(
        players,
        Deck {
            cards: Vec::new(),
            discard: vec![
                card(1, Suit::Hearts, Rank::Three),
                card(2, Suit::Clubs, Rank::Four),
                card(3, Suit::Diamonds, Rank::Five),
                card(4, Suit::Spades, Rank::Six),
            ],
        },
    )
}

fn push_next_hand(before: &GameState, seed: u64) -> Vec<Card> {
    let mut deck = before.deck.clone();
    let mut rng = StdRng::seed_from_u64(seed);
    let discarded = deck.discard.pop().expect("push needs a top card");
    let penalty = bot_one(&mut deck, &mut rng);
    let _start = bot_one(&mut deck, &mut rng);
    let mut hand = before.players[1].hand.clone();
    hand.push(discarded);
    hand.push(penalty);
    hand
}

fn bot_one(deck: &mut Deck, rng: &mut StdRng) -> Card {
    match deck.draw_with(rng) {
        TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
        other => panic!("expected one card, got {other:?}"),
    }
}

fn bot_draw_n(deck: &Deck, seed: u64, count: usize) -> Vec<Card> {
    let mut deck = deck.clone();
    let mut rng = StdRng::seed_from_u64(seed);
    let mut drawn = Vec::new();
    for _ in 0..count {
        drawn.push(bot_one(&mut deck, &mut rng));
    }
    drawn
}

fn seeds_whose_bot_draws_differ(deck: &Deck, count: usize) -> (u64, u64) {
    let first = bot_draw_n(deck, 0, count);
    for seed in 1..64 {
        if bot_draw_n(deck, seed, count) != first {
            return (0, seed);
        }
    }
    panic!("expected two seeds to draw in a different order");
}
