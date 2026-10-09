import CoreTransferable
import Foundation
import UniformTypeIdentifiers

/// Where the drag lands. A new meld is one play. A meld index is one hit.
public enum DropTarget: Equatable {
    case newMeld
    case meld(Int)
}

/// Cards in the drag. The hand card with that id is the one that moves.
public struct CardDrag: Equatable, Codable, Transferable {
    public var cards: [BoardCard]

    public init(cards: [BoardCard]) {
        self.cards = cards
    }

    public static var transferRepresentation: some TransferRepresentation {
        CodableRepresentation(contentType: .json)
    }
}

/// One hit onto one meld already on the board.
public struct MeldDrop: Equatable {
    public var meldIndex: Int
    public var cards: [BoardCard]

    public init(meldIndex: Int, cards: [BoardCard]) {
        self.meldIndex = meldIndex
        self.cards = cards
    }
}

/// The action a drop asks the engine to run.
public enum MoveIntent: Equatable {
    case playMeld([[BoardCard]])
    case hitMeld([MeldDrop])
}

/// What the engine answered. An accept is the table the engine returned.
public enum DropAnswer: Equatable {
    case accepted(HeldCards)
    case gameError
    case refused
}

/// The hand plus the board a drop reads and returns. The screen reads the picture's hand.
public struct HeldCards: Equatable {
    public var hand: [BoardCard]
    public var board: [[BoardCard]]

    public init(hand: [BoardCard], board: [[BoardCard]]) {
        self.hand = hand
        self.board = board
    }
}

/// `nil` when the drag is empty, repeats an id, or names a card that is not in the hand.
public func moveIntent(drag: CardDrag, onto target: DropTarget, hand: [BoardCard]) -> MoveIntent? {
    guard let cards = cardsInHand(drag, hand: hand) else {
        return nil
    }
    switch target {
    case .newMeld:
        return .playMeld([cards])
    case .meld(let index):
        return .hitMeld([MeldDrop(meldIndex: index, cards: cards)])
    }
}

/// Asks the engine, then keeps `table` on a game error or a refusal.
public func settleDrop(
    table: HeldCards,
    drag: CardDrag,
    onto target: DropTarget,
    perform: (MoveIntent) -> DropAnswer
) -> HeldCards {
    guard let intent = moveIntent(drag: drag, onto: target, hand: table.hand) else {
        return table
    }
    switch perform(intent) {
    case .accepted(let next):
        return next
    case .gameError, .refused:
        return table
    }
}

private func cardsInHand(_ drag: CardDrag, hand: [BoardCard]) -> [BoardCard]? {
    if drag.cards.isEmpty {
        return nil
    }
    var seen: Set<UInt32> = []
    var resolved: [BoardCard] = []
    for dragged in drag.cards {
        if seen.contains(dragged.id) {
            return nil
        }
        seen.insert(dragged.id)
        guard let held = hand.first(where: { $0.id == dragged.id }) else {
            return nil
        }
        resolved.append(held)
    }
    return resolved
}
