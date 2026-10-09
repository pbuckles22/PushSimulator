//! Point-averse seats shed high cards. Hoarder seats keep wilds through round 3.
//!
//! A card worth 10 or more is high (10 through king, an ace, a two, a joker).
//! Rounds 4 and 5 are late. A hoarder plays, hits, discards, and pushes wilds
//! from round 4 on. Seed 1 with two random seats still scores 5 and 85.

use rand::rngs::StdRng;
use rand::SeedableRng;

use push_core::actions::Action;
use push_core::card::{Card, Rank, Suit};
use push_core::deck::Deck;
use push_core::game_state::GameState;
use push_core::legal_moves::{visit_legal_kind, LegalKind};
use push_core::player::Player;
use push_core::profiles::{finish_profile_game, play_profile_turn, BotProfile};
use push_core::random_bot::{finish_random_game, game_finished_five_rounds};

fn card(id: u32, suit: Suit, rank: Rank) -> Card {
    Card {
        id,
        suit,
        rank,
        locked_until_turn: 0,
    }
}

fn joker(id: u32) -> Card {
    card(id, Suit::None, Rank::Joker)
}

fn two_seats(hand: Vec<Card>, discard: Vec<Card>, draw: Vec<Card>) -> GameState {
    let mut players = vec![Player::new(0, 0), Player::new(1, 1)];
    players[0].hand = hand;
    players[1].hand = vec![card(900, Suit::Diamonds, Rank::Nine)];
    let mut deck = Deck::new();
    deck.cards = draw;
    deck.discard = discard;
    GameState::new(players, deck)
}

fn play(profile: BotProfile, state: &mut GameState) {
    let mut rng = StdRng::seed_from_u64(1);
    play_profile_turn(state, 0, &mut rng, profile);
}

fn discarded(state: &GameState) -> Card {
    *state.deck.discard.last().expect("the turn left a discard")
}

fn hand_has(state: &GameState, seat: usize, card: Card) -> bool {
    state.players[seat].hand.contains(&card)
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

/// Chain: visit discards → point-averse turn. The ace (15) leaves. The four stays.
#[test]
fn test_visit_legal_discards_point_averse_discards_the_highest_card() {
    let four = card(1, Suit::Hearts, Rank::Four);
    let king = card(2, Suit::Spades, Rank::King);
    let ace = card(3, Suit::Clubs, Rank::Ace);
    let mut state = two_seats(
        vec![four, king, ace],
        vec![card(4, Suit::Diamonds, Rank::Queen)],
        vec![
            card(5, Suit::Hearts, Rank::Six),
            card(6, Suit::Spades, Rank::Seven),
        ],
    );
    state.players[0].is_on_board = true;
    let mut listed = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Discard, &mut |action| {
        listed.push(action);
    });
    let best = listed
        .iter()
        .map(|action| match action {
            Action::DiscardCard(card) => card.get_penalty_value(),
            _ => 0,
        })
        .max()
        .expect("three discards");

    play(BotProfile::PointAverse, &mut state);

    let gone = discarded(&state);
    assert_eq!(gone, ace);
    assert_eq!(gone.get_penalty_value(), best);
    assert!(hand_has(&state, 0, four));
    assert!(hand_has(&state, 0, king));
    assert!(!hand_has(&state, 0, ace));
}

/// A four is taken. An ace is pushed onto the other seat.
#[test]
fn test_point_averse_bot_takes_a_low_discard_and_pushes_a_high_one() {
    let four = card(1, Suit::Hearts, Rank::Four);
    let king = card(2, Suit::Spades, Rank::King);
    let draw = vec![
        card(3, Suit::Clubs, Rank::Six),
        card(4, Suit::Diamonds, Rank::Seven),
    ];
    let mut low = two_seats(vec![king], vec![four], draw.clone());
    play(BotProfile::PointAverse, &mut low);
    assert!(hand_has(&low, 0, four));
    assert!(!hand_has(&low, 1, four));

    let ace = card(8, Suit::Hearts, Rank::Ace);
    let three = card(9, Suit::Clubs, Rank::Three);
    let mut high = two_seats(vec![three], vec![ace], draw);
    play(BotProfile::PointAverse, &mut high);
    assert!(hand_has(&high, 1, ace));
    assert!(!hand_has(&high, 0, ace));
}

/// Chain: visit discards → hoarder turn. The joker stays through round 3.
/// Round 4 discards it.
#[test]
fn test_visit_legal_discards_hoarder_holds_a_wild_until_round_four() {
    let four = card(1, Suit::Hearts, Rank::Four);
    let king = card(2, Suit::Spades, Rank::King);
    let wild = joker(3);
    for round in [1u8, 3] {
        let mut state = two_seats(
            vec![four, king, wild],
            vec![card(4, Suit::Diamonds, Rank::Queen)],
            vec![card(5, Suit::Hearts, Rank::Six)],
        );
        state.round_number = round;
        state.players[0].is_on_board = true;
        let mut listed = Vec::new();
        let _ = visit_legal_kind(&state, 0, LegalKind::Discard, &mut |action| {
            listed.push(action);
        });
        assert!(listed.iter().any(|action| matches!(
            action,
            Action::DiscardCard(card) if card.id == wild.id
        )));

        play(BotProfile::Hoarder, &mut state);

        assert!(hand_has(&state, 0, wild), "round {round} keeps the joker");
        assert_eq!(discarded(&state), king);
    }

    let mut late = two_seats(
        vec![four, king, wild],
        vec![card(4, Suit::Diamonds, Rank::Queen)],
        vec![card(5, Suit::Hearts, Rank::Six)],
    );
    late.round_number = 4;
    late.players[0].is_on_board = true;
    play(BotProfile::Hoarder, &mut late);
    assert_eq!(discarded(&late), wild);
    assert!(hand_has(&late, 0, king));
    assert!(hand_has(&late, 0, four));
}

/// One wild stays. A second wild is shed. A kept wild is taken; a wild past the cap is pushed.
#[test]
fn test_keep_one_wild_holds_the_cap_and_sheds_the_extra() {
    let four = card(1, Suit::Hearts, Rank::Four);
    let king = card(2, Suit::Spades, Rank::King);
    let wild = joker(3);
    let mut under = two_seats(
        vec![four, king, wild],
        vec![card(4, Suit::Diamonds, Rank::Queen)],
        vec![card(5, Suit::Hearts, Rank::Six)],
    );
    under.players[0].is_on_board = true;
    play(BotProfile::Keep(1), &mut under);
    assert!(hand_has(&under, 0, wild));
    assert_eq!(discarded(&under), king);

    let extra = joker(6);
    let mut over = two_seats(
        vec![king, wild, extra],
        vec![card(4, Suit::Diamonds, Rank::Queen)],
        vec![card(5, Suit::Hearts, Rank::Six)],
    );
    over.players[0].is_on_board = true;
    play(BotProfile::Keep(1), &mut over);
    assert!(hand_has(&over, 0, king));
    assert!(discarded(&over).is_wild());
    assert_eq!(
        over.players[0]
            .hand
            .iter()
            .filter(|card| card.is_wild())
            .count(),
        1
    );

    let draw = vec![
        card(7, Suit::Clubs, Rank::Six),
        card(8, Suit::Diamonds, Rank::Seven),
    ];
    let incoming = joker(9);
    let mut empty = two_seats(vec![king, four], vec![incoming], draw.clone());
    play(BotProfile::Keep(1), &mut empty);
    assert!(hand_has(&empty, 0, incoming));
    assert!(!hand_has(&empty, 1, incoming));

    let held = joker(10);
    let mut full = two_seats(vec![king, four, held], vec![incoming], draw);
    play(BotProfile::Keep(1), &mut full);
    assert!(hand_has(&full, 1, incoming));
    assert!(!hand_has(&full, 0, incoming));
}

/// Round 1 takes the joker off the discard. Round 4 pushes it to the other seat.
#[test]
fn test_hoarder_bot_takes_a_wild_until_round_four_then_pushes_it() {
    let wild = joker(1);
    let king = card(2, Suit::Spades, Rank::King);
    let four = card(3, Suit::Hearts, Rank::Four);
    let draw = vec![
        card(4, Suit::Clubs, Rank::Six),
        card(5, Suit::Diamonds, Rank::Seven),
    ];
    let mut early = two_seats(vec![king, four], vec![wild], draw.clone());
    play(BotProfile::Hoarder, &mut early);
    assert!(hand_has(&early, 0, wild));
    assert!(!hand_has(&early, 1, wild));

    let mut late = two_seats(vec![king, four], vec![wild], draw);
    late.round_number = 4;
    play(BotProfile::Hoarder, &mut late);
    assert!(hand_has(&late, 1, wild));
    assert!(!hand_has(&late, 0, wild));
}

/// Chain: visit plays → point-averse lays the joker down. The hoarder keeps it.
#[test]
fn test_visit_legal_plays_point_averse_uses_a_wild_and_hoarder_holds_it() {
    let fours = [
        card(1, Suit::Hearts, Rank::Four),
        card(2, Suit::Spades, Rank::Four),
        card(3, Suit::Clubs, Rank::Four),
    ];
    let fives = [
        card(4, Suit::Hearts, Rank::Five),
        card(5, Suit::Spades, Rank::Five),
    ];
    let wild = joker(6);
    let king = card(7, Suit::Diamonds, Rank::King);
    let low = card(8, Suit::Clubs, Rank::Three);
    let draw = vec![
        card(9, Suit::Hearts, Rank::Six),
        card(10, Suit::Spades, Rank::Seven),
    ];
    let mut seen = two_seats(
        vec![fours[0], fours[1], fours[2], fives[0], fives[1], wild, king],
        vec![low],
        draw.clone(),
    );
    let mut plays = Vec::new();
    let _ = visit_legal_kind(&seen, 0, LegalKind::Play, &mut |action| {
        plays.push(action);
    });
    assert!(plays.iter().any(|action| match action {
        Action::PlayMeld(melds) => melds.iter().flatten().any(|card| card.id == wild.id),
        _ => false,
    }));

    play(BotProfile::PointAverse, &mut seen);
    assert!(seen.players[0].is_on_board);
    assert!(!hand_has(&seen, 0, wild));
    assert!(seen.board.iter().flatten().any(|card| card.id == wild.id));

    let mut held = two_seats(
        vec![fours[0], fours[1], fours[2], fives[0], fives[1], wild, king],
        vec![low],
        draw,
    );
    play(BotProfile::Hoarder, &mut held);
    assert!(!held.players[0].is_on_board);
    assert!(hand_has(&held, 0, wild));
    assert!(held.board.iter().flatten().all(|card| card.id != wild.id));
}

/// Round 4 needs the joker for the third set. The hoarder plays it.
#[test]
fn test_visit_legal_plays_hoarder_uses_a_wild_in_round_four() {
    let fours = [
        card(1, Suit::Hearts, Rank::Four),
        card(2, Suit::Spades, Rank::Four),
        card(3, Suit::Clubs, Rank::Four),
    ];
    let fives = [
        card(4, Suit::Hearts, Rank::Five),
        card(5, Suit::Spades, Rank::Five),
        card(6, Suit::Clubs, Rank::Five),
    ];
    let sixes = [
        card(7, Suit::Hearts, Rank::Six),
        card(8, Suit::Spades, Rank::Six),
    ];
    let wild = joker(9);
    let king = card(10, Suit::Diamonds, Rank::King);
    let low = card(11, Suit::Clubs, Rank::Three);
    let mut state = two_seats(
        vec![
            fours[0], fours[1], fours[2], fives[0], fives[1], fives[2], sixes[0], sixes[1], wild,
            king,
        ],
        vec![low],
        vec![
            card(12, Suit::Hearts, Rank::Seven),
            card(13, Suit::Spades, Rank::Eight),
        ],
    );
    state.round_number = 4;
    let mut plays = Vec::new();
    let _ = visit_legal_kind(&state, 0, LegalKind::Play, &mut |action| {
        plays.push(action);
    });
    assert!(plays.iter().any(|action| match action {
        Action::PlayMeld(melds) => melds.iter().flatten().any(|card| card.id == wild.id),
        _ => false,
    }));

    play(BotProfile::Hoarder, &mut state);

    assert!(state.players[0].is_on_board);
    assert!(!hand_has(&state, 0, wild));
    assert!(state.board.iter().flatten().any(|card| card.id == wild.id));
}

/// Chain: visit hits → point-averse hits the joker. The hoarder keeps it in round 1.
#[test]
fn test_visit_legal_hits_point_averse_hits_a_wild_and_hoarder_holds_it() {
    let wild = joker(1);
    let king = card(2, Suit::Spades, Rank::King);
    let three = card(3, Suit::Hearts, Rank::Three);
    let eights = vec![
        card(4, Suit::Hearts, Rank::Eight),
        card(5, Suit::Spades, Rank::Eight),
        card(6, Suit::Clubs, Rank::Eight),
    ];
    let mut shed = two_seats(
        vec![wild, king, three],
        vec![card(7, Suit::Diamonds, Rank::Queen)],
        vec![card(8, Suit::Hearts, Rank::Six)],
    );
    shed.players[0].is_on_board = true;
    shed.board = vec![eights.clone()];
    shed.drawn_card_id = Some(king.id);
    let mut hits = Vec::new();
    let _ = visit_legal_kind(&shed, 0, LegalKind::Hit, &mut |action| {
        hits.push(action);
    });
    assert!(hits.iter().any(|action| match action {
        Action::HitMeld(hits) => hits
            .iter()
            .any(|hit| hit.cards.iter().any(|card| card.id == wild.id)),
        _ => false,
    }));

    play(BotProfile::PointAverse, &mut shed);
    assert!(shed.board.iter().flatten().any(|card| card.id == wild.id));
    assert!(!hand_has(&shed, 0, wild));

    let mut held = two_seats(
        vec![wild, king, three],
        vec![card(7, Suit::Diamonds, Rank::Queen)],
        vec![card(8, Suit::Hearts, Rank::Six)],
    );
    held.players[0].is_on_board = true;
    held.board = vec![eights];
    held.drawn_card_id = Some(king.id);
    play(BotProfile::Hoarder, &mut held);
    assert!(hand_has(&held, 0, wild));
    assert!(held.board.iter().flatten().all(|card| card.id != wild.id));
    assert_eq!(discarded(&held), king);
}

/// Two random seats through the profile entry still score 5 and 85 on seed 1.
#[test]
fn test_finish_random_game_profile_seed_one_scores_five_and_eighty_five() {
    let through_profile = finish_profile_game(1, [BotProfile::Random, BotProfile::Random]);
    let direct = finish_random_game(1);
    assert_eq!(through_profile, direct);
    assert_eq!(through_profile.state.players[0].total_score, 5);
    assert_eq!(through_profile.state.players[1].total_score, 85);
}

/// A seat that keeps one wild still finishes the seeds that used to stall.
#[test]
fn test_keep_one_wild_vs_point_averse_finishes_five_rounds() {
    let seats = [BotProfile::Keep(1), BotProfile::PointAverse];
    for seed in [1u64, 44, 1_883] {
        let game = finish_profile_game(seed, seats);
        assert!(
            game_finished_five_rounds(&game.state),
            "seed {seed} round {}",
            game.state.round_number
        );
        assert_eq!(shoe_ids(&game.state), fresh_shoe_ids());
    }
}

/// Seeds that stall in round 3 still finish. A low discard is pushed once the
/// round has already run 24 turns, and a seat on the board sheds without taking.
#[test]
fn test_deal_profile_turns_point_averse_vs_hoarder_round_three_seeds_finish() {
    let seats = [BotProfile::PointAverse, BotProfile::Hoarder];
    for seed in [29u64, 44, 1_883, 12_535, 25_042] {
        let game = finish_profile_game(seed, seats);
        assert!(
            game_finished_five_rounds(&game.state),
            "seed {seed} round {}",
            game.state.round_number
        );
        assert_eq!(shoe_ids(&game.state), fresh_shoe_ids());
    }
}

/// Chain: deal three seats → keep-1 and two point-averse turns → five rounds.
/// The shoe is intact.
#[test]
fn test_deal_three_seats_keep_one_vs_point_averse_finishes_five_rounds() {
    let seats = [
        BotProfile::Keep(1),
        BotProfile::PointAverse,
        BotProfile::PointAverse,
    ];
    let game = finish_profile_game(1, seats);
    assert_eq!(game.state.players.len(), 3);
    assert!(game_finished_five_rounds(&game.state));
    assert!(game.state.players.iter().all(|player| player.points == 0));
    assert_eq!(shoe_ids(&game.state), fresh_shoe_ids());
}

/// Chain: deal five seats → keep-4 and four point-averse turns → five rounds.
/// The shoe is intact.
#[test]
fn test_deal_five_seats_keep_four_vs_point_averse_finishes_five_rounds() {
    let seats = [
        BotProfile::Keep(4),
        BotProfile::PointAverse,
        BotProfile::PointAverse,
        BotProfile::PointAverse,
        BotProfile::PointAverse,
    ];
    let game = finish_profile_game(1, seats);
    assert_eq!(game.state.players.len(), 5);
    assert!(game_finished_five_rounds(&game.state));
    assert!(game.state.players.iter().all(|player| player.points == 0));
    assert_eq!(shoe_ids(&game.state), fresh_shoe_ids());
}

/// Chain: deal → point-averse and hoarder turns → five rounds. The shoe is intact.
#[test]
fn test_deal_profile_turns_point_averse_vs_hoarder_finishes_five_rounds() {
    let seats = [BotProfile::PointAverse, BotProfile::Hoarder];
    let first = finish_profile_game(1, seats);
    let second = finish_profile_game(1, seats);
    assert!(game_finished_five_rounds(&first.state));
    assert_eq!(first.state.players[0].points, 0);
    assert_eq!(first.state.players[1].points, 0);
    assert_eq!(shoe_ids(&first.state), fresh_shoe_ids());
    assert_eq!(first, second);
}
