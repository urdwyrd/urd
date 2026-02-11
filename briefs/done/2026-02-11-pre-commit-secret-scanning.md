# Brief: Pre-Commit Secret Scanning Hooks

## Execution Record

> **Instructions for AI:** Before this brief is moved to `briefs/done/`, fill in this section completely. Be specific and honest — this is the project's permanent record of what happened.

**Date completed:** 2026-02-11
**Status:** Complete

### What was done
- Installed Husky v9 as a workspace root devDependency (`pnpm add -D husky --workspace-root`)
- Ran `pnpm exec husky init` — created `.husky/` directory and added `prepare` script to root `package.json`
- Replaced `.husky/pre-commit` contents with `gitleaks git --pre-commit --staged --verbose`
- Created `.gitleaks.toml` at repo root with default rules extended and allowlist for briefs and the config file itself
- Ran baseline scan (`gitleaks git --verbose`) against full repo history — 31 commits scanned, no leaks found
- Created `.github/workflows/secret-scan.yml` — runs `gitleaks/gitleaks-action@v2` on every push to `main` and on all PRs, with `fetch-depth: 0` for full history scanning
- Verified hook blocks secrets: staged a file containing `aws_secret_access_key = wJalrXUtnFEMI/K7MDENG/bPxRfiCYSECRETKEYXX` — gitleaks detected it (`generic-api-key` rule, entropy 4.67) and returned exit code 1
- Verified hook allows clean commits: staged normal project files — gitleaks returned exit code 0

### Deviations from brief
- **gitleaks installed via winget (Step 1) was done manually by the user** before execution began — the brief was written to include this as an implementation step, but it was already complete
- **Verification used `aws_secret_access_key` instead of `AKIAIOSFODNN7EXAMPLE`** — the brief's suggested test key (`AKIAIOSFODNN7EXAMPLE`) is a well-known AWS example key that gitleaks deliberately allowlists. Used a more realistic secret pattern instead

### Issues encountered
- **gitleaks not on bash PATH after winget install** — winget added the package directory to the Windows user PATH, but the current shell session (started before installation) hadn't picked it up. Git hooks will use the system PATH at execution time, so this only affected the verification step in this session. Resolved by manually exporting the path for testing

### Notes for next brief
- The pre-commit hook requires gitleaks to be installed as a system binary on each developer's machine — document this in a future CONTRIBUTING.md or the README
- If gitleaks is not installed, the pre-commit hook will fail with "command not found" and block the commit — this is the safe default (fail closed), but could be surprising for new contributors
- Husky is now in place and ready for additional hooks (lint-staged, commitlint) when those tools are configured
- The CI secret scanning workflow runs on all pushes to `main` and all PRs — it is not path-filtered, unlike the deploy workflow

---

## Context

The Urd monorepo already has two layers of secret protection:

1. **`.gitignore`** covers `.env`, `.env.*`, `.dev.vars`, and `.wrangler/`
2. **GitHub's server-side secret scanning** is active on the public repo and will flag known secret patterns after they're pushed

Both are reactive — they help after the fact, but neither prevents a developer (or AI assistant) from accidentally committing a secret in the first place. A **pre-commit hook** closes this gap by scanning staged changes locally before they ever reach a commit.

This was flagged as a priority in both the scaffold brief and the deployment brief's execution records, and is listed in the deployment brief's future considerations.

### Current secrets in the project

| Secret | Location | Risk if leaked |
|--------|----------|----------------|
| `CLOUDFLARE_API_TOKEN` | GitHub Actions secrets only | Attacker could deploy to or delete Pages projects |
| `CLOUDFLARE_ACCOUNT_ID` | GitHub Actions secrets only | Low on its own, but aids targeted attacks |

As the project grows (database credentials, API keys for future services, signing keys for the compiler), the attack surface increases. Better to have the hook in place now, before the next secret enters the workflow.

---

## Strategy

Use **gitleaks** as a Git pre-commit hook, managed via **Husky** (a lightweight Git hooks manager for Node/pnpm projects).

### Why gitleaks

- Fast, single-binary scanner written in Go — no runtime dependencies
- Excellent default ruleset covering 150+ secret patterns (AWS, Cloudflare, GitHub tokens, private keys, generic high-entropy strings, etc.)
- Supports a `.gitleaks.toml` config file for allowlisting known false positives
- Active maintenance with regular rule updates
- Works identically on Windows, macOS, and Linux
- Can also run in CI as an additional safety net

### Why Husky (not lefthook, simple-git-hooks, or manual hooks)

- Already the most common Git hooks manager in the pnpm/Node ecosystem
- Zero-config for basic use — `husky init` scaffolds everything
- Hooks are versioned shell scripts in `.husky/` — transparent, no magic
- Works with pnpm workspaces out of the box via `prepare` script
- If the team later needs lint-staged or other pre-commit tasks, Husky is already in place

### Why not secretlint

- secretlint is a valid alternative, but it's a Node-based tool with heavier dependencies
- gitleaks is faster (native binary) and has broader pattern coverage
- For a monorepo that will eventually include Rust code, a language-agnostic binary is preferable

---

## Implementation Steps

### Step 1: Install gitleaks

gitleaks is a standalone binary. Install it per platform:

**Windows (winget):**
```bash
winget install gitleaks
```

**macOS (Homebrew):**
```bash
brew install gitleaks
```

**Linux (or CI):**
```bash
# Download from GitHub releases
# https://github.com/gitleaks/gitleaks/releases
# Or use the go install method:
go install github.com/gitleaks/gitleaks/v8@latest
```

**Verify installation:**
```bash
gitleaks version
```

> **Note:** gitleaks must be installed on each developer's machine. It is not an npm dependency — it's a system tool, like `git` itself. Document the installation requirement in the project README or a CONTRIBUTING.md file.

### Step 2: Install Husky

From the repo root:

```bash
pnpm add -D husky --workspace-root
```

Then initialise Husky:

```bash
pnpm exec husky init
```

This will:
- Create a `.husky/` directory
- Add a `prepare` script to root `package.json` that runs `husky` (ensures hooks are installed after `pnpm install`)
- Create a sample `.husky/pre-commit` file

### Step 3: Create the pre-commit hook

Replace the contents of `.husky/pre-commit` with:

```bash
gitleaks git --pre-commit --staged --verbose
```

This scans only the staged diff (not the entire repo history), which keeps it fast.

**Expected behaviour:**
- If no secrets are found → exit 0, commit proceeds
- If a secret pattern is detected → exit 1, commit is blocked, and the offending file/line is printed

### Step 4: Create `.gitleaks.toml` config

Create `.gitleaks.toml` at the repo root with an initial configuration:

```toml
# gitleaks configuration
# https://github.com/gitleaks/gitleaks

title = "urd repo gitleaks config"

# Extend the default rules (don't replace them)
[extend]
useDefault = true

# Allowlist paths that are known safe
[allowlist]
  paths = [
    '''briefs/.*\.md''',       # Briefs may contain example secret patterns in documentation
    '''\.gitleaks\.toml''',    # This config file itself
  ]
```

> **Important:** The allowlist should be as narrow as possible. Only add paths when a genuine false positive is confirmed. Never allowlist broad patterns like `*.js` or `sites/**`.

### Step 5: Run a baseline scan

Before committing the hook, run gitleaks against the full repo to confirm there are no existing secrets:

```bash
gitleaks git --verbose
```

If this reports findings, investigate each one:
- **True positive:** The secret must be rotated immediately (even if removed from the repo, it's in Git history)
- **False positive:** Add a specific allowlist entry to `.gitleaks.toml`

### Step 6: Add CI secret scanning (optional but recommended)

Add a new job to `.github/workflows/deploy.yml` (or create a separate workflow) that runs gitleaks on every push:

```yaml
  secret-scan:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Run gitleaks
        uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

This acts as a safety net for commits made without the local hook (e.g., from GitHub's web editor, or a machine without gitleaks installed).

> **Decision for executor:** This step can be a separate workflow file (`.github/workflows/secret-scan.yml`) or added as a job in the existing deploy workflow. A separate file is cleaner — it runs on all pushes regardless of path, unlike the deploy workflow which only triggers on `sites/urd.dev/**` changes.

---

## Files to Create/Modify

| File | Action | Contains secrets? |
|------|--------|-------------------|
| `.husky/pre-commit` | Create | No |
| `.gitleaks.toml` | Create | No |
| `package.json` (root) | Modify — add `husky` devDependency and `prepare` script | No |
| `.github/workflows/secret-scan.yml` | Create (if Step 6 is included) | No |

---

## Verification

After implementation, verify all of the following:

1. **Hook triggers on commit:** Stage any file, run `git commit` — you should see gitleaks output in the terminal
2. **Hook blocks a real secret:** Create a test file with a fake AWS key pattern (e.g., `AKIAIOSFODNN7EXAMPLE`), stage it, and attempt to commit — it should be blocked
3. **Hook allows clean commits:** Stage a normal code change and commit — it should succeed without delay
4. **Baseline scan is clean:** `gitleaks git --verbose` reports no findings (or only documented false positives)
5. **CI scan runs (if Step 6):** Push a commit and verify the secret-scan job appears in GitHub Actions
6. **`pnpm install` sets up hooks:** Clone the repo fresh (or delete `.husky/_/`) and run `pnpm install` — the pre-commit hook should be automatically installed

---

## What NOT To Do

- Do not install gitleaks as an npm package — use the native binary for speed and reliability
- Do not use `--no-verify` to bypass the hook (and do not document it as a workaround)
- Do not add broad allowlist patterns — each exception should be specific and justified
- Do not scan the entire Git history on every commit — use `--staged` for pre-commit speed
- Do not add lint-staged, prettier, or other pre-commit tasks in this brief — keep scope focused on secret scanning only

---

## Future Considerations

- **lint-staged integration:** Once ESLint and/or Prettier are configured, Husky is already in place to run lint-staged as an additional pre-commit step
- **Commit message linting:** Husky can also run commitlint on a `commit-msg` hook to enforce conventional commits
- **gitleaks in PR checks:** The CI action can be configured to post inline comments on PRs when secrets are detected, rather than just failing the check
- **Custom rules:** If the project introduces its own secret formats (e.g., Urd API keys with a specific prefix), add custom regex rules to `.gitleaks.toml`
