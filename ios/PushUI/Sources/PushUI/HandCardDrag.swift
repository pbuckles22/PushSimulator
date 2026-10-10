#if os(iOS)
import SwiftUI
import UIKit

/// One hand card that lifts with a drag session. The session hides this view while the
/// card is in the air and shows it again when the session ends. The preview is a copy,
/// so the slot can be empty without taking the lifted picture with it.
struct HandCardDragSource: UIViewRepresentable {
    let card: BoardCard

    func makeCoordinator() -> Coordinator {
        Coordinator(card: card)
    }

    func makeUIView(context: Context) -> HandCardHost {
        let host = HandCardHost(card: card)
        let drag = UIDragInteraction(delegate: context.coordinator)
        drag.isEnabled = true
        host.addInteraction(drag)
        return host
    }

    func updateUIView(_ uiView: HandCardHost, context: Context) {
        context.coordinator.card = card
        uiView.update(card: card)
    }

    func sizeThatFits(_ proposal: ProposedViewSize, uiView: HandCardHost, context: Context) -> CGSize? {
        let width = proposal.width ?? 200
        let height = proposal.height ?? 120
        let fitted = uiView.fittedSize(in: CGSize(width: width, height: height))
        if fitted.width < 1 || fitted.height < 1 {
            return CGSize(width: 56, height: 80)
        }
        return fitted
    }

    final class Coordinator: NSObject, UIDragInteractionDelegate {
        var card: BoardCard

        init(card: BoardCard) {
            self.card = card
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            itemsForBeginning session: UIDragSession
        ) -> [UIDragItem] {
            let drag = CardDrag(cards: [card])
            LiftedHandCard.begin(drag)
            let item = UIDragItem(itemProvider: handCardItemProvider(for: drag))
            item.localObject = drag
            let face = card
            item.previewProvider = {
                handCardDragPreview(for: face, size: interaction.view?.bounds.size ?? CGSize(width: 56, height: 80))
            }
            return [item]
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            sessionIsRestrictedToDraggingApplication session: UIDragSession
        ) -> Bool {
            true
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            prefersFullSizePreviewsFor session: UIDragSession
        ) -> Bool {
            true
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            previewForLifting item: UIDragItem,
            session: UIDragSession
        ) -> UITargetedDragPreview? {
            guard let source = interaction.view, source.bounds.width > 1, source.bounds.height > 1 else {
                return nil
            }
            let face = HandCardPreview(card: card, size: source.bounds.size)
            let parameters = UIDragPreviewParameters()
            parameters.backgroundColor = .clear
            parameters.visiblePath = UIBezierPath(roundedRect: face.bounds, cornerRadius: 8)
            let target = UIDragPreviewTarget(
                container: source,
                center: CGPoint(x: source.bounds.midX, y: source.bounds.midY)
            )
            return UITargetedDragPreview(view: face, parameters: parameters, target: target)
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            previewForCancelling item: UIDragItem,
            withDefault defaultPreview: UITargetedDragPreview
        ) -> UITargetedDragPreview? {
            defaultPreview
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            item: UIDragItem,
            willAnimateCancelWith animator: UIDragAnimating
        ) {
            let alpha = HandSlotVisibility.afterDrag(of: card.id).alpha
            animator.addCompletion { _ in
                interaction.view?.alpha = alpha
            }
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            willAnimateLiftWith animator: UIDragAnimating,
            session: UIDragSession
        ) {
            let alpha = HandSlotVisibility.duringDrag(of: card.id).alpha
            animator.addAnimations {
                interaction.view?.alpha = alpha
            }
            animator.addCompletion { _ in
                interaction.view?.alpha = alpha
            }
        }

        func dragInteraction(
            _ interaction: UIDragInteraction,
            session: UIDragSession,
            didEndWith operation: UIDropOperation
        ) {
            interaction.view?.alpha = HandSlotVisibility.afterDrag(of: card.id).alpha
        }
    }
}

/// The picture that travels with the finger. Drawn here so the lift is the face,
/// not a blank snapshot of the slot.
final class HandCardPreview: UIView {
    init(card: BoardCard, size: CGSize) {
        super.init(frame: CGRect(origin: .zero, size: size))
        isOpaque = true
        backgroundColor = .white
        layer.cornerRadius = 8
        layer.borderWidth = 1
        layer.borderColor = UIColor.black.cgColor
        clipsToBounds = true
        let face = CardFace(card)
        let color: UIColor = face.isRed ? .red : .black
        let fonts = previewFonts(lineCount: handCardPreviewLines(card).count)
        let labels = zip(handCardPreviewLines(card), fonts).map { text, font -> UILabel in
            let label = UILabel()
            label.text = text
            label.font = font
            label.textColor = color
            label.textAlignment = .center
            label.sizeToFit()
            return label
        }
        let spacing: CGFloat = 4
        let total = labels.reduce(CGFloat(0)) { $0 + $1.bounds.height }
            + spacing * CGFloat(max(labels.count - 1, 0))
        var y = (bounds.height - total) / 2
        for label in labels {
            label.frame = CGRect(x: 0, y: y, width: bounds.width, height: label.bounds.height)
            addSubview(label)
            y += label.bounds.height + spacing
        }
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        nil
    }
}

private func previewFonts(lineCount: Int) -> [UIFont] {
    let title = UIFont.preferredFont(forTextStyle: .title2)
    let pip = UIFont.systemFont(ofSize: title.pointSize, weight: .bold)
    let suit = UIFont.preferredFont(forTextStyle: .title3)
    let locked = UIFont.preferredFont(forTextStyle: .caption2)
    if lineCount >= 3 {
        return [pip, suit, locked]
    }
    if lineCount == 2 {
        return [pip, suit]
    }
    return [pip]
}

private func handCardDragPreview(for card: BoardCard, size: CGSize) -> UIDragPreview {
    let face = HandCardPreview(card: card, size: size)
    let parameters = UIDragPreviewParameters()
    parameters.backgroundColor = .clear
    parameters.visiblePath = UIBezierPath(roundedRect: face.bounds, cornerRadius: 8)
    return UIDragPreview(view: face, parameters: parameters)
}

final class HandCardHost: UIView {
    private let hosting: UIHostingController<CardView>
    private var card: BoardCard

    init(card: BoardCard) {
        self.card = card
        hosting = UIHostingController(rootView: CardView(card: card))
        super.init(frame: .zero)
        backgroundColor = .clear
        hosting.view.backgroundColor = .clear
        hosting.view.isUserInteractionEnabled = false
        hosting.sizingOptions = .intrinsicContentSize
        addSubview(hosting.view)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) {
        nil
    }

    func update(card: BoardCard) {
        guard card != self.card else {
            return
        }
        self.card = card
        hosting.rootView = CardView(card: card)
    }

    func fittedSize(in proposal: CGSize) -> CGSize {
        hosting.sizeThatFits(in: proposal)
    }

    override func layoutSubviews() {
        super.layoutSubviews()
        hosting.view.frame = bounds
    }
}
#endif
