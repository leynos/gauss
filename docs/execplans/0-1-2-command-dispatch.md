# Execution Plan: 0.1.2 Implement Command Dispatch

**Status**: Complete **Roadmap reference**: `docs/roadmap.md` §0.1.2 **Depends
on**: 0.1.1 (Action enum — complete)

## Documents

This execution plan is split into the following sections:

| Document                                                   | Contents                                 |
| ---------------------------------------------------------- | ---------------------------------------- |
| [Overview](0-1-2-command-dispatch/overview.md)             | Design decisions, rationale, integration |
| [Implementation](0-1-2-command-dispatch/implementation.md) | Steps 1–6: core command code             |
| [Testing](0-1-2-command-dispatch/testing.md)               | Steps 7–12: tests and quality gates      |

## Code Snippets

Large code blocks are stored in the
[snippets/](0-1-2-command-dispatch/snippets/) directory:

| Snippet                                                                                        | Description              |
| ---------------------------------------------------------------------------------------------- | ------------------------ |
| [command-enum.rs](0-1-2-command-dispatch/snippets/command-enum.rs)                             | Command enum definition  |
| [deleted-shape.rs](0-1-2-command-dispatch/snippets/deleted-shape.rs)                           | DeletedShape struct      |
| [step1-core-structures.rs](0-1-2-command-dispatch/snippets/step1-core-structures.rs)           | Core structures          |
| [step2-command-impl.rs](0-1-2-command-dispatch/snippets/step2-command-impl.rs)                 | Command impl             |
| [step3-command-inverse.rs](0-1-2-command-dispatch/snippets/step3-command-inverse.rs)           | CommandInverse           |
| [step4-helper-functions.rs](0-1-2-command-dispatch/snippets/step4-helper-functions.rs)         | Helper functions         |
| [step5-prepare-command.rs](0-1-2-command-dispatch/snippets/step5-prepare-command.rs)           | prepare_command()        |
| [step7-unit-tests.rs](0-1-2-command-dispatch/snippets/step7-unit-tests.rs)                     | Unit tests               |
| [step8-command.feature](0-1-2-command-dispatch/snippets/step8-command.feature)                 | BDD feature file         |
| [step8-command-bdd.rs](0-1-2-command-dispatch/snippets/step8-command-bdd.rs)                   | BDD step implementations |
| [step9-gpui-integration.rs](0-1-2-command-dispatch/snippets/step9-gpui-integration.rs)         | GPUI integration tests   |
| [step11-architecture-update.md](0-1-2-command-dispatch/snippets/step11-architecture-update.md) | Architecture doc update  |

## Quick Summary

Commands are concrete, undoable state changes that bridge user intent (Actions)
to atomic document mutations (DocOps). See the
[overview](0-1-2-command-dispatch/overview.md) for design rationale.

```text
Action (user intent)       e.g., DeleteSelection
   │
   ▼  dispatch()
Command (undoable mutation) e.g., DeleteSelectionCommand { ids: [...] }
   │
   ▼  apply()
DocChange / DocOp          e.g., RemoveShape { index, shape }
```
