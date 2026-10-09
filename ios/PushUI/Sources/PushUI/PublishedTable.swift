import Combine
import Foundation

/// `@Published` picture of the table. `load` reads the engine.
public final class PublishedTable: ObservableObject {
    @Published public private(set) var picture: TablePicture
    private let load: () -> TablePicture

    public init(load: @escaping () -> TablePicture) {
        self.load = load
        picture = load()
    }

    public var deckSizeText: String {
        "Deck size \(picture.deckSize)"
    }

    /// Reads the engine again. The picture changes only when the new value differs.
    public func reload() {
        let next = load()
        if next != picture {
            picture = next
        }
    }
}
