# Architecture update: command design

## 7.1 Command design (implemented 2025-12)

**Design decision:** Commands are implemented as an **enum with data** rather
than a trait, for the following reasons:

- **Exhaustive matching**: All command variants can be matched exhaustively.
- **Serialization**: Enums are serializable for macro recording.
- **Consistency**: Matches the Action enum design.

Commands capture:

- Pre-conditions (can the command execute?)
- Context (what data is needed?)
- Inverse (how to undo?)
- Name (human-readable description)

The relationship is:

- Actions represent user intent
- Commands capture concrete mutations with undo data
- DocOps are atomic invertible document mutations
