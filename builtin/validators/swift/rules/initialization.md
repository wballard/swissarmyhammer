---
name: initialization
description: No slow work in init, every stored property set at init time, property observers and callback closures that call a named method
---

# Swift Initialization

- **`init` does no slow work.** A caller expects `init` to return at once, and an `init` that is not `throws` has no way to report a failure. Open a database, read a large file, or make a network request in an explicit `start()`/`load()` the caller calls. DON'T: `init(path: String) { rows = Database.open(path).allRows() }`. DO: `init(path: String) { self.path = path }` beside `func load() throws { rows = try Database.open(path).allRows() }`.
- **`init` sets every stored property.** A property the type fills in later holds `nil`, or a wrong value, for every caller between `init` and the fill, and each reader must then test for a state the type should not have. DON'T: `private var session: Session?` that a later `configure(token:)` assigns. DO: `private let session: Session` that `init(token:)` assigns. swiftlint's `implicitly_unwrapped_optional` decides the `Session!` spelling of this defect and reports nothing about a plain `Optional` a later method assigns, so this bullet reads that shape.
- **A property observer that branches calls a named method.** A `didSet`/`willSet` body holding an `if`, a `switch` or a loop buries the logic inside a declaration, where no caller can name it and no test can call it. DON'T: `var count: Int = 0 { didSet { if count == 0 { … } else if count > Badge.cap { … } else { … } } }`. DO: `var count: Int = 0 { didSet { countDidChange() } }` beside `private func countDidChange() { … }`.
- **A callback closure that branches calls a named method.** The same defect, reached through a completion handler. DON'T: `loader { loaded in var kept: [Photo] = []; for photo in loaded where !photo.isPlaceholder { kept.append(photo) } … }`. DO: `loader { loaded in self.photosDidLoad(loaded) }` beside `private func photosDidLoad(_ loaded: [Photo]) { … }`. swiftlint's `closure_body_length` decides the LENGTH of a closure, at 250 lines, and reports nothing about a short closure that branches, so this bullet reads the branch.
