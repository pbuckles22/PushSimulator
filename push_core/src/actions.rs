//! Action handlers: take, push, meld, hit, steal, discard (Epics 1.3–1.7).

use crate::deck::Deck;
use crate::player::Player;

/// A player choice. Taking the discard is the first one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    /// Move the discard pile's top card into the player's hand.
    TakeDiscard,
}

impl Action {
    /// Runs this action.
    ///
    /// [`Action::TakeDiscard`] moves the discard top onto the end of the hand.
    /// Cards under that top stay. The draw pile stays.
    pub fn apply(self, player: &mut Player, deck: &mut Deck) {
        match self {
            Self::TakeDiscard => {
                let card = deck.discard.pop().expect("take discard needs a top card");
                player.hand.push(card);
            }
        }
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
        let mut player = Player::new(1, 0);
        player.hand = vec![kept];
        deck.discard = vec![under, top];

        Action::TakeDiscard.apply(&mut player, &mut deck);

        assert_eq!(player.hand, vec![kept, top]);
        assert_eq!(deck.discard, vec![under]);
        assert_eq!(deck.cards, draw_before);
        assert_eq!(player.points, 0);
        assert_eq!(player.total_score, 0);
        assert!(!player.is_on_board);
        assert_eq!(player.id, 1);
        assert_eq!(player.seat_index, 0);
    }
}
