# AI Accounts Hub CLI

Standalone `aah` command for AI Accounts Hub. You can use it without installing the desktop app.

## Install

```bash
npm install -g @murongg/aah-cli
```

The npm package has its own standalone CLI version and downloads the matching prebuilt native binary from the `cli-vX.Y.Z` GitHub Release during installation.

Publishing from GitHub Actions requires the repository secret `NPM_TOKEN` to be an npm Automation token when the npm account has publish 2FA enabled. A regular publish token will fail with `EOTP` because CI cannot provide a one-time password.

## Usage

Open the interactive TUI:

```bash
aah tui
```

TUI shortcuts:

- `up/down` or `j/k`: select account
- `Enter`: switch to the selected account
- `r`: refresh quota
- `1/2/3/a`: filter Codex, Claude, Gemini, or all
- `q` or `Esc`: quit

Run script-friendly commands:

```bash
aah add --provider codex
aah list
aah current
aah refresh
aah switch --provider codex user@example.com
```

`aah add --provider ...` starts the provider's login flow, stores the account in the managed account pool, and leaves the current active CLI account unchanged.

Filter by provider:

```bash
aah list --provider codex
aah current --provider claude
aah refresh --provider gemini
```

Use JSON output:

```bash
aah list --json
aah current --json
```

## Relay Mode

If you need a local Codex-compatible endpoint, you can enable the built-in relay.

- The relay is off by default
- It currently serves Codex routes only
- It only binds to `127.0.0.1`
- The default base URL is `http://127.0.0.1:8765/codex`
- The desktop app and CLI share the same running relay instance

Manage the local relay:

```bash
aah relay status
aah relay start --port 8765
aah relay stop
aah relay set-port 9876
```

- `aah relay start [--port ...]` persists `enabled=true` and makes sure the relay is running
- `aah relay stop` persists `enabled=false` and stops the current relay instance

Use a custom data directory:

```bash
aah --data-dir ~/.ai-accounts-hub list
```

By default, the CLI stores and reads managed account data from `~/.ai-accounts-hub`. The desktop app uses the same directory, and startup migrates older desktop app data into this shared directory by default.

Relay settings are stored in `~/.ai-accounts-hub/settings.json`. The shared relay runtime registry lives at `~/.ai-accounts-hub/relay/runtime.json`. If you pass `--data-dir`, both paths move under that directory.
