import PushShoe

extension CardValue {
    /// Their face for one natural rank and suit. The symbols must be the ones `CardFaceView` draws.
    init?(natural face: NaturalFace) {
        guard let rank = Rank(rawValue: face.rankRaw),
              let suit = Suit(rawValue: face.suitRaw) else {
            return nil
        }
        self.init(rank: rank, suit: suit)
        guard rank.symbol == face.rankSymbol,
              suit.symbol == face.suitSymbol,
              suit.systemImageName == face.systemImageName else {
            return nil
        }
    }
}
