This crate allows one to generate logs about how tasks are scheduled, in order to generate a
profile of the CPU usage of the binary.

This crate leverages https://github.com/catapult-project/catapult/tree/11513e359cd60e369bbbd1f4f2ef648c1bccabd0/tracing

# Usage

First, import the traits:

```rust
use futures_diagnose_exec::{FutureExt as _, Future01Ext as _};
```

Then whenever you create a `Future`, append `.with_diagnostics("name")`. For example:

```rust
async_std::spawn(future.with_diagnostics("my-task-name"))
```

Set the environment variable `PROFILE_DIR` to a directory of your choice (e.g.
`profile`) and then, run your code. Files named `profile.<pid>.<num>.json` will
be generated in the directory set beforehand.

Then, open Chrome and go to the URL `chrome://tracing`, and load the `profile.json`.

# FAQ

- Chrome tells me `chrome://tracing` "can't be reached".

  Chromium shipped with recent Debian versions has the tracing feature disabled.
  See the [Debian bug
  report](https://bugs.debian.org/cgi-bin/bugreport.cgi?bug=922431) for details.
