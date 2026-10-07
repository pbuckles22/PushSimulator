//! Action handlers: take, push, meld, hit, steal, discard (Epics 1.3–1.7).

use crate::card::Card;
use crate::deck::{Deck, TurnDraw};
use crate::player::Player;

/// A player choice during the draw.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move the discard pile's top card into the player's hand.
    TakeDiscard,
    /// Move the discard top to the next player, who also draws a penalty. The actor then draws.
    PushDiscard,
}

impl Action {
    /// Runs this action for the player at `actor_index`.
    ///
    /// [`Action::TakeDiscard`] moves the discard top onto the end of that player's hand.
    /// Cards under that top stay. The draw pile stays.
    ///
    /// [`Action::PushDiscard`] moves the discard top onto the end of the next player's hand.
    /// That player then draws one card. The actor draws the next card to start the turn.
    /// Cards under the pushed top stay.
    pub fn apply(self, players: &mut [Player], actor_index: usize, deck: &mut Deck) {
        match self {
            Self::TakeDiscard => {
                let card = deck.discard.pop().expect("take discard needs a top card");
                players[actor_index].hand.push(card);
            }
            Self::PushDiscard => push_discard(players, actor_index, deck),
        }
    }
}

/// The next seat takes the discard and one draw-pile card. The actor draws after that.
fn push_discard(players: &mut [Player], actor_index: usize, deck: &mut Deck) {
    assert!(players.len() >= 2, "Push is played with 2 or more players");
    let next_index = (actor_index + 1) % players.len();
    let discarded = deck.discard.pop().expect("push discard needs a top card");
    let penalty = draw_one(deck, "push penalty");
    let start = draw_one(deck, "push turn");
    players[next_index].hand.push(discarded);
    players[next_index].hand.push(penalty);
    players[actor_index].hand.push(start);
}

fn draw_one(deck: &mut Deck, why: &str) -> Card {
    match deck.draw() {
        TurnDraw::One(card) | TurnDraw::LastCard(card) => card,
        other => panic!("{why} expected one card, got {other:?}"),
    }
}

#[cfg(test)]
mod tests {
    use crate::actions::Action;
    use crate::card::{Card, Rank, Suit};
    use crate::deck::Deck;
    use crate::player::Player;

    #[test]
    fn test_take_discard() {
        let mut deck = Deck::new();
        let draw_before = deck.cards.clone();
        let kept = Card {
            id: 200,
            suit: Suit::Hearts,
            rank: Rank::Four,
            locked_until_turn: 2,
        };
        let under = Card {
            id: 201,
            suit: Suit::Clubs,
            rank: Rank::Five,
            locked_until_turn: 0,
        };
        let top = Card {
            id: 202,
            suit: Suit::Spades,
            rank: Rank::Ace,
            locked_until_turn: 4,
        };
        let mut players = vec![Player::new(1, 0)];
        players[0].hand = vec![kept];
        deck.discard = vec![under, top];

        Action::TakeDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(players[0].hand, vec![kept, top]);
        assert_eq!(deck.discard, vec![under]);
        assert_eq!(deck.cards, draw_before);
        assert_eq!(players[0].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert!(!players[0].is_on_board);
        assert_eq!(players[0].id, 1);
        assert_eq!(players[0].seat_index, 0);
    }

    fn card(id: u32, suit: Suit, rank: Rank) -> Card {
        Card {
            id,
            suit,
            rank,
            locked_until_turn: 0,
        }
    }

    /// Player 1 at seat 0, Player 2 at seat 1. The draw pile's last card is the penalty.
    fn push_setup() -> (Vec<Player>, Deck, Card, Card, Card, Card) {
        let kept = card(200, Suit::Hearts, Rank::Four);
        let other = card(201, Suit::Clubs, Rank::Five);
        let under = card(202, Suit::Diamonds, Rank::Six);
        let top = card(203, Suit::Spades, Rank::Ace);
        let actor_draw = card(204, Suit::Hearts, Rank::Jack);
        let penalty = card(205, Suit::Clubs, Rank::Three);
        let mut players = vec![Player::new(1, 0), Player::new(2, 1)];
        players[0].hand = vec![kept];
        players[1].hand = vec![other];
        let mut deck = Deck::new();
        deck.cards = vec![actor_draw, penalty];
        deck.discard = vec![under, top];
        (players, deck, under, top, penalty, actor_draw)
    }

    #[test]
    fn test_push_discard_moves_card() {
        let (mut players, mut deck, under, top, _, _) = push_setup();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(players[1].hand[0].id, 201);
        assert_eq!(players[1].hand[1], top);
        assert_eq!(deck.discard, vec![under]);
        assert!(players[0].hand.iter().all(|card| card.id != top.id));
        assert_eq!(players[0].id, 1);
        assert_eq!(players[1].id, 2);
        assert_eq!(players[0].points, 0);
        assert_eq!(players[1].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert_eq!(players[1].total_score, 0);
    }

    #[test]
    fn test_push_discard_penalty_draw() {
        let (mut players, mut deck, _, top, penalty, _) = push_setup();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(
            players[1].hand,
            vec![card(201, Suit::Clubs, Rank::Five), top, penalty]
        );
        assert_eq!(penalty.get_penalty_value(), 5);
        assert_eq!(
            players[1].calculate_hand_penalty(),
            5 + top.get_penalty_value() + penalty.get_penalty_value()
        );
        assert_eq!(players[1].points, 0);
        assert_eq!(players[1].total_score, 0);
    }

    #[test]
    fn test_push_turn_advancement() {
        let (mut players, mut deck, _, _, penalty, actor_draw) = push_setup();
        let draw_before = deck.cards.len();

        Action::PushDiscard.apply(&mut players, 0, &mut deck);

        assert_eq!(
            players[0].hand,
            vec![card(200, Suit::Hearts, Rank::Four), actor_draw]
        );
        assert_eq!(actor_draw.get_penalty_value(), 10);
        assert_eq!(deck.cards.len(), draw_before - 2);
        assert!(deck
            .cards
            .iter()
            .all(|card| card.id != penalty.id && card.id != actor_draw.id));
        assert_eq!(players[0].points, 0);
        assert_eq!(players[0].total_score, 0);
        assert!(!players[0].is_on_board);
        assert!(!players[1].is_on_board);
    }
}
