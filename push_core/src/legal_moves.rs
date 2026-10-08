//! Every action the engine would accept for one seat.

use std::collections::HashSet;

use crate::actions::{validate_action, Action, MeldHit, WildSteal};
use crate::card::{Card, Rank, Suit};
use crate::deck::{one_card_draw_count, Deck};
use crate::game_state::{GameState, TurnPhase};
use crate::resolution::ActionResolution;
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// Which family of legal actions a search wants.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LegalKind {
    Take,
    Push,
    Play,
    Hit,
    Steal,
    Discard,
    Draw,
}

/// Actions [`GameState::apply`] would accept for `actor_index` on this table.
///
/// This is the compatibility list. [`visit_legal_moves`] is the search.
/// The hand is checked against the board: a hit has to leave a set or a run,
/// a steal has to replace a wild, and an off-board discard of a card that fits
/// is left out. The table stays as it was.
pub fn generate_legal_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    let mut moves = Vec::new();
    visit_legal_moves(state, actor_index, &mut |action| moves.push(action));
    moves
}

/// Calls `visit` once for each action [`generate_legal_moves`] would list, in that order.
pub fn visit_legal_moves(state: &GameState, actor: usize, visit: &mut impl FnMut(Action)) {
    for kind in [
        LegalKind::Take,
        LegalKind::Push,
        LegalKind::Play,
        LegalKind::Hit,
        LegalKind::Steal,
        LegalKind::Discard,
        LegalKind::Draw,
    ] {
        visit_legal_kind(state, actor, kind, visit);
    }
}

/// Calls `visit` for one family of actions, in the same order the full list has them.
///
/// A partial meld that already has two natural ranks and two natural suits is skipped.
/// Adding cards cannot make that group a set or a run. Wilds, ace-high runs, ace-low
/// runs, duplicate cards, all-wild melds, and a second meld of the same type stay.
pub fn visit_legal_kind(
    state: &GameState,
    actor: usize,
    kind: LegalKind,
    visit: &mut impl FnMut(Action),
) {
    if state.round_over || actor >= state.players.len() {
        return;
    }
    match kind {
        LegalKind::Take => {
            if !state.deck.discard.is_empty() && accepts(state, actor, Action::TakeDiscard) {
                visit(Action::TakeDiscard);
            }
        }
        LegalKind::Push => {
            if push_is_legal(&state.deck) {
                visit(Action::PushDiscard);
            }
        }
        LegalKind::Play => visit_plays(state, actor, visit),
        LegalKind::Hit => visit_hits(state, actor, visit),
        LegalKind::Steal => visit_steals(state, actor, visit),
        LegalKind::Discard => visit_discards(state, actor, visit),
        LegalKind::Draw => {
            if state.turn_phase == TurnPhase::PenaltyDrawing
                && state.penalty_seat == Some(actor)
                && accepts(state, actor, Action::DrawFromDeck)
            {
                visit(Action::DrawFromDeck);
            }
        }
    }
}

fn accepts(state: &GameState, actor_index: usize, action: Action) -> bool {
    matches!(
        validate_action(state, actor_index, &action),
        ActionResolution::Accepted(_)
    )
}

pub(crate) fn push_is_legal(deck: &Deck) -> bool {
    let Some(left) = deck.discard.len().checked_sub(1) else {
        return false;
    };
    one_card_draw_count(deck.cards.len(), left) >= 2
}

/// Two natural ranks and two natural suits cannot become a set or a run.
fn group_is_dead(cards: &[Card]) -> bool {
    ranks_and_suits_conflict(cards.iter().copied())
}

fn addition_fits(meld: &[Card], extra: &[Card]) -> bool {
    let n = meld.len() + extra.len();
    if n > 32 {
        let mut with = Vec::with_capacity(n);
        with.extend(meld.iter().copied());
        with.extend(extra.iter().copied());
        return validate_set(&with) || validate_run(&with);
    }
    let blank = Card {
        id: 0,
        suit: Suit::None,
        rank: Rank::Joker,
        locked_until_turn: 0,
    };
    let mut with = [blank; 32];
    with[..meld.len()].copy_from_slice(meld);
    with[meld.len()..n].copy_from_slice(extra);
    validate_set(&with[..n]) || validate_run(&with[..n])
}

fn group_is_dead_with(meld: &[Card], extra: &[Card]) -> bool {
    ranks_and_suits_conflict(meld.iter().copied().chain(extra.iter().copied()))
}

/// Heap-free. A second natural rank together with a second natural suit is a dead group.
fn ranks_and_suits_conflict(cards: impl Iterator<Item = Card>) -> bool {
    let mut first_rank = None;
    let mut second_rank = false;
    let mut first_suit = None;
    let mut second_suit = false;
    for card in cards {
        if card.is_wild() {
            continue;
        }
        match first_rank {
            None => first_rank = Some(card.rank),
            Some(rank) if card.rank != rank => second_rank = true,
            Some(_) => {}
        }
        if card.suit != Suit::None {
            match first_suit {
                None => first_suit = Some(card.suit),
                Some(suit) if card.suit != suit => second_suit = true,
                Some(_) => {}
            }
        }
        if second_rank && second_suit {
            return true;
        }
    }
    false
}

fn playable_hand(state: &GameState, actor_index: usize) -> Vec<Card> {
    state.players[actor_index]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .collect()
}

fn visit_plays(state: &GameState, actor: usize, visit: &mut impl FnMut(Action)) {
    if state.players[actor].is_on_board {
        return;
    }
    let groups = match state.round_number {
        1 | 2 | 3 | 5 => 2,
        4 => 3,
        _ => return,
    };
    let cards = playable_hand(state, actor);
    let mut built = vec![Vec::new(); groups];
    assign_plays(
        state,
        actor,
        state.round_number,
        &cards,
        0,
        &mut built,
        visit,
    );
}

fn assign_plays(
    state: &GameState,
    actor: usize,
    round: u8,
    cards: &[Card],
    index: usize,
    groups: &mut [Vec<Card>],
    visit: &mut impl FnMut(Action),
) {
    if index == cards.len() {
        if groups.iter().all(|group| !group.is_empty()) && check_round_requirements(round, groups) {
            let action = Action::PlayMeld(groups.to_vec());
            if accepts(state, actor, action.clone()) {
                visit(action);
            }
        }
        return;
    }
    assign_plays(state, actor, round, cards, index + 1, groups, visit);
    for slot in 0..groups.len() {
        groups[slot].push(cards[index]);
        if !group_is_dead(&groups[slot]) {
            assign_plays(state, actor, round, cards, index + 1, groups, visit);
        }
        groups[slot].pop();
    }
}

fn visit_hits(state: &GameState, actor: usize, visit: &mut impl FnMut(Action)) {
    if !state.players[actor].is_on_board || state.board.is_empty() {
        return;
    }
    let hand = playable_hand(state, actor);
    if hand.is_empty() {
        return;
    }
    let additions: Vec<Vec<Vec<Card>>> = state
        .board
        .iter()
        .map(|meld| additions(meld, &hand))
        .collect();
    let mut hits = Vec::new();
    let mut used = HashSet::new();
    combine_hits(state, actor, 0, &additions, &mut used, &mut hits, visit);
}

fn additions(meld: &[Card], hand: &[Card]) -> Vec<Vec<Card>> {
    let mut found = Vec::new();
    let mut extra = Vec::new();
    collect_additions(meld, hand, 0, &mut extra, &mut found);
    found
}

fn collect_additions(
    meld: &[Card],
    hand: &[Card],
    index: usize,
    extra: &mut Vec<Card>,
    found: &mut Vec<Vec<Card>>,
) {
    if index == hand.len() {
        if extra.is_empty() {
            return;
        }
        if addition_fits(meld, extra) {
            found.push(extra.clone());
        }
        return;
    }
    collect_additions(meld, hand, index + 1, extra, found);
    extra.push(hand[index]);
    if !group_is_dead_with(meld, extra) {
        collect_additions(meld, hand, index + 1, extra, found);
    }
    extra.pop();
}

fn combine_hits(
    state: &GameState,
    actor: usize,
    meld_index: usize,
    additions: &[Vec<Vec<Card>>],
    used: &mut HashSet<u32>,
    hits: &mut Vec<MeldHit>,
    visit: &mut impl FnMut(Action),
) {
    if meld_index == additions.len() {
        if !hits.is_empty() {
            let action = Action::HitMeld(hits.clone());
            if accepts(state, actor, action.clone()) {
                visit(action);
            }
        }
        return;
    }
    combine_hits(state, actor, meld_index + 1, additions, used, hits, visit);
    for extra in &additions[meld_index] {
        if extra.iter().any(|card| used.contains(&card.id)) {
            continue;
        }
        for card in extra {
            used.insert(card.id);
        }
        hits.push(MeldHit {
            meld_index,
            cards: extra.clone(),
        });
        combine_hits(state, actor, meld_index + 1, additions, used, hits, visit);
        hits.pop();
        for card in extra {
            used.remove(&card.id);
        }
    }
}

fn visit_steals(state: &GameState, actor: usize, visit: &mut impl FnMut(Action)) {
    if !state.players[actor].is_on_board {
        return;
    }
    for (meld_index, meld) in state.board.iter().enumerate() {
        for wild in meld.iter().copied().filter(|card| card.is_wild()) {
            for natural in state.players[actor]
                .hand
                .iter()
                .copied()
                .filter(|card| !card.is_wild())
            {
                let action = Action::StealWild(WildSteal {
                    meld_index,
                    wild,
                    natural,
                });
                if accepts(state, actor, action.clone()) {
                    visit(action);
                }
            }
        }
    }
}

fn visit_discards(state: &GameState, actor: usize, visit: &mut impl FnMut(Action)) {
    for card in &state.players[actor].hand {
        let action = Action::DiscardCard(*card);
        if accepts(state, actor, action.clone()) {
            visit(action);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_legal_moves, visit_legal_kind, LegalKind};
    use crate::actions::{Action, MeldHit, WildSteal};
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
    use crate::game_state::{GameState, TurnPhase};
    use crate::legacy_oracle::{accepts, assign_plays, hit_moves};
    use crate::player::Player;
    use crate::validation::card_can_be_played;

    fn card(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    fn locked(id: u32, suit: Suit, rank: Rank, locked_until_turn: u32) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn,
        }
    }

    fn table(hand: Vec<Card>, other: Vec<Card>, draw: Vec<Card>) -> GameState {
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].points = 4;
        players[1].points = 4;
        players[0].total_score = 9;
        players[1].total_score = 9;
        players[0].hand = hand;
        players[1].hand = other;
        let mut deck = Deck::new();
        deck.cards = draw;
        deck.discard = vec![card(900, Suit::Diamonds, Rank::Queen)];
        GameState::new(players, deck)
    }

    fn same_table(left: &GameState, right: &GameState) {
        assert_eq!(left.players, right.players);
        assert_eq!(left.deck, right.deck);
        assert_eq!(left.round_number, right.round_number);
        assert_eq!(left.board, right.board);
        assert_eq!(left.turn_counter, right.turn_counter);
        assert_eq!(left.drawn_card_id, right.drawn_card_id);
        assert_eq!(left.turn_phase, right.turn_phase);
        assert_eq!(left.penalty_seat, right.penalty_seat);
        assert_eq!(left.round_over, right.round_over);
    }

    fn listed(state: &GameState, actor: usize) -> Vec<Action> {
        let before = state.clone();
        let moves = generate_legal_moves(state, actor);
        same_table(state, &before);
        for action in &moves {
            let mut trial = state.clone();
            assert!(
                trial.apply(action.clone(), actor),
                "listed action was refused: {action:?}"
            );
            assert_eq!(trial.players[0].points, state.players[0].points);
            assert_eq!(trial.players[1].points, state.players[1].points);
            assert_eq!(trial.players[0].total_score, state.players[0].total_score);
            assert_eq!(trial.players[1].total_score, state.players[1].total_score);
        }
        moves
    }

    fn has_hit(moves: &[Action], hits: &[MeldHit]) -> bool {
        moves
            .iter()
            .any(|action| matches!(action, Action::HitMeld(found) if found == hits))
    }

    fn has_discard(moves: &[Action], card: Card) -> bool {
        moves
            .iter()
            .any(|action| matches!(action, Action::DiscardCard(found) if *found == card))
    }

    fn has_play(moves: &[Action], melds: &[Vec<Card>]) -> bool {
        moves
            .iter()
            .any(|action| matches!(action, Action::PlayMeld(found) if found == melds))
    }

    fn has_steal(moves: &[Action], steal: WildSteal) -> bool {
        moves
            .iter()
            .any(|action| matches!(action, Action::StealWild(found) if *found == steal))
    }

    /// Seat 0 is on the board. The 8♦ hits the eights, the 9♥ hits the heart run,
    /// and the joker hits either. The 4♦ and the king fit nothing, so they are
    /// discards only. Take and push are open. A play, a steal, and a penalty draw
    /// are not.
    #[test]
    fn test_generate_legal_moves() {
        let eight_diamonds = card(1, Suit::Diamonds, Rank::Eight);
        let nine_hearts = card(2, Suit::Hearts, Rank::Nine);
        let four_diamonds = card(3, Suit::Diamonds, Rank::Four);
        let king = card(4, Suit::Spades, Rank::King);
        let joker = card(5, Suit::None, Rank::Joker);
        let mut state = table(
            vec![eight_diamonds, nine_hearts, four_diamonds, king, joker],
            vec![card(6, Suit::Clubs, Rank::Three)],
            vec![
                card(7, Suit::Clubs, Rank::Jack),
                card(8, Suit::Spades, Rank::Three),
            ],
        );
        state.players[0].is_on_board = true;
        state.board = vec![
            vec![
                card(10, Suit::Spades, Rank::Eight),
                card(11, Suit::Clubs, Rank::Eight),
                card(12, Suit::Hearts, Rank::Eight),
            ],
            vec![
                card(20, Suit::Hearts, Rank::Five),
                card(21, Suit::Hearts, Rank::Six),
                card(22, Suit::Hearts, Rank::Seven),
                card(23, Suit::Hearts, Rank::Eight),
            ],
        ];

        let moves = listed(&state, 0);

        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![eight_diamonds],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 1,
                cards: vec![nine_hearts],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![joker],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 1,
                cards: vec![joker],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![eight_diamonds, joker],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[
                MeldHit {
                    meld_index: 0,
                    cards: vec![eight_diamonds],
                },
                MeldHit {
                    meld_index: 1,
                    cards: vec![nine_hearts, joker],
                },
            ]
        ));
        assert!(!has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![four_diamonds],
            }]
        ));
        assert!(!has_hit(
            &moves,
            &[MeldHit {
                meld_index: 1,
                cards: vec![eight_diamonds],
            }]
        ));
        assert!(!has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![king],
            }]
        ));
        assert!(has_discard(&moves, eight_diamonds));
        assert!(has_discard(&moves, nine_hearts));
        assert!(has_discard(&moves, four_diamonds));
        assert!(has_discard(&moves, king));
        assert!(has_discard(&moves, joker));
        assert!(moves.contains(&Action::TakeDiscard));
        assert!(moves.contains(&Action::PushDiscard));
        assert!(!moves
            .iter()
            .any(|action| matches!(action, Action::PlayMeld(_))));
        assert!(!moves
            .iter()
            .any(|action| matches!(action, Action::StealWild(_))));
        assert!(!moves.contains(&Action::DrawFromDeck));
        assert_eq!(moves.len(), 18);
        assert!(!state.round_over);
        assert_eq!(state.turn_phase, TurnPhase::Playing);
    }

    /// The 3♠ fits 5♠–8♠ only together with the 4♠.
    #[test]
    fn test_generate_legal_moves_gap_needs_both_cards() {
        let three = card(1, Suit::Spades, Rank::Three);
        let four = card(2, Suit::Spades, Rank::Four);
        let king = card(3, Suit::Hearts, Rank::King);
        let mut state = table(
            vec![three, four, king],
            vec![card(4, Suit::Clubs, Rank::Queen)],
            vec![
                card(5, Suit::Diamonds, Rank::Jack),
                card(6, Suit::Clubs, Rank::Ace),
            ],
        );
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Five),
            card(11, Suit::Spades, Rank::Six),
            card(12, Suit::Spades, Rank::Seven),
            card(13, Suit::Spades, Rank::Eight),
        ]];

        let moves = listed(&state, 0);

        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![four],
            }]
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![three, four],
            }]
        ));
        assert!(!has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![three],
            }]
        ));
        assert_eq!(moves.len(), 7);
    }

    /// Off the board, two sets can be laid down. The 8♦ fits the table, so it
    /// cannot be discarded. The king can. A hit is refused.
    #[test]
    fn test_generate_legal_moves_off_board_lays_down_and_keeps_a_fitting_card() {
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
        let eight = card(7, Suit::Diamonds, Rank::Eight);
        let king = card(8, Suit::Spades, Rank::King);
        let mut hand = fours.clone();
        hand.extend(fives.clone());
        hand.push(eight);
        hand.push(king);
        let mut state = table(
            hand,
            vec![card(9, Suit::Diamonds, Rank::Queen)],
            vec![
                card(30, Suit::Hearts, Rank::Jack),
                card(31, Suit::Clubs, Rank::Ace),
            ],
        );
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Clubs, Rank::Eight),
            card(12, Suit::Hearts, Rank::Eight),
        ]];

        let moves = listed(&state, 0);

        assert!(has_play(&moves, &[fours.clone(), fives.clone()]));
        assert!(has_play(&moves, &[fives, fours.clone()]));
        assert!(!has_play(&moves, &[fours]));
        assert!(has_discard(&moves, king));
        assert!(has_discard(&moves, card(1, Suit::Hearts, Rank::Four)));
        assert!(!has_discard(&moves, eight));
        assert!(!moves
            .iter()
            .any(|action| matches!(action, Action::HitMeld(_))));
        assert!(!moves.contains(&Action::DrawFromDeck));
        assert_eq!(moves.len(), 11);
    }

    /// Six fours are two sets of three, in either order. One meld of six is not a round-1 play.
    #[test]
    fn test_generate_legal_moves_splits_six_fours() {
        let fours = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Diamonds, Rank::Four),
            card(3, Suit::Clubs, Rank::Four),
            card(4, Suit::Spades, Rank::Four),
            card(5, Suit::Hearts, Rank::Four),
            card(6, Suit::Diamonds, Rank::Four),
        ];
        let state = table(
            fours.clone(),
            vec![card(9, Suit::Clubs, Rank::Queen)],
            vec![
                card(30, Suit::Spades, Rank::Jack),
                card(31, Suit::Clubs, Rank::Ace),
            ],
        );
        let first = fours[..3].to_vec();
        let second = fours[3..].to_vec();

        let moves = listed(&state, 0);

        assert!(has_play(&moves, &[first.clone(), second.clone()]));
        assert!(has_play(&moves, &[second, first]));
        assert!(!has_play(&moves, &[fours.clone()]));
        assert!(has_discard(&moves, fours[0]));
        assert_eq!(moves.len(), 28);
    }

    /// A locked 8♦ stays out of the hit. It can still be discarded.
    #[test]
    fn test_generate_legal_moves_locked_card_cannot_hit() {
        let eight = locked(1, Suit::Diamonds, Rank::Eight, 3);
        let king = card(2, Suit::Spades, Rank::King);
        let mut state = table(
            vec![eight, king],
            vec![card(3, Suit::Clubs, Rank::Three)],
            vec![
                card(4, Suit::Hearts, Rank::Jack),
                card(5, Suit::Clubs, Rank::Ace),
            ],
        );
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Clubs, Rank::Eight),
            card(12, Suit::Hearts, Rank::Eight),
        ]];

        let moves = listed(&state, 0);

        assert!(!moves
            .iter()
            .any(|action| matches!(action, Action::HitMeld(_))));
        assert!(has_discard(&moves, eight));
        assert!(has_discard(&moves, king));
        assert_eq!(moves.len(), 4);
    }

    /// The 5♦ replaces the joker. The 9♦ does not. A locked 5♦ cannot steal or hit.
    #[test]
    fn test_generate_legal_moves_steals_the_card_the_wild_stands_for() {
        let five = card(1, Suit::Diamonds, Rank::Five);
        let nine = card(2, Suit::Diamonds, Rank::Nine);
        let king = card(3, Suit::Hearts, Rank::King);
        let joker = card(10, Suit::None, Rank::Joker);
        let mut state = table(
            vec![five, nine, king],
            vec![card(4, Suit::Clubs, Rank::Three)],
            vec![
                card(5, Suit::Hearts, Rank::Jack),
                card(6, Suit::Clubs, Rank::Ace),
            ],
        );
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(11, Suit::Spades, Rank::Five),
            card(12, Suit::Hearts, Rank::Five),
            joker,
        ]];

        let moves = listed(&state, 0);

        assert!(has_steal(
            &moves,
            WildSteal {
                meld_index: 0,
                wild: joker,
                natural: five,
            }
        ));
        assert!(!has_steal(
            &moves,
            WildSteal {
                meld_index: 0,
                wild: joker,
                natural: nine,
            }
        ));
        assert!(has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![five],
            }]
        ));
        assert!(!has_hit(
            &moves,
            &[MeldHit {
                meld_index: 0,
                cards: vec![nine],
            }]
        ));
        assert_eq!(moves.len(), 7);

        let locked_five = locked(1, Suit::Diamonds, Rank::Five, 4);
        state.players[0].hand[0] = locked_five;
        let locked_moves = listed(&state, 0);
        assert!(!has_steal(
            &locked_moves,
            WildSteal {
                meld_index: 0,
                wild: joker,
                natural: locked_five,
            }
        ));
        assert!(!locked_moves
            .iter()
            .any(|action| matches!(action, Action::HitMeld(_))));
        assert!(has_discard(&locked_moves, locked_five));
    }

    /// The card just taken can be discarded while it fits. The other eight cannot.
    #[test]
    fn test_generate_legal_moves_quick_discard_of_a_fitting_card() {
        let taken = card(1, Suit::Diamonds, Rank::Eight);
        let held = card(2, Suit::Clubs, Rank::Eight);
        let king = card(3, Suit::Spades, Rank::King);
        let mut state = table(
            vec![taken, held, king],
            vec![card(4, Suit::Hearts, Rank::Three)],
            vec![
                card(5, Suit::Hearts, Rank::Jack),
                card(6, Suit::Clubs, Rank::Ace),
            ],
        );
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Hearts, Rank::Eight),
            card(12, Suit::Diamonds, Rank::Eight),
        ]];
        state.drawn_card_id = Some(taken.id);

        let moves = listed(&state, 0);

        assert!(has_discard(&moves, taken));
        assert!(has_discard(&moves, king));
        assert!(!has_discard(&moves, held));
        assert!(!moves.contains(&Action::DrawFromDeck));
        assert_eq!(moves.len(), 4);
    }

    /// Only the penalty seat is offered the draw. A fitting eight stays in the hand.
    /// The king can still be discarded.
    #[test]
    fn test_generate_legal_moves_penalty_draw_is_only_for_that_seat() {
        let eight = card(1, Suit::Diamonds, Rank::Eight);
        let king = card(2, Suit::Spades, Rank::King);
        let mut state = table(
            vec![eight, king],
            vec![card(3, Suit::Clubs, Rank::Three)],
            vec![
                card(4, Suit::Hearts, Rank::Jack),
                card(5, Suit::Clubs, Rank::Ace),
            ],
        );
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Clubs, Rank::Eight),
            card(12, Suit::Hearts, Rank::Eight),
        ]];
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(0);

        let moves = listed(&state, 0);
        assert!(moves.contains(&Action::DrawFromDeck));
        assert!(has_discard(&moves, king));
        assert!(!has_discard(&moves, eight));
        assert!(!moves
            .iter()
            .any(|action| matches!(action, Action::HitMeld(_))));

        let other = listed(&state, 1);
        assert!(!other.contains(&Action::DrawFromDeck));
        assert!(has_discard(&other, card(3, Suit::Clubs, Rank::Three)));
    }

    /// A closed round lists nothing. An empty discard lists no take and no push.
    #[test]
    fn test_generate_legal_moves_round_over_and_empty_discard_list_nothing_to_draw() {
        let king = card(1, Suit::Spades, Rank::King);
        let mut state = table(
            vec![king],
            vec![card(2, Suit::Clubs, Rank::Three)],
            vec![card(3, Suit::Hearts, Rank::Jack)],
        );
        state.players[0].is_on_board = true;
        state.round_over = true;

        assert!(listed(&state, 0).is_empty());
        assert!(listed(&state, 1).is_empty());

        state.round_over = false;
        state.deck.discard.clear();
        state.deck.cards.clear();
        let moves = listed(&state, 0);
        assert!(has_discard(&moves, king));
        assert!(!moves.contains(&Action::TakeDiscard));
        assert!(!moves.contains(&Action::PushDiscard));
        assert!(!moves.contains(&Action::DrawFromDeck));
        assert_eq!(moves.len(), 1);
        assert!(listed(&state, 2).is_empty());
    }

    /// A wild still forms a set. The visited plays match the oracle as a multiset.
    #[test]
    fn test_visit_legal_moves_keeps_a_wild_set() {
        let joker = card(1, Suit::None, Rank::Joker);
        let four_h = card(2, Suit::Hearts, Rank::Four);
        let four_s = card(3, Suit::Spades, Rank::Four);
        let five_h = card(4, Suit::Hearts, Rank::Five);
        let five_s = card(5, Suit::Spades, Rank::Five);
        let five_c = card(6, Suit::Clubs, Rank::Five);
        let state = play_table(1, vec![joker, four_h, four_s, five_h, five_s, five_c]);
        let plays = played(&state);
        assert_same_multiset(&plays, &oracle_plays(&state));
        assert!(plays.iter().any(|action| meld_has(action, joker.id)));
    }

    /// Ace-low and ace-high runs stay. An ace wrap is not a run.
    #[test]
    fn test_visit_legal_moves_keeps_ace_low_and_ace_high_runs() {
        let low = play_table(
            2,
            vec![
                card(1, Suit::Hearts, Rank::Ace),
                card(2, Suit::Clubs, Rank::Two),
                card(3, Suit::Hearts, Rank::Three),
                card(4, Suit::Hearts, Rank::Four),
                card(5, Suit::Spades, Rank::Nine),
                card(6, Suit::Hearts, Rank::Nine),
                card(7, Suit::Diamonds, Rank::Nine),
            ],
        );
        let low_plays = played(&low);
        assert_same_multiset(&low_plays, &oracle_plays(&low));
        assert!(low_plays.iter().any(|action| meld_has(action, 1)));

        let high = play_table(
            2,
            vec![
                card(11, Suit::Hearts, Rank::Jack),
                card(12, Suit::Hearts, Rank::Queen),
                card(13, Suit::Hearts, Rank::King),
                card(14, Suit::Hearts, Rank::Ace),
                card(15, Suit::Spades, Rank::Nine),
                card(16, Suit::Hearts, Rank::Nine),
                card(17, Suit::Diamonds, Rank::Nine),
            ],
        );
        let high_plays = played(&high);
        assert_same_multiset(&high_plays, &oracle_plays(&high));
        assert!(high_plays.iter().any(|action| meld_has(action, 14)));

        let wrap = play_table(
            2,
            vec![
                card(21, Suit::Hearts, Rank::King),
                card(22, Suit::Hearts, Rank::Ace),
                card(23, Suit::Clubs, Rank::Two),
                card(24, Suit::Hearts, Rank::Three),
                card(25, Suit::Spades, Rank::Nine),
                card(26, Suit::Hearts, Rank::Nine),
                card(27, Suit::Diamonds, Rank::Nine),
            ],
        );
        assert!(played(&wrap).is_empty());
        assert!(oracle_plays(&wrap).is_empty());
    }

    /// Two fours of hearts stay distinct. One card id is not listed twice inside a meld.
    #[test]
    fn test_visit_legal_moves_keeps_duplicate_cards() {
        let state = play_table(
            1,
            vec![
                card(1, Suit::Hearts, Rank::Four),
                card(2, Suit::Hearts, Rank::Four),
                card(3, Suit::Spades, Rank::Four),
                card(4, Suit::Hearts, Rank::Five),
                card(5, Suit::Spades, Rank::Five),
                card(6, Suit::Clubs, Rank::Five),
            ],
        );
        let plays = played(&state);
        assert_same_multiset(&plays, &oracle_plays(&state));
        assert!(plays
            .iter()
            .any(|action| { meld_has(action, 1) && meld_has(action, 2) }));
        assert!(plays.iter().all(|action| !meld_repeats_an_id(action)));
    }

    /// Four wilds are still one set beside a natural set.
    #[test]
    fn test_visit_legal_moves_keeps_an_all_wild_meld() {
        let state = play_table(
            1,
            vec![
                card(1, Suit::None, Rank::Joker),
                card(2, Suit::None, Rank::Joker),
                card(3, Suit::Hearts, Rank::Two),
                card(4, Suit::Spades, Rank::Two),
                card(5, Suit::Hearts, Rank::Eight),
                card(6, Suit::Spades, Rank::Eight),
                card(7, Suit::Clubs, Rank::Eight),
            ],
        );
        let plays = played(&state);
        assert_same_multiset(&plays, &oracle_plays(&state));
        assert!(plays.iter().any(|action| {
            matches!(action, Action::PlayMeld(melds) if melds.iter().any(|meld| {
                meld.len() == 4 && meld.iter().all(|card| card.is_wild())
            }))
        }));
    }

    /// Round 3 asks for two runs. One run is not enough. Both runs are visited.
    #[test]
    fn test_visit_legal_moves_keeps_repeated_meld_types() {
        let state = play_table(
            3,
            vec![
                card(1, Suit::Hearts, Rank::Four),
                card(2, Suit::Hearts, Rank::Five),
                card(3, Suit::Hearts, Rank::Six),
                card(4, Suit::Hearts, Rank::Seven),
                card(5, Suit::Spades, Rank::Four),
                card(6, Suit::Spades, Rank::Five),
                card(7, Suit::Spades, Rank::Six),
                card(8, Suit::Spades, Rank::Seven),
            ],
        );
        let plays = played(&state);
        assert_same_multiset(&plays, &oracle_plays(&state));
        assert!(plays
            .iter()
            .all(|action| matches!(action, Action::PlayMeld(melds) if melds.len() == 2)));
        assert!(plays
            .iter()
            .any(|action| meld_has(action, 1) && meld_has(action, 5)));
    }

    /// Hits that use a wild, an ace, or a second copy match the oracle multiset.
    #[test]
    fn test_visit_legal_moves_keeps_wild_ace_and_duplicate_hits() {
        let joker = card(1, Suit::None, Rank::Joker);
        let ace = card(2, Suit::Hearts, Rank::Ace);
        let eight_a = card(3, Suit::Hearts, Rank::Eight);
        let eight_b = card(4, Suit::Diamonds, Rank::Eight);
        let mut state = play_table(1, vec![joker, ace, eight_a, eight_b]);
        state.players[0].is_on_board = true;
        state.board = vec![
            vec![
                card(10, Suit::Spades, Rank::Eight),
                card(11, Suit::Clubs, Rank::Eight),
                card(12, Suit::Diamonds, Rank::Joker),
            ],
            vec![
                card(13, Suit::Hearts, Rank::Jack),
                card(14, Suit::Hearts, Rank::Queen),
                card(15, Suit::Hearts, Rank::King),
            ],
            vec![
                card(16, Suit::Clubs, Rank::Two),
                card(17, Suit::Hearts, Rank::Three),
                card(18, Suit::Hearts, Rank::Four),
            ],
        ];
        let hits = hit(&state);
        assert_same_multiset(&hits, &oracle_hits(&state));
        assert!(hits.iter().any(|action| meld_has(action, joker.id)));
        assert!(hits.iter().any(|action| meld_has(action, ace.id)));
        assert!(hits.iter().any(|action| meld_has(action, eight_a.id)));
        assert!(hits.iter().any(|action| meld_has(action, eight_b.id)));
    }

    /// One kind is that kind only, in the same order the full list has it.
    #[test]
    fn test_visit_legal_kind_matches_the_full_list_in_order() {
        let state = play_table(
            1,
            vec![
                card(1, Suit::Hearts, Rank::Four),
                card(2, Suit::Spades, Rank::Four),
                card(3, Suit::Clubs, Rank::Four),
                card(4, Suit::Hearts, Rank::Five),
                card(5, Suit::Spades, Rank::Five),
                card(6, Suit::Clubs, Rank::Five),
                card(7, Suit::Spades, Rank::King),
            ],
        );
        let all = generate_legal_moves(&state, 0);
        let plays = kinded(&state, LegalKind::Play);
        let hits = kinded(&state, LegalKind::Hit);
        assert_eq!(
            plays,
            all.iter()
                .filter(|action| matches!(action, Action::PlayMeld(_)))
                .cloned()
                .collect::<Vec<_>>()
        );
        assert!(hits.is_empty());
        assert!(plays
            .iter()
            .all(|action| matches!(action, Action::PlayMeld(_))));
        assert_eq!(plays.len(), oracle_plays(&state).len());
    }

    fn play_table(round: u8, hand: Vec<Card>) -> GameState {
        let mut state = table(
            hand,
            vec![card(90, Suit::Clubs, Rank::King)],
            vec![card(91, Suit::Hearts, Rank::Three)],
        );
        state.round_number = round;
        state
    }

    fn played(state: &GameState) -> Vec<Action> {
        kinded(state, LegalKind::Play)
    }

    fn hit(state: &GameState) -> Vec<Action> {
        kinded(state, LegalKind::Hit)
    }

    fn kinded(state: &GameState, kind: LegalKind) -> Vec<Action> {
        let before = state.clone();
        let mut found = Vec::new();
        visit_legal_kind(state, 0, kind, &mut |action| found.push(action));
        same_table(state, &before);
        found
    }

    fn oracle_plays(state: &GameState) -> Vec<Action> {
        let groups = match state.round_number {
            1 | 2 | 3 | 5 => 2,
            4 => 3,
            _ => return Vec::new(),
        };
        if state.players[0].is_on_board {
            return Vec::new();
        }
        let cards: Vec<Card> = state.players[0]
            .hand
            .iter()
            .copied()
            .filter(|card| card_can_be_played(card, state.turn_counter))
            .collect();
        let mut found = Vec::new();
        let mut built = vec![Vec::new(); groups];
        assign_plays(state.round_number, &cards, 0, &mut built, &mut found);
        found
            .into_iter()
            .filter(|action| accepts(state, 0, action.clone()))
            .collect()
    }

    fn oracle_hits(state: &GameState) -> Vec<Action> {
        hit_moves(state, 0)
    }

    fn assert_same_multiset(live: &[Action], oracle: &[Action]) {
        assert_eq!(sorted_actions(live), sorted_actions(oracle));
        assert_eq!(live.len(), oracle.len());
    }

    fn sorted_actions(actions: &[Action]) -> Vec<String> {
        let mut keys: Vec<String> = actions.iter().map(|action| format!("{action:?}")).collect();
        keys.sort();
        keys
    }

    fn meld_has(action: &Action, id: u32) -> bool {
        match action {
            Action::PlayMeld(melds) => melds.iter().flatten().any(|card| card.id == id),
            Action::HitMeld(hits) => hits
                .iter()
                .flat_map(|hit| &hit.cards)
                .any(|card| card.id == id),
            _ => false,
        }
    }

    fn meld_repeats_an_id(action: &Action) -> bool {
        let Action::PlayMeld(melds) = action else {
            return false;
        };
        for meld in melds {
            let mut ids: Vec<u32> = meld.iter().map(|card| card.id).collect();
            let before = ids.len();
            ids.sort_unstable();
            ids.dedup();
            if ids.len() != before {
                return true;
            }
        }
        false
    }
}
