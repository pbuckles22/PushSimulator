//! Every action the engine would accept for one seat.

use std::collections::HashSet;

use crate::actions::{Action, MeldHit, WildSteal};
use crate::card::Card;
use crate::deck::{Deck, TurnDraw};
use crate::game_state::{GameState, TurnPhase};
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// Actions [`GameState::apply`] would accept for `actor_index` on this table.
///
/// The hand is checked against the board: a hit has to leave a set or a run,
/// a steal has to replace a wild, and an off-board discard of a card that fits
/// is left out. The table stays as it was.
pub fn generate_legal_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    if state.round_over || actor_index >= state.players.len() {
        return Vec::new();
    }

    let mut moves = Vec::new();
    if !state.deck.discard.is_empty() && accepts(state, actor_index, Action::TakeDiscard) {
        moves.push(Action::TakeDiscard);
    }
    if push_is_legal(&state.deck) {
        moves.push(Action::PushDiscard);
    }
    moves.extend(play_moves(state, actor_index));
    moves.extend(hit_moves(state, actor_index));
    moves.extend(steal_moves(state, actor_index));
    moves.extend(discard_moves(state, actor_index));
    if state.turn_phase == TurnPhase::PenaltyDrawing
        && state.penalty_seat == Some(actor_index)
        && accepts(state, actor_index, Action::DrawFromDeck)
    {
        moves.push(Action::DrawFromDeck);
    }
    moves
}

fn accepts(state: &GameState, actor_index: usize, action: Action) -> bool {
    let mut trial = state.clone();
    trial.apply(action, actor_index)
}

fn push_is_legal(deck: &Deck) -> bool {
    if deck.discard.is_empty() {
        return false;
    }
    let mut trial = deck.clone();
    trial.discard.pop();
    one_card(&mut trial) && one_card(&mut trial)
}

fn one_card(deck: &mut Deck) -> bool {
    matches!(deck.draw(), TurnDraw::One(_) | TurnDraw::LastCard(_))
}

fn playable_hand(state: &GameState, actor_index: usize) -> Vec<Card> {
    state.players[actor_index]
        .hand
        .iter()
        .copied()
        .filter(|card| card_can_be_played(card, state.turn_counter))
        .collect()
}

fn play_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    if state.players[actor_index].is_on_board {
        return Vec::new();
    }
    let groups = match state.round_number {
        1 | 2 | 3 | 5 => 2,
        4 => 3,
        _ => return Vec::new(),
    };
    let cards = playable_hand(state, actor_index);
    let mut built = vec![Vec::new(); groups];
    let mut found = Vec::new();
    assign_plays(state.round_number, &cards, 0, &mut built, &mut found);
    found
        .into_iter()
        .filter(|action| accepts(state, actor_index, action.clone()))
        .collect()
}

fn assign_plays(
    round: u8,
    cards: &[Card],
    index: usize,
    groups: &mut [Vec<Card>],
    found: &mut Vec<Action>,
) {
    if index == cards.len() {
        if groups.iter().all(|group| !group.is_empty()) && check_round_requirements(round, groups) {
            found.push(Action::PlayMeld(groups.to_vec()));
        }
        return;
    }
    assign_plays(round, cards, index + 1, groups, found);
    for slot in 0..groups.len() {
        groups[slot].push(cards[index]);
        assign_plays(round, cards, index + 1, groups, found);
        groups[slot].pop();
    }
}

fn hit_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    if !state.players[actor_index].is_on_board || state.board.is_empty() {
        return Vec::new();
    }
    let hand = playable_hand(state, actor_index);
    if hand.is_empty() {
        return Vec::new();
    }
    let additions: Vec<Vec<Vec<Card>>> = state
        .board
        .iter()
        .map(|meld| additions(meld, &hand))
        .collect();
    let mut found = Vec::new();
    let mut hits = Vec::new();
    let mut used = HashSet::new();
    combine_hits(0, &additions, &mut used, &mut hits, &mut found);
    found
        .into_iter()
        .filter(|action| accepts(state, actor_index, action.clone()))
        .collect()
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
        let mut with = meld.to_vec();
        with.extend(extra.iter().copied());
        if validate_set(&with) || validate_run(&with) {
            found.push(extra.clone());
        }
        return;
    }
    collect_additions(meld, hand, index + 1, extra, found);
    extra.push(hand[index]);
    collect_additions(meld, hand, index + 1, extra, found);
    extra.pop();
}

fn combine_hits(
    meld_index: usize,
    additions: &[Vec<Vec<Card>>],
    used: &mut HashSet<u32>,
    hits: &mut Vec<MeldHit>,
    found: &mut Vec<Action>,
) {
    if meld_index == additions.len() {
        if !hits.is_empty() {
            found.push(Action::HitMeld(hits.clone()));
        }
        return;
    }
    combine_hits(meld_index + 1, additions, used, hits, found);
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
        combine_hits(meld_index + 1, additions, used, hits, found);
        hits.pop();
        for card in extra {
            used.remove(&card.id);
        }
    }
}

fn steal_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    if !state.players[actor_index].is_on_board {
        return Vec::new();
    }
    let mut found = Vec::new();
    for (meld_index, meld) in state.board.iter().enumerate() {
        for wild in meld.iter().copied().filter(|card| card.is_wild()) {
            for natural in state.players[actor_index]
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
                if accepts(state, actor_index, action.clone()) {
                    found.push(action);
                }
            }
        }
    }
    found
}

fn discard_moves(state: &GameState, actor_index: usize) -> Vec<Action> {
    state.players[actor_index]
        .hand
        .iter()
        .copied()
        .filter_map(|card| {
            let action = Action::DiscardCard(card);
            accepts(state, actor_index, action.clone()).then_some(action)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::generate_legal_moves;
    use crate::actions::{Action, MeldHit, WildSteal};
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
    use crate::game_state::{GameState, TurnPhase};
    use crate::player::Player;

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
}
