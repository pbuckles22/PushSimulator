//! One WASM tick plays the current point-averse seat and exposes that table as JSON.
//!
//! Chain: deal two seats → one point-averse turn → advance the turn or the round
//! → JSON of that `GameState`. The HTML viewer is not part of this crate.

use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::{GameState, TurnPhase};
use push_core::player::Player;
use push_core::profiles::{finish_profile_game, play_profile_turn, BotProfile};
use push_core::random_bot::{game_finished_five_rounds, new_two_seat_table};
use push_wasm::WasmGame;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde_json::Value;

fn card(id: u32, suit: Suit, rank: Rank) -> Card {
    Card {
        id,
        suit,
        rank,
        locked_until_turn: 0,
    }
}

/// The match loop's one-turn advance, written here so the tick cannot pass by
/// calling the same helper the assertion also calls.
fn point_averse_table(seed: u64, ticks: usize) -> (GameState, usize) {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut state = new_two_seat_table(&mut rng);
    let mut seat = 0usize;
    for _ in 0..ticks {
        if game_finished_five_rounds(&state) || seat >= state.players.len() {
            break;
        }
        play_profile_turn(&mut state, seat, &mut rng, BotProfile::PointAverse);
        if state.round_over {
            assert!(state.advance_to_next_round_with(&mut rng));
            seat = 0;
            continue;
        }
        if state.turn_phase == TurnPhase::PenaltyDrawing {
            continue;
        }
        state.advance_turn();
        seat = (seat + 1) % state.players.len();
    }
    (state, seat)
}

fn card_ids(state: &GameState) -> Vec<u32> {
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

fn assert_json_is_the_table(game: &WasmGame) {
    let json = game.state_json();
    let parsed: GameState = serde_json::from_str(&json).expect("tick JSON is a GameState");
    assert_eq!(&parsed, game.state());
    let value: Value = serde_json::from_str(&json).expect("tick JSON is an object");
    for key in [
        "players",
        "deck",
        "round_number",
        "board",
        "turn_counter",
        "drawn_card_id",
        "turn_phase",
        "penalty_seat",
        "round_over",
    ] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    let hand = &value["players"][0]["hand"][0];
    for key in ["id", "suit", "rank", "locked_until_turn"] {
        assert!(hand.get(key).is_some(), "missing card {key}");
    }
}

#[test]
fn test_opening_table_json_names_the_dealt_state() {
    let game = WasmGame::new(1);
    assert_eq!(game.seat(), 0);
    assert_json_is_the_table(&game);
    let value: Value = serde_json::from_str(&game.state_json()).expect("opening JSON");
    assert_eq!(value["turn_phase"], "Playing");
    assert_eq!(value["round_number"], 1);
    assert_eq!(value["round_over"], false);
    assert_eq!(value["turn_counter"], 0);
    assert!(value["penalty_seat"].is_null());
    assert!(value["drawn_card_id"].is_null());
    assert_eq!(value["players"][0]["hand"].as_array().unwrap().len(), 10);
    assert_eq!(value["players"][1]["hand"].as_array().unwrap().len(), 10);
    assert_eq!(value["deck"]["discard"].as_array().unwrap().len(), 1);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
}

/// Chain: deal → one point-averse turn → the same advance the match loop uses.
#[test]
fn test_deal_point_averse_tick_matches_one_profile_turn() {
    let mut game = WasmGame::new(1);
    game.tick();

    let (expected, seat) = point_averse_table(1, 1);
    assert_eq!(game.state(), &expected);
    assert_eq!(game.seat(), seat);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
    assert!(game.state().players.iter().all(|player| player.points == 0));
    assert_json_is_the_table(&game);
}

/// The second tick is the other seat, on the same shoe and the same seed.
#[test]
fn test_second_tick_plays_the_other_point_averse_seat() {
    let mut game = WasmGame::new(1);
    game.tick();
    game.tick();

    let (expected, seat) = point_averse_table(1, 2);
    assert_eq!(game.state(), &expected);
    assert_eq!(game.seat(), seat);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
    assert_json_is_the_table(&game);
}

#[test]
fn test_same_seed_repeats_the_tick() {
    let mut left = WasmGame::new(1);
    let mut again = WasmGame::new(1);
    left.tick();
    again.tick();
    assert_eq!(left.state(), again.state());
    assert_eq!(left.seat(), again.seat());
    assert_eq!(left.state_json(), again.state_json());
}

#[test]
fn test_other_seed_does_not_match_the_tick() {
    let mut left = WasmGame::new(1);
    let mut right = WasmGame::new(2);
    left.tick();
    right.tick();
    assert_ne!(left.state(), right.state());
}

/// A penalty draw that finds a safe card returns to playing, then the turn advances.
#[test]
fn test_penalty_draw_discards_the_safe_card_and_advances() {
    let held = card(1, Suit::Hearts, Rank::Four);
    let bystander = card(2, Suit::Diamonds, Rank::Five);
    let king = card(3, Suit::Clubs, Rank::King);
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = vec![held];
    players[1].hand = vec![bystander];
    let mut state = GameState::new(
        players,
        Deck {
            cards: vec![king],
            discard: Vec::new(),
        },
    );
    state.turn_phase = TurnPhase::PenaltyDrawing;
    state.penalty_seat = Some(0);
    let mut game = WasmGame::from_table(state, 1, 0);

    game.tick();

    assert_eq!(game.seat(), 1);
    assert_eq!(game.state().turn_counter, 1);
    assert_eq!(game.state().turn_phase, TurnPhase::Playing);
    assert_eq!(game.state().penalty_seat, None);
    assert!(!game.state().round_over);
    assert_eq!(game.state().round_number, 1);
    assert_eq!(game.state().players[0].hand, vec![held]);
    assert_eq!(game.state().players[1].hand, vec![bystander]);
    assert_eq!(game.state().deck.discard, vec![king]);
    assert!(game.state().deck.cards.is_empty());
    assert_eq!(game.state().players[0].points, 0);
    assert_eq!(game.state().players[1].points, 0);
    assert_eq!(game.state().players[0].total_score, 0);
}

/// Nothing left to draw ends the round. The tick scores those hands and deals again.
#[test]
fn test_empty_penalty_draw_deals_the_next_round() {
    let dealt = WasmGame::new(4);
    let mut state = dealt.state().clone();
    let mut rest = Vec::new();
    rest.append(&mut state.deck.cards);
    rest.append(&mut state.deck.discard);
    state.players[0].hand.append(&mut rest);
    state.turn_phase = TurnPhase::PenaltyDrawing;
    state.penalty_seat = Some(0);
    let penalty: u32 = state
        .players
        .iter()
        .map(|player| player.calculate_hand_penalty())
        .sum();
    let mut game = WasmGame::from_table(state, 9, 0);

    game.tick();

    assert_eq!(game.seat(), 0);
    assert_eq!(game.state().round_number, 2);
    assert!(!game.state().round_over);
    assert_eq!(game.state().turn_phase, TurnPhase::Playing);
    assert_eq!(game.state().penalty_seat, None);
    assert_eq!(game.state().turn_counter, 0);
    assert!(game.state().drawn_card_id.is_none());
    assert!(game.state().board.is_empty());
    assert!(game
        .state()
        .players
        .iter()
        .all(|player| !player.is_on_board && player.points == 0 && player.hand.len() == 10));
    assert_eq!(
        game.state()
            .players
            .iter()
            .map(|player| player.total_score)
            .sum::<u32>(),
        penalty
    );
    assert_eq!(game.state().deck.discard.len(), 1);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
}

/// A round that is already over is scored and dealt. The seat returns to 0.
#[test]
fn test_round_over_tick_scores_and_deals_the_next_round() {
    let dealt = WasmGame::new(1);
    let mut state = dealt.state().clone();
    let penalty: u32 = state
        .players
        .iter()
        .map(|player| player.calculate_hand_penalty())
        .sum();
    state.round_over = true;
    let mut game = WasmGame::from_table(state, 3, 1);

    game.tick();

    assert_eq!(game.seat(), 0);
    assert_eq!(game.state().round_number, 2);
    assert!(!game.state().round_over);
    assert_eq!(game.state().turn_phase, TurnPhase::Playing);
    assert_eq!(game.state().penalty_seat, None);
    assert_eq!(game.state().turn_counter, 0);
    assert!(game.state().board.is_empty());
    assert!(game
        .state()
        .players
        .iter()
        .all(|player| !player.is_on_board && player.points == 0 && player.hand.len() == 10));
    assert_eq!(
        game.state()
            .players
            .iter()
            .map(|player| player.total_score)
            .sum::<u32>(),
        penalty
    );
    assert_eq!(game.state().deck.discard.len(), 1);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
    assert_json_is_the_table(&game);
}

#[test]
fn test_tick_with_no_such_seat_leaves_the_table() {
    let dealt = WasmGame::new(1);
    let state = dealt.state().clone();
    let mut game = WasmGame::from_table(state.clone(), 1, 9);

    game.tick();

    assert_eq!(game.seat(), 9);
    assert_eq!(game.state(), &state);
    assert_eq!(game.state_json(), dealt.state_json());
}

/// Chain: deal → point-averse ticks → five rounds. One more tick does not play round 6.
#[test]
fn test_deal_point_averse_ticks_finish_five_rounds_and_then_stop() {
    let finished = finish_profile_game(1, [BotProfile::PointAverse, BotProfile::PointAverse]);
    assert!(game_finished_five_rounds(&finished.state));
    assert!(finished
        .state
        .players
        .iter()
        .all(|player| player.points == 0));

    let mut game = WasmGame::new(1);
    let mut ticks = 0usize;
    while !game_finished_five_rounds(game.state()) {
        game.tick();
        ticks += 1;
        assert!(
            ticks <= 8_000,
            "seed 1 did not finish five rounds in 8000 ticks"
        );
    }

    assert_eq!(ticks, finished.turns as usize);
    assert_eq!(game.state(), &finished.state);
    assert_eq!(game.seat(), 0);
    assert_eq!(game.state().round_number, 6);
    assert!(!game.state().round_over);
    assert_eq!(card_ids(game.state()), (0..108).collect::<Vec<_>>());
    assert_json_is_the_table(&game);

    let stopped = game.state().clone();
    game.tick();
    assert_eq!(game.state(), &stopped);
    assert_eq!(game.seat(), 0);
}
