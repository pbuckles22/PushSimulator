//! Every action the engine would accept for one seat.

use std::collections::{BTreeMap, HashSet};
use std::ops::ControlFlow;

use crate::actions::{validate_action, Action, MeldHit, WildSteal};
use crate::card::{Card, Rank, Suit};
use crate::deck::{one_card_draw_count, Deck};
use crate::game_state::{GameState, TurnPhase};
use crate::resolution::ActionResolution;
use crate::validation::{card_can_be_played, check_round_requirements, validate_run, validate_set};

/// What a legal-move visitor returns.
///
/// `()`, `true`, and [`ControlFlow::Continue`] keep walking.
/// `false` and [`ControlFlow::Break`] stop the walk.
pub trait VisitFlow {
    fn visit_flow(self) -> ControlFlow<()>;
}

impl VisitFlow for () {
    fn visit_flow(self) -> ControlFlow<()> {
        ControlFlow::Continue(())
    }
}

impl VisitFlow for bool {
    fn visit_flow(self) -> ControlFlow<()> {
        if self {
            ControlFlow::Continue(())
        } else {
            ControlFlow::Break(())
        }
    }
}

impl<B, C> VisitFlow for ControlFlow<B, C> {
    fn visit_flow(self) -> ControlFlow<()> {
        match self {
            ControlFlow::Continue(_) => ControlFlow::Continue(()),
            ControlFlow::Break(_) => ControlFlow::Break(()),
        }
    }
}

/// Search steps already taken. [`u32::MAX`] does not count and does not stop.
struct WalkLimit {
    nodes: u32,
    max_nodes: u32,
}

impl WalkLimit {
    fn new(max_nodes: u32) -> Self {
        Self {
            nodes: 0,
            max_nodes,
        }
    }

    fn tick(&mut self) -> ControlFlow<()> {
        if self.max_nodes == u32::MAX {
            return ControlFlow::Continue(());
        }
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.max_nodes {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    }
}

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
///
/// `visit` returns `()`, `true`, or [`ControlFlow::Continue`] to keep walking.
/// `false` or [`ControlFlow::Break`] stops this walk, including later kinds.
pub fn visit_legal_moves<V, R>(state: &GameState, actor: usize, visit: &mut V)
where
    V: FnMut(Action) -> R,
    R: VisitFlow,
{
    for kind in [
        LegalKind::Take,
        LegalKind::Push,
        LegalKind::Play,
        LegalKind::Hit,
        LegalKind::Steal,
        LegalKind::Discard,
        LegalKind::Draw,
    ] {
        if visit_legal_kind(state, actor, kind, visit).is_break() {
            return;
        }
    }
}

/// Calls `visit` for one family of actions, in the same order the full list has them.
///
/// A partial meld that already has two natural ranks and two natural suits is skipped.
/// Adding cards cannot make that group a set or a run. Wilds, ace-high runs, ace-low
/// runs, duplicate cards, all-wild melds, and a second meld of the same type stay.
/// The return is [`ControlFlow::Break`] when `visit` stops the walk.
pub fn visit_legal_kind<V, R>(
    state: &GameState,
    actor: usize,
    kind: LegalKind,
    visit: &mut V,
) -> ControlFlow<()>
where
    V: FnMut(Action) -> R,
    R: VisitFlow,
{
    visit_legal_kind_within(state, actor, kind, u32::MAX, visit)
}

/// Same walk as [`visit_legal_kind`], stopping after `max_nodes` search steps.
///
/// [`u32::MAX`] walks until `visit` stops or the family is finished. A walk that
/// has not emitted an action yet can still stop on this ceiling.
pub(crate) fn visit_legal_kind_within<V, R>(
    state: &GameState,
    actor: usize,
    kind: LegalKind,
    max_nodes: u32,
    visit: &mut V,
) -> ControlFlow<()>
where
    V: FnMut(Action) -> R,
    R: VisitFlow,
{
    if state.round_over || actor >= state.players.len() {
        return ControlFlow::Continue(());
    }
    let mut limit = WalkLimit::new(max_nodes);
    let mut adapted = |action: Action| visit(action).visit_flow();
    match kind {
        LegalKind::Take => {
            if !state.deck.discard.is_empty() && accepts(state, actor, Action::TakeDiscard) {
                adapted(Action::TakeDiscard)
            } else {
                ControlFlow::Continue(())
            }
        }
        LegalKind::Push => {
            if push_is_legal(&state.deck) {
                adapted(Action::PushDiscard)
            } else {
                ControlFlow::Continue(())
            }
        }
        LegalKind::Play => visit_plays(state, actor, &mut limit, &mut adapted),
        LegalKind::Hit => visit_hits(state, actor, &mut limit, &mut adapted),
        LegalKind::Steal => visit_steals(state, actor, &mut adapted),
        LegalKind::Discard => visit_discards(state, actor, &mut adapted),
        LegalKind::Draw => {
            // An empty shoe is still a draw. That draw ends the round.
            if state.turn_phase == TurnPhase::PenaltyDrawing
                && state.penalty_seat == Some(actor)
                && accepts(state, actor, Action::DrawFromDeck)
            {
                adapted(Action::DrawFromDeck)
            } else {
                ControlFlow::Continue(())
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

fn visit_plays<F>(
    state: &GameState,
    actor: usize,
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    if state.players[actor].is_on_board {
        return visit_free_plays(state, actor, limit, visit);
    }
    let groups = match state.round_number {
        1 | 2 | 3 | 5 => 2,
        4 => 3,
        _ => return ControlFlow::Continue(()),
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
        limit,
        visit,
    )
}

/// On the board, any valid set or run may be laid. Candidates are maximal sets
/// and runs from the playable hand. Each candidate is yielded alone. When there
/// are at most six, disjoint combinations are yielded too.
fn visit_free_plays<F>(
    state: &GameState,
    actor: usize,
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    let cards = playable_hand(state, actor);
    let candidates = free_meld_candidates(&cards);
    for meld in &candidates {
        limit.tick()?;
        let action = Action::PlayMeld(vec![meld.clone()]);
        if accepts(state, actor, action.clone()) {
            visit(action)?;
        }
    }
    if candidates.len() > 6 {
        return ControlFlow::Continue(());
    }
    let mut chosen = Vec::new();
    visit_disjoint_free_melds(state, actor, &candidates, 0, &mut chosen, limit, visit)
}

fn visit_disjoint_free_melds<F>(
    state: &GameState,
    actor: usize,
    candidates: &[Vec<Card>],
    index: usize,
    chosen: &mut Vec<Vec<Card>>,
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    limit.tick()?;
    if index == candidates.len() {
        if chosen.len() >= 2 {
            let action = Action::PlayMeld(chosen.clone());
            if accepts(state, actor, action.clone()) {
                return visit(action);
            }
        }
        return ControlFlow::Continue(());
    }
    visit_disjoint_free_melds(state, actor, candidates, index + 1, chosen, limit, visit)?;
    if chosen
        .iter()
        .any(|held| shares_a_card(held, &candidates[index]))
    {
        return ControlFlow::Continue(());
    }
    chosen.push(candidates[index].clone());
    let flow = visit_disjoint_free_melds(state, actor, candidates, index + 1, chosen, limit, visit);
    chosen.pop();
    flow
}

fn shares_a_card(left: &[Card], right: &[Card]) -> bool {
    left.iter()
        .any(|card| right.iter().any(|other| other.id == card.id))
}

fn free_meld_candidates(cards: &[Card]) -> Vec<Vec<Card>> {
    let mut found = Vec::new();
    found.extend(maximal_sets(cards));
    found.extend(maximal_runs(cards));
    found
}

fn maximal_sets(cards: &[Card]) -> Vec<Vec<Card>> {
    let wilds: Vec<Card> = cards
        .iter()
        .copied()
        .filter(|card| card.is_wild())
        .collect();
    let mut by_rank: Vec<(Rank, Vec<Card>)> = Vec::new();
    for card in cards {
        if card.is_wild() {
            continue;
        }
        if let Some((_, group)) = by_rank.iter_mut().find(|(rank, _)| *rank == card.rank) {
            group.push(*card);
        } else {
            by_rank.push((card.rank, vec![*card]));
        }
    }
    let mut found = Vec::new();
    for (_, group) in &by_rank {
        if group.len() >= 3 {
            if validate_set(group) {
                found.push(group.clone());
            }
            continue;
        }
        let need = 3 - group.len();
        if need <= wilds.len() {
            let mut meld = group.clone();
            meld.extend(wilds[..need].iter().copied());
            if validate_set(&meld) {
                found.push(meld);
            }
        }
    }
    if wilds.len() >= 3 {
        let meld = wilds[..3].to_vec();
        if validate_set(&meld) {
            found.push(meld);
        }
    }
    found
}

fn maximal_runs(cards: &[Card]) -> Vec<Vec<Card>> {
    let wilds: Vec<Card> = cards
        .iter()
        .copied()
        .filter(|card| card.is_wild())
        .collect();
    let mut found = Vec::new();
    for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades] {
        for ace_high in [false, true] {
            let mut by_rank: BTreeMap<u8, Card> = BTreeMap::new();
            for card in cards {
                if card.suit != suit || card.is_wild() {
                    continue;
                }
                let Some(value) = run_rank_value(card.rank, ace_high) else {
                    continue;
                };
                by_rank.entry(value).or_insert(*card);
            }
            if by_rank.is_empty() {
                continue;
            }
            let values: Vec<u8> = by_rank.keys().copied().collect();
            let mut start = 0;
            while start < values.len() {
                let mut end = start;
                let mut holes = 0usize;
                while end + 1 < values.len() {
                    let gap = usize::from(values[end + 1] - values[end] - 1);
                    if holes + gap > wilds.len() {
                        break;
                    }
                    holes += gap;
                    end += 1;
                }
                let span = usize::from(values[end] - values[start] + 1);
                let naturals = end - start + 1;
                let holes = span - naturals;
                if span >= 4 && holes <= wilds.len() {
                    let mut meld: Vec<Card> = values[start..=end]
                        .iter()
                        .map(|value| by_rank[value])
                        .collect();
                    meld.extend(wilds[..holes].iter().copied());
                    if validate_run(&meld) {
                        found.push(meld);
                    }
                } else if naturals + wilds.len() >= 4 {
                    let need = 4 - naturals;
                    if need > holes && need <= wilds.len() {
                        let mut meld: Vec<Card> = values[start..=end]
                            .iter()
                            .map(|value| by_rank[value])
                            .collect();
                        meld.extend(wilds[..need].iter().copied());
                        if validate_run(&meld) {
                            found.push(meld);
                        }
                    }
                }
                start = end + 1;
            }
        }
    }
    found
}

fn run_rank_value(rank: Rank, ace_high: bool) -> Option<u8> {
    Some(match rank {
        Rank::Ace => {
            if ace_high {
                14
            } else {
                1
            }
        }
        Rank::Three => 3,
        Rank::Four => 4,
        Rank::Five => 5,
        Rank::Six => 6,
        Rank::Seven => 7,
        Rank::Eight => 8,
        Rank::Nine => 9,
        Rank::Ten => 10,
        Rank::Jack => 11,
        Rank::Queen => 12,
        Rank::King => 13,
        Rank::Two | Rank::Joker => return None,
    })
}

fn assign_plays<F>(
    state: &GameState,
    actor: usize,
    round: u8,
    cards: &[Card],
    index: usize,
    groups: &mut [Vec<Card>],
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    limit.tick()?;
    if index == cards.len() {
        if groups.iter().all(|group| !group.is_empty()) && check_round_requirements(round, groups) {
            let action = Action::PlayMeld(groups.to_vec());
            if accepts(state, actor, action.clone()) {
                return visit(action);
            }
        }
        return ControlFlow::Continue(());
    }
    assign_plays(state, actor, round, cards, index + 1, groups, limit, visit)?;
    for slot in 0..groups.len() {
        groups[slot].push(cards[index]);
        let flow = if !group_is_dead(&groups[slot]) {
            assign_plays(state, actor, round, cards, index + 1, groups, limit, visit)
        } else {
            ControlFlow::Continue(())
        };
        groups[slot].pop();
        flow?;
    }
    ControlFlow::Continue(())
}

fn visit_hits<F>(
    state: &GameState,
    actor: usize,
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    if !state.players[actor].is_on_board || state.board.is_empty() {
        return ControlFlow::Continue(());
    }
    let hand = playable_hand(state, actor);
    if hand.is_empty() {
        return ControlFlow::Continue(());
    }
    let mut additions = Vec::with_capacity(state.board.len());
    for meld in &state.board {
        let mut found = Vec::new();
        let mut extra = Vec::new();
        collect_additions(meld, &hand, 0, &mut extra, &mut found, limit)?;
        additions.push(found);
    }
    let mut hits = Vec::new();
    let mut used = HashSet::new();
    combine_hits(
        state, actor, 0, &additions, &mut used, &mut hits, limit, visit,
    )
}

fn collect_additions(
    meld: &[Card],
    hand: &[Card],
    index: usize,
    extra: &mut Vec<Card>,
    found: &mut Vec<Vec<Card>>,
    limit: &mut WalkLimit,
) -> ControlFlow<()> {
    limit.tick()?;
    if index == hand.len() {
        if extra.is_empty() {
            return ControlFlow::Continue(());
        }
        if addition_fits(meld, extra) {
            found.push(extra.clone());
        }
        return ControlFlow::Continue(());
    }
    collect_additions(meld, hand, index + 1, extra, found, limit)?;
    extra.push(hand[index]);
    let flow = if !group_is_dead_with(meld, extra) {
        collect_additions(meld, hand, index + 1, extra, found, limit)
    } else {
        ControlFlow::Continue(())
    };
    extra.pop();
    flow
}

fn combine_hits<F>(
    state: &GameState,
    actor: usize,
    meld_index: usize,
    additions: &[Vec<Vec<Card>>],
    used: &mut HashSet<u32>,
    hits: &mut Vec<MeldHit>,
    limit: &mut WalkLimit,
    visit: &mut F,
) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    limit.tick()?;
    if meld_index == additions.len() {
        if !hits.is_empty() {
            let action = Action::HitMeld(hits.clone());
            if accepts(state, actor, action.clone()) {
                return visit(action);
            }
        }
        return ControlFlow::Continue(());
    }
    combine_hits(
        state,
        actor,
        meld_index + 1,
        additions,
        used,
        hits,
        limit,
        visit,
    )?;
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
        let flow = combine_hits(
            state,
            actor,
            meld_index + 1,
            additions,
            used,
            hits,
            limit,
            visit,
        );
        hits.pop();
        for card in extra {
            used.remove(&card.id);
        }
        flow?;
    }
    ControlFlow::Continue(())
}

fn visit_steals<F>(state: &GameState, actor: usize, visit: &mut F) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    if !state.players[actor].is_on_board {
        return ControlFlow::Continue(());
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
                    visit(action)?;
                }
            }
        }
    }
    ControlFlow::Continue(())
}

fn visit_discards<F>(state: &GameState, actor: usize, visit: &mut F) -> ControlFlow<()>
where
    F: FnMut(Action) -> ControlFlow<()>,
{
    for card in &state.players[actor].hand {
        let action = Action::DiscardCard(*card);
        if accepts(state, actor, action.clone()) {
            visit(action)?;
        }
    }
    ControlFlow::Continue(())
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

    /// Both piles are empty. The penalty seat is still offered the draw that ends the round.
    /// The other seat is not. After that draw, the round is over and the draw is gone.
    #[test]
    fn test_generate_legal_moves_lists_draw_when_both_piles_are_empty() {
        let eight = card(1, Suit::Diamonds, Rank::Eight);
        let mut state = table(vec![eight], vec![card(3, Suit::Clubs, Rank::Three)], vec![]);
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Clubs, Rank::Eight),
            card(12, Suit::Hearts, Rank::Eight),
        ]];
        state.deck.discard.clear();
        state.deck.cards.clear();
        state.turn_phase = TurnPhase::PenaltyDrawing;
        state.penalty_seat = Some(0);

        let moves = listed(&state, 0);
        assert!(moves.contains(&Action::DrawFromDeck));
        assert!(!has_discard(&moves, eight));

        let other = listed(&state, 1);
        assert!(!other.contains(&Action::DrawFromDeck));
        assert!(has_discard(&other, card(3, Suit::Clubs, Rank::Three)));

        assert!(state.apply(Action::DrawFromDeck, 0));
        assert_eq!(state.players[0].hand, vec![eight]);
        assert!(state.deck.cards.is_empty());
        assert!(state.deck.discard.is_empty());
        assert_eq!(state.turn_phase, TurnPhase::PenaltyDrawing);
        assert_eq!(state.penalty_seat, Some(0));
        assert!(!listed(&state, 0).contains(&Action::DrawFromDeck));
        assert!(state.round_over);
    }

    /// On the board, a run of 4 in the hand is offered as PlayMeld.
    #[test]
    fn test_generate_legal_moves_includes_free_melds() {
        let run = vec![
            card(1, Suit::Hearts, Rank::Four),
            card(2, Suit::Hearts, Rank::Five),
            card(3, Suit::Hearts, Rank::Six),
            card(4, Suit::Hearts, Rank::Seven),
        ];
        let king = card(5, Suit::Spades, Rank::King);
        let mut state = table(
            vec![run[0], run[1], run[2], run[3], king],
            vec![card(6, Suit::Clubs, Rank::Three)],
            vec![
                card(7, Suit::Diamonds, Rank::Jack),
                card(8, Suit::Clubs, Rank::Ace),
            ],
        );
        state.players[0].is_on_board = true;
        state.board = vec![vec![
            card(10, Suit::Spades, Rank::Eight),
            card(11, Suit::Clubs, Rank::Eight),
            card(12, Suit::Hearts, Rank::Eight),
        ]];

        let moves = listed(&state, 0);

        assert!(has_play(&moves, &[run]));
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

    /// A node ceiling stops a play walk before it lists the plays a full walk lists.
    #[test]
    fn test_visit_node_ceiling_stops_a_play_walk_before_it_finishes() {
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
        let full = played(&state);
        assert!(!full.is_empty());
        let mut capped = Vec::new();
        let flow = super::visit_legal_kind_within(&state, 0, LegalKind::Play, 0, &mut |action| {
            capped.push(action);
            true
        });
        assert!(flow.is_break());
        assert!(capped.is_empty());
        assert!(capped.len() < full.len());
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
        let _ = visit_legal_kind(state, 0, kind, &mut |action| found.push(action));
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
