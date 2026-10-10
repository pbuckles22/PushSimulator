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

/// Calls `settleDrop`. A game error, a refusal, or a drag that never asks
/// returns the same picture. An accept keeps the version, the shoe, and the round,
/// and shows the hand and board the stand-in returned.
public func applyScreenDrop(
    picture: TablePicture,
    drag: CardDrag,
    onto target: DropTarget,
    engine: (MoveIntent) -> DropAnswer
) -> TablePicture {
    let held = HeldCards(hand: picture.hand, board: picture.board)
    let next = settleDrop(table: held, drag: drag, onto: target, perform: engine)
    if next.hand == held.hand && next.board == held.board {
        return picture
    }
    return TablePicture(
        version: picture.version,
        deckSize: picture.deckSize,
        roundNumber: picture.roundNumber,
        board: next.board,
        hand: next.hand
    )
}

/// A picture-only answer used when the screen has no engine.
/// A new meld takes an unlocked card. A drop onto a row is refused.
/// The live screen passes Rust instead.
public func standInAnswer(picture: TablePicture, intent: MoveIntent) -> DropAnswer {
    switch intent {
    case .playMeld(let groups):
        guard groups.count == 1, let cards = groups.first else {
            return .refused
        }
        if cards.contains(where: { $0.lockedUntilTurn > 0 }) {
            return .refused
        }
        let ids = Set(cards.map(\.id))
        let hand = picture.hand.filter { !ids.contains($0.id) }
        return .accepted(HeldCards(hand: hand, board: picture.board + [cards]))
    case .hitMeld:
        return .refused
    }
}

/// The bytes a hand lift puts on the drag. A drop reads them back as that same `CardDrag`.
public func handCardDragItemData(_ drag: CardDrag) throws -> Data {
    try JSONEncoder().encode(drag)
}

public func cardDrag(fromLiftedData data: Data) throws -> CardDrag {
    try JSONDecoder().decode(CardDrag.self, from: data)
}

/// One hand card, as JSON, for the in-app drop targets.
public func handCardItemProvider(for drag: CardDrag) -> NSItemProvider {
    let data = try? handCardDragItemData(drag)
    let provider = NSItemProvider()
    provider.registerDataRepresentation(
        forTypeIdentifier: UTType.json.identifier,
        visibility: .all
    ) { completion in
        completion(data, nil)
        return nil
    }
    return provider
}

/// The card in the air inside this app. A drop reads it at once. The bytes stay for a
/// drag that did not start here.
public enum LiftedHandCard {
    private static var held: CardDrag?

    public static func begin(_ drag: CardDrag) {
        held = drag
    }

    public static func take() -> CardDrag? {
        defer { held = nil }
        return held
    }

    public static func end() {
        held = nil
    }
}

/// A card lifted in this app drops now. Any other drag waits for its bytes.
@discardableResult
public func acceptLiftedCards(
    _ providers: [NSItemProvider],
    perform: @escaping (CardDrag) -> Bool
) -> Bool {
    if let drag = LiftedHandCard.take() {
        return perform(drag)
    }
    guard let provider = providers.first,
          provider.hasItemConformingToTypeIdentifier(UTType.json.identifier) else {
        return false
    }
    provider.loadDataRepresentation(forTypeIdentifier: UTType.json.identifier) { data, _ in
        guard let data, let drag = try? cardDrag(fromLiftedData: data) else {
            return
        }
        DispatchQueue.main.async {
            _ = perform(drag)
        }
    }
    return true
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
