//! Panic boundaries for an action. Legal and rejected moves are classified later.

use crate::actions::{Action, MeldHit, WildSteal};
use crate::card::Card;
use crate::deck::one_card_draw_count;
use crate::game_state::{GameState, TurnPhase};

/// What a legal action intends to do. Draws stay unresolved until commit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionPlan {
    TakeDiscard,
    PushDiscard,
    PlayMeld(Vec<Vec<Card>>),
    HitMeld(Vec<MeldHit>),
    StealWild(WildSteal),
    DiscardCard(Card),
    DrawFromDeck,
}

/// A rules refusal that still changes the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RejectionPlan {
    /// The card stays in the hand. This seat enters penalty drawing.
    StartPenaltyDraw { seat: usize },
    /// The action is refused and the table stays as it was.
    Unchanged,
}

/// A move [`GameState::apply`] would panic on. The table stays as it was.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameError {
    /// `actor` is not a seat at this table.
    ActorOutOfRange { actor: usize, seats: usize },
    /// Push needs two seats.
    FewerThanTwoPlayers { seats: usize },
    /// Take or push needs a discard top.
    EmptyDiscard,
    /// One-card draws left after the discard top is set aside.
    ///
    /// A [`crate::deck::TurnDraw::LastTwo`] split counts as zero.
    InsufficientDraws { available: usize },
}

/// Whether an action is legal, a penalized refusal, or nonsense.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionResolution {
    Accepted(ActionPlan),
    Rejected(RejectionPlan),
    Invalid(GameError),
}

/// `Some(Invalid(_))` when [`GameState::apply`] would panic. `None` when it would not.
///
/// A closed round returns before those panics. Push checks the seat count, then the
/// discard, then two one-card draws, then the actor. Take checks the discard, then
/// the actor. An empty hit does not index the actor. The table stays as it was.
pub fn invalid_resolution(
    state: &GameState,
    actor: usize,
    action: &Action,
) -> Option<ActionResolution> {
    panic_error(state, actor, action).map(ActionResolution::Invalid)
}

fn panic_error(state: &GameState, actor: usize, action: &Action) -> Option<GameError> {
    if state.round_over {
        return None;
    }
    match action {
        Action::TakeDiscard => take_error(state, actor),
        Action::PushDiscard => push_error(state, actor),
        Action::PlayMeld(_) | Action::StealWild(_) | Action::DiscardCard(_) => {
            actor_error(state, actor)
        }
        Action::HitMeld(hits) => {
            if hits.is_empty() {
                None
            } else {
                actor_error(state, actor)
            }
        }
        Action::DrawFromDeck => {
            if state.turn_phase != TurnPhase::PenaltyDrawing || state.penalty_seat != Some(actor) {
                None
            } else {
                actor_error(state, actor)
            }
        }
    }
}

fn take_error(state: &GameState, actor: usize) -> Option<GameError> {
    if state.deck.discard.is_empty() {
        return Some(GameError::EmptyDiscard);
    }
    actor_error(state, actor)
}

fn push_error(state: &GameState, actor: usize) -> Option<GameError> {
    let seats = state.players.len();
    if seats < 2 {
        return Some(GameError::FewerThanTwoPlayers { seats });
    }
    if state.deck.discard.is_empty() {
        return Some(GameError::EmptyDiscard);
    }
    let available = one_card_draw_count(state.deck.cards.len(), state.deck.discard.len() - 1);
    if available < 2 {
        return Some(GameError::InsufficientDraws { available });
    }
    actor_error(state, actor)
}

fn actor_error(state: &GameState, actor: usize) -> Option<GameError> {
    let seats = state.players.len();
    if actor >= seats {
        Some(GameError::ActorOutOfRange { actor, seats })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::{invalid_resolution, ActionPlan, ActionResolution, GameError, RejectionPlan};
    use crate::actions::{validate_action, Action, MeldHit, WildSteal};
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
    use crate::game_state::{GameState, TurnPhase};
    use crate::legacy_oracle::legacy_apply_with_rng;
    use crate::player::Player;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    /// A seat that is not at the table is Invalid once apply would index it.
    #[test]
    fn test_invalid_resolution_actor_out_of_range() {
        let state = seated(2, 4, 1);
        let out = GameError::ActorOutOfRange { actor: 2, seats: 2 };
        assert_eq!(resolved(&state, 2, &Action::TakeDiscard), invalid(out));
        assert_eq!(resolved(&state, 2, &Action::PushDiscard), invalid(out));
        assert_eq!(
            resolved(&state, 2, &Action::PlayMeld(Vec::new())),
            invalid(out)
        );
        assert_eq!(
            resolved(&state, 2, &Action::DiscardCard(sample(1))),
            invalid(out)
        );
        assert_eq!(
            resolved(&state, 2, &Action::StealWild(sample_steal())),
            invalid(out)
        );
        assert_eq!(
            resolved(&state, 2, &Action::HitMeld(vec![sample_hit()])),
            invalid(out)
        );
        assert_eq!(resolved(&state, 2, &Action::HitMeld(Vec::new())), None);
        assert_eq!(resolved(&state, 0, &Action::TakeDiscard), None);
        assert_eq!(resolved(&state, 1, &Action::PushDiscard), None);
    }

    /// Push panics on the seat count before it looks at the actor or the piles.
    #[test]
    fn test_invalid_resolution_fewer_than_two_players() {
        let one = seated(1, 4, 1);
        assert_eq!(
            resolved(&one, 0, &Action::PushDiscard),
            invalid(GameError::FewerThanTwoPlayers { seats: 1 })
        );
        assert_eq!(
            resolved(&one, 5, &Action::PushDiscard),
            invalid(GameError::FewerThanTwoPlayers { seats: 1 })
        );
        assert_eq!(resolved(&one, 0, &Action::TakeDiscard), None);

        let none = seated(0, 4, 1);
        assert_eq!(
            resolved(&none, 0, &Action::PushDiscard),
            invalid(GameError::FewerThanTwoPlayers { seats: 0 })
        );
    }

    /// Take and push pop the discard before they index the actor.
    #[test]
    fn test_invalid_resolution_empty_discard() {
        let state = seated(2, 4, 0);
        assert_eq!(
            resolved(&state, 0, &Action::TakeDiscard),
            invalid(GameError::EmptyDiscard)
        );
        assert_eq!(
            resolved(&state, 0, &Action::PushDiscard),
            invalid(GameError::EmptyDiscard)
        );
        assert_eq!(
            resolved(&state, 9, &Action::TakeDiscard),
            invalid(GameError::EmptyDiscard)
        );
        assert_eq!(
            resolved(&state, 9, &Action::PushDiscard),
            invalid(GameError::EmptyDiscard)
        );
        assert_eq!(resolved(&state, 0, &Action::PlayMeld(Vec::new())), None);
        assert_eq!(resolved(&state, 0, &Action::DrawFromDeck), None);
    }

    /// A push needs two one-card draws after the discard top is set aside.
    /// The count is the same one `Deck::has_draw_capacity` reports. LastTwo adds nothing.
    #[test]
    fn test_invalid_resolution_insufficient_draws() {
        let shapes = [
            (0, 1),
            (1, 1),
            (2, 1),
            (0, 2),
            (1, 2),
            (0, 3),
            (1, 3),
            (2, 3),
            (0, 4),
        ];
        for (draw, discard) in shapes {
            let state = seated(2, draw, discard);
            let available = draws_after_pop(&state);
            let got = resolved(&state, 0, &Action::PushDiscard);
            if available < 2 {
                assert_eq!(
                    got,
                    invalid(GameError::InsufficientDraws { available }),
                    "draw {draw}, discard {discard}"
                );
            } else {
                assert_eq!(got, None, "draw {draw}, discard {discard}");
            }
        }
    }

    /// A closed round returns before any panic. Penalty drawing indexes only that seat.
    #[test]
    fn test_invalid_resolution_closed_round_and_penalty_seat_do_not_invent_panics() {
        let mut closed = seated(1, 0, 0);
        closed.round_over = true;
        assert_eq!(resolved(&closed, 0, &Action::TakeDiscard), None);
        assert_eq!(resolved(&closed, 4, &Action::PushDiscard), None);

        let mut playing = seated(2, 4, 1);
        assert_eq!(resolved(&playing, 5, &Action::DrawFromDeck), None);

        playing.turn_phase = TurnPhase::PenaltyDrawing;
        playing.penalty_seat = Some(0);
        assert_eq!(resolved(&playing, 5, &Action::DrawFromDeck), None);

        playing.penalty_seat = Some(5);
        assert_eq!(
            resolved(&playing, 5, &Action::DrawFromDeck),
            invalid(GameError::ActorOutOfRange { actor: 5, seats: 2 })
        );
    }

    /// The check does not change the table.
    #[test]
    fn test_invalid_resolution_leaves_the_table() {
        let state = seated(2, 1, 1);
        let before = state.clone();
        assert_eq!(
            resolved(&state, 0, &Action::PushDiscard),
            invalid(GameError::InsufficientDraws { available: 1 })
        );
        assert_eq!(
            resolved(&state, 3, &Action::TakeDiscard),
            invalid(GameError::ActorOutOfRange { actor: 3, seats: 2 })
        );
        assert!(same_table(&state, &before));
    }

    /// Accepted, rejected, and invalid stay three different answers.
    #[test]
    fn test_action_resolution_keeps_accept_reject_and_invalid_apart() {
        let accepted = ActionResolution::Accepted(ActionPlan::TakeDiscard);
        let rejected = ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat: 0 });
        let invalid = ActionResolution::Invalid(GameError::EmptyDiscard);
        assert!(matches!(
            accepted,
            ActionResolution::Accepted(ActionPlan::TakeDiscard)
        ));
        assert!(matches!(
            rejected,
            ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat: 0 })
        ));
        assert!(matches!(
            invalid,
            ActionResolution::Invalid(GameError::EmptyDiscard)
        ));
        assert_ne!(resolution_kind(&accepted), resolution_kind(&rejected));
        assert_ne!(resolution_kind(&accepted), resolution_kind(&invalid));
        match ActionPlan::PushDiscard {
            ActionPlan::TakeDiscard
            | ActionPlan::PushDiscard
            | ActionPlan::PlayMeld(_)
            | ActionPlan::HitMeld(_)
            | ActionPlan::StealWild(_)
            | ActionPlan::DiscardCard(_)
            | ActionPlan::DrawFromDeck => {}
        }
    }

    fn resolved(state: &GameState, actor: usize, action: &Action) -> Option<ActionResolution> {
        invalid_resolution(state, actor, action)
    }

    fn invalid(error: GameError) -> Option<ActionResolution> {
        Some(ActionResolution::Invalid(error))
    }

    fn resolution_kind(resolution: &ActionResolution) -> u8 {
        match resolution {
            ActionResolution::Accepted(_) => 0,
            ActionResolution::Rejected(_) => 1,
            ActionResolution::Invalid(_) => 2,
        }
    }

    fn seated(seats: usize, draw: usize, discard: usize) -> GameState {
        let players = (0..seats)
            .map(|seat| Player::new((seat + 1) as u32, seat as u32))
            .collect();
        let cards = (0..draw as u32).map(sample).collect();
        let thrown = (0..discard as u32).map(|id| sample(1_000 + id)).collect();
        GameState::new(
            players,
            Deck {
                cards,
                discard: thrown,
            },
        )
    }

    fn draws_after_pop(state: &GameState) -> usize {
        let mut deck = state.deck.clone();
        deck.discard.pop().expect("the discard has a top card");
        let mut available = 0;
        while deck.has_draw_capacity(available + 1) {
            available += 1;
        }
        available
    }

    fn sample(id: u32) -> Card {
        Card {
            id,
            suit: Suit::Hearts,
            rank: Rank::Five,
            locked_until_turn: 0,
        }
    }

    fn sample_hit() -> MeldHit {
        MeldHit {
            meld_index: 0,
            cards: vec![sample(7)],
        }
    }

    fn sample_steal() -> WildSteal {
        WildSteal {
            meld_index: 0,
            wild: sample(8),
            natural: sample(9),
        }
    }

    /// A legal take is accepted and does not change the table until apply.
    /// Apply then returns true. An empty discard is Invalid and apply leaves the table.
    #[test]
    fn test_validate_action_accepts_a_take_and_invalid_apply_does_not_mutate() {
        let mut state = seated(2, 4, 1);
        let top = *state
            .deck
            .discard
            .last()
            .expect("the discard has a top card");
        let before = state.clone();
        assert_eq!(
            validate_action(&state, 0, &Action::TakeDiscard),
            ActionResolution::Accepted(ActionPlan::TakeDiscard)
        );
        assert!(same_table(&state, &before));
        assert!(state.apply_with_rng(Action::TakeDiscard, 0, &mut StdRng::seed_from_u64(1)));
        assert_eq!(state.players[0].hand, vec![top]);
        assert_eq!(state.drawn_card_id, Some(top.id));
        assert!(state.deck.discard.is_empty());

        let mut empty = seated(2, 4, 0);
        let before = empty.clone();
        assert_eq!(
            validate_action(&empty, 0, &Action::TakeDiscard),
            ActionResolution::Invalid(GameError::EmptyDiscard)
        );
        assert!(!empty.apply_with_rng(Action::TakeDiscard, 0, &mut StdRng::seed_from_u64(1)));
        assert!(same_table(&empty, &before));
    }

    /// A push that cannot draw two cards is Invalid. Apply returns false and leaves the table.
    #[test]
    fn test_validate_action_invalid_push_does_not_mutate() {
        let mut short = seated(2, 1, 1);
        let before = short.clone();
        assert!(matches!(
            validate_action(&short, 0, &Action::PushDiscard),
            ActionResolution::Invalid(GameError::InsufficientDraws { available: 1 })
        ));
        assert!(!short.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(1)));
        assert!(same_table(&short, &before));

        let mut one_seat = seated(1, 4, 1);
        let before = one_seat.clone();
        assert_eq!(
            validate_action(&one_seat, 0, &Action::PushDiscard),
            ActionResolution::Invalid(GameError::FewerThanTwoPlayers { seats: 1 })
        );
        assert!(!one_seat.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(1)));
        assert!(same_table(&one_seat, &before));
    }

    /// An off-board discard that fits starts penalty drawing and returns false.
    /// The same discard while another seat already owns the penalty does not move that seat.
    #[test]
    fn test_validate_action_rejects_a_fitting_discard_into_penalty() {
        let (mut state, seven) = fitting_sevens();
        assert_eq!(
            validate_action(&state, 0, &Action::DiscardCard(seven)),
            ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat: 0 })
        );
        assert!(!state.apply_with_rng(
            Action::DiscardCard(seven),
            0,
            &mut StdRng::seed_from_u64(1)
        ));
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert!(state.players[0].hand.contains(&seven));

        let (mut other, seven) = fitting_sevens();
        other.turn_phase = TurnPhase::PenaltyDrawing;
        other.penalty_seat = Some(1);
        let before = other.clone();
        assert_eq!(
            validate_action(&other, 0, &Action::DiscardCard(seven)),
            ActionResolution::Rejected(RejectionPlan::Unchanged)
        );
        assert!(!other.apply_with_rng(
            Action::DiscardCard(seven),
            0,
            &mut StdRng::seed_from_u64(1)
        ));
        assert!(same_table(&other, &before));
    }

    /// A closed round is a refusal, including a take that would otherwise panic.
    #[test]
    fn test_validate_action_closed_round_is_unchanged() {
        let mut state = seated(2, 4, 0);
        state.round_over = true;
        let before = state.clone();
        assert_eq!(
            validate_action(&state, 0, &Action::TakeDiscard),
            ActionResolution::Rejected(RejectionPlan::Unchanged)
        );
        assert!(!state.apply_with_rng(Action::TakeDiscard, 0, &mut StdRng::seed_from_u64(1)));
        assert!(same_table(&state, &before));
    }

    /// Accepted, rejected, and invalid commits match the frozen oracle on one seed.
    /// Panic inputs stay out of that comparison.
    #[test]
    fn test_validate_action_apply_with_rng_matches_legacy_oracle() {
        let mut live = seated(2, 0, 4);
        let mut oracle = live.clone();
        assert_eq!(
            validate_action(&live, 0, &Action::PushDiscard),
            ActionResolution::Accepted(ActionPlan::PushDiscard)
        );
        assert!(live.apply_with_rng(Action::PushDiscard, 0, &mut StdRng::seed_from_u64(3)));
        assert!(legacy_apply_with_rng(
            &mut oracle,
            Action::PushDiscard,
            0,
            &mut StdRng::seed_from_u64(3),
        ));
        assert_eq!(live.players, oracle.players);
        assert_eq!(live.deck, oracle.deck);
        assert_eq!(live.drawn_card_id, oracle.drawn_card_id);
        assert_eq!(live.turn_phase, oracle.turn_phase);
        assert_eq!(live.round_over, oracle.round_over);

        let (mut live, seven) = fitting_sevens();
        let mut oracle = live.clone();
        assert!(matches!(
            validate_action(&live, 0, &Action::DiscardCard(seven)),
            ActionResolution::Rejected(RejectionPlan::StartPenaltyDraw { seat: 0 })
        ));
        assert!(!live.apply_with_rng(Action::DiscardCard(seven), 0, &mut StdRng::seed_from_u64(1)));
        assert!(!legacy_apply_with_rng(
            &mut oracle,
            Action::DiscardCard(seven),
            0,
            &mut StdRng::seed_from_u64(1),
        ));
        assert_eq!(live.players, oracle.players);
        assert_eq!(live.deck, oracle.deck);
        assert_eq!(live.board, oracle.board);
        assert_eq!(live.turn_phase, oracle.turn_phase);
        assert_eq!(live.penalty_seat, oracle.penalty_seat);

        let mut live = round_one_sets();
        let melds = live.players[0].hand.clone();
        let action = Action::PlayMeld(vec![melds[..3].to_vec(), melds[3..6].to_vec()]);
        let mut oracle = live.clone();
        assert!(matches!(
            validate_action(&live, 0, &action),
            ActionResolution::Accepted(ActionPlan::PlayMeld(_))
        ));
        assert!(live.apply_with_rng(action.clone(), 0, &mut StdRng::seed_from_u64(1)));
        assert!(legacy_apply_with_rng(
            &mut oracle,
            action,
            0,
            &mut StdRng::seed_from_u64(1),
        ));
        assert_eq!(live.players, oracle.players);
        assert_eq!(live.board, oracle.board);
        assert!(live.round_over);
        assert_eq!(live.round_over, oracle.round_over);
    }

    fn fitting_sevens() -> (GameState, Card) {
        let board = vec![
            sample_rank(1, Suit::Hearts, Rank::Seven),
            sample_rank(2, Suit::Spades, Rank::Seven),
            sample_rank(3, Suit::Clubs, Rank::Seven),
        ];
        let seven = sample_rank(4, Suit::Diamonds, Rank::Seven);
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].hand = vec![seven, sample_rank(5, Suit::Spades, Rank::King)];
        let mut state = seated(2, 2, 1);
        state.players = players;
        state.board = vec![board];
        (state, seven)
    }

    fn round_one_sets() -> GameState {
        let hand = vec![
            sample_rank(10, Suit::Hearts, Rank::Four),
            sample_rank(11, Suit::Spades, Rank::Four),
            sample_rank(12, Suit::Clubs, Rank::Four),
            sample_rank(13, Suit::Hearts, Rank::Five),
            sample_rank(14, Suit::Spades, Rank::Five),
            sample_rank(15, Suit::Clubs, Rank::Five),
        ];
        let mut state = seated(2, 4, 1);
        state.players[0].hand = hand;
        state
    }

    fn sample_rank(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    fn same_table(left: &GameState, right: &GameState) -> bool {
        left.players == right.players
            && left.deck == right.deck
            && left.round_number == right.round_number
            && left.board == right.board
            && left.turn_counter == right.turn_counter
            && left.drawn_card_id == right.drawn_card_id
            && left.turn_phase == right.turn_phase
            && left.penalty_seat == right.penalty_seat
            && left.round_over == right.round_over
    }
}
