# Command Dispatch: Implementation

**Parent**: [0-1-2-command-dispatch.md](../0-1-2-command-dispatch.md)

## Step 1: Create `src/model/command.rs`

Define the core structures. See
[snippets/step1-core-structures.rs](snippets/step1-core-structures.rs).

## Step 2: Implement Command methods

See [snippets/step2-command-impl.rs](snippets/step2-command-impl.rs).

## Step 3: Implement CommandInverse

See [snippets/step3-command-inverse.rs](snippets/step3-command-inverse.rs).

## Step 4: Implement helper functions

See [snippets/step4-helper-functions.rs](snippets/step4-helper-functions.rs).

## Step 5: Implement command preparation from Action

See [snippets/step5-prepare-command.rs](snippets/step5-prepare-command.rs).

## Step 6: Update `src/model/mod.rs`

```rust
pub mod command;

pub use command::{Command, CommandInverse, DeletedShape, UserError, prepare_command};
```
