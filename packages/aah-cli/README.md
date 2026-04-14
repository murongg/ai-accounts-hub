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
aah list
aah current
aah refresh
aah switch --provider codex user@example.com
```

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

Use a custom data directory:

```bash
aah --data-dir ~/.ai-accounts-hub list
```

By default, the CLI stores and reads managed account data from `~/.ai-accounts-hub`. The desktop app uses the same directory, and startup migrates older desktop app data into this shared directory by default.
