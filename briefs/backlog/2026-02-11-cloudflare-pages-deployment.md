# Brief: Deploy urd.dev to Cloudflare Pages

## Context

The urd.dev static site (Astro 5, built at `sites/urd.dev/dist`) needs a deployment pipeline to Cloudflare Pages. The project is a pnpm monorepo where non-site files (briefs, design docs, etc.) change frequently. We must avoid wasting builds on pushes that don't touch the site.

**Decision:** Use **GitHub Actions with path filtering** + **Wrangler CLI** to deploy. This avoids Cloudflare's git integration (which triggers a build on every push regardless of what changed) and keeps us well within the free tier (500 builds/month).

---

## Strategy

1. **No Cloudflare git integration** — we do not connect the repo to Cloudflare Pages via the dashboard. All deploys go through the Wrangler CLI, triggered by GitHub Actions.
2. **Path-filtered GitHub Action** — the workflow only runs when files under `sites/urd.dev/` actually change on `main`.
3. **Wrangler CLI deploys the pre-built `dist/`** — no build happens on Cloudflare's side, so there's no Cloudflare build minute cost either.
4. **Secrets stay in GitHub** — the Cloudflare API token lives in GitHub Actions secrets, never in the repo.

---

## Security Checklist

> Complete every item before the first deploy. This is not optional.

### Secrets That Must NEVER Be Committed

| Secret | Where It Lives | Purpose |
|--------|---------------|---------|
| `CLOUDFLARE_API_TOKEN` | GitHub repo → Settings → Secrets → Actions | Authenticates Wrangler CLI to Cloudflare |
| `CLOUDFLARE_ACCOUNT_ID` | GitHub repo → Settings → Secrets → Actions | Identifies your Cloudflare account |

### Pre-Deploy Safety

- [ ] **Verify `.gitignore` covers all secret-bearing files.** The following patterns must be present (they already are as of 2026-02-11):
  ```
  .env
  .env.*
  .dev.vars
  .wrangler/
  ```
- [ ] **Never create a `wrangler.toml` that contains account IDs or tokens.** The `wrangler.toml` in this brief contains only the project name and output directory — no secrets. Account ID and API token are passed via environment variables in CI.
- [ ] **Audit the build output before first deploy.** Run `pnpm build` locally, then inspect `sites/urd.dev/dist/` to confirm it contains only static HTML, CSS, JS, and assets — no `.env`, no config files, no source maps with embedded secrets.
- [ ] **Use a scoped Cloudflare API token.** When creating the token in the Cloudflare dashboard, grant **only** `Cloudflare Pages: Edit` permission. Do not use a global API key.
- [ ] **Enable GitHub's secret scanning** on the repo (Settings → Code security → Secret scanning). This catches accidental commits of API keys, tokens, and credentials. It should already be active for public repos.
- [ ] **Consider adding a pre-commit hook** for local secret scanning (e.g., `git-secrets`, `secretlint`, or `gitleaks`). This catches mistakes before they reach GitHub.

---

## Implementation Steps

### Step 1: Create the Cloudflare Pages Project (One-Time, Manual)

Do this in the Cloudflare dashboard — not via git integration:

1. Go to **Cloudflare Dashboard → Workers & Pages → Create → Pages**
2. Choose **"Direct Upload"** (not "Connect to Git")
3. Name the project: `urd-dev`
4. You don't need to upload anything yet — the first GitHub Actions run will handle it
5. After creation, go to **Custom domains** and add `urd.dev`
   - If your DNS is already on Cloudflare, it will auto-create the CNAME record
   - If not, add a CNAME record pointing `urd.dev` → `urd-dev.pages.dev`

### Step 2: Create a Scoped Cloudflare API Token (One-Time, Manual)

1. Go to **Cloudflare Dashboard → My Profile → API Tokens → Create Token**
2. Use **Custom token** with these permissions:
   - **Account** → Cloudflare Pages → **Edit**
3. Set **Account Resources** to your specific account (not "All accounts")
4. Create the token and **copy it immediately** (it won't be shown again)

### Step 3: Add Secrets to GitHub (One-Time, Manual)

1. Go to your repo on GitHub → **Settings → Secrets and variables → Actions**
2. Add two repository secrets:
   - `CLOUDFLARE_API_TOKEN` → paste the token from Step 2
   - `CLOUDFLARE_ACCOUNT_ID` → find this in Cloudflare Dashboard → Overview sidebar (right side)

### Step 4: Add `wrangler.toml` to the Repo

Create `wrangler.toml` at the **repo root**:

```toml
#:schema node_modules/wrangler/config-schema.json
name = "urd-dev"
pages_build_output_dir = "sites/urd.dev/dist"
```

**This file contains NO secrets.** It is safe to commit.

### Step 5: Create the GitHub Actions Workflow

Create `.github/workflows/deploy.yml`:

```yaml
name: Deploy to Cloudflare Pages

on:
  push:
    branches: [main]
    paths:
      - 'sites/urd.dev/**'

  # Allow manual deploy from the Actions tab
  workflow_dispatch:

jobs:
  deploy:
    runs-on: ubuntu-latest
    permissions:
      contents: read
      deployments: write

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup pnpm
        uses: pnpm/action-setup@v4

      - name: Setup Node
        uses: actions/setup-node@v4
        with:
          node-version: 20
          cache: pnpm

      - name: Install dependencies
        run: pnpm install --frozen-lockfile

      - name: Build
        run: pnpm build

      - name: Deploy to Cloudflare Pages
        uses: cloudflare/wrangler-action@v3
        with:
          apiToken: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          accountId: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
          command: pages deploy
```

### Step 6: Manual Deploy (For Emergencies or First Deploy)

If you need to deploy without pushing to GitHub:

```bash
# Build first
pnpm build

# Deploy (requires CLOUDFLARE_API_TOKEN and CLOUDFLARE_ACCOUNT_ID as env vars)
# Option A: Set env vars inline
CLOUDFLARE_API_TOKEN=<your-token> CLOUDFLARE_ACCOUNT_ID=<your-id> npx wrangler pages deploy

# Option B: Use .dev.vars (already gitignored)
# Create .dev.vars at repo root with:
#   CLOUDFLARE_API_TOKEN=<your-token>
#   CLOUDFLARE_ACCOUNT_ID=<your-id>
# Then run:
npx wrangler pages deploy
```

**Never paste tokens directly into your terminal history if you can avoid it.** Prefer environment files (`.dev.vars` is already gitignored) or a secrets manager.

---

## What Triggers a Deploy

| Event | Deploys? | Why |
|-------|----------|-----|
| Push to `main` that changes `sites/urd.dev/**` | Yes | Path filter matches |
| Push to `main` that only changes `briefs/`, `docs/`, `design/`, `README.md`, etc. | No | Path filter excludes |
| Push to any non-`main` branch | No | Branch filter excludes |
| Manual trigger via GitHub Actions "Run workflow" button | Yes | `workflow_dispatch` |
| `npx wrangler pages deploy` from local machine | Yes | Direct CLI deploy |

---

## Verifying It Works

After the first deploy:

1. Check **GitHub → Actions** tab — the workflow should show a successful run
2. Check **Cloudflare Dashboard → Pages → urd-dev** — you should see the deployment with a timestamp
3. Visit `https://urd-dev.pages.dev` (the default Cloudflare Pages URL) to confirm the site is live
4. If custom domain is configured, visit `https://urd.dev`

---

## Cost & Limits (Free Tier)

| Resource | Free Tier Limit | Our Usage |
|----------|----------------|-----------|
| Builds per month | 500 | Only when `sites/urd.dev/` changes — likely <20/month |
| Concurrent builds | 1 | Fine for our scale |
| Sites | Unlimited | We have 1 |
| Bandwidth | Unlimited | N/A |
| Requests | Unlimited | N/A |

---

## Future Considerations

- **Preview deploys on PRs:** Add a second workflow triggered on `pull_request` with the same path filter. Deploy to a preview URL using Wrangler's branch deploy feature.
- **Multiple sites:** If `urd.world` is added later, create a separate Cloudflare Pages project and a parallel workflow with its own path filter (`sites/urd.world/**`).
- **Pre-commit secret scanning:** Install `gitleaks` or `secretlint` as a pre-commit hook for an extra layer of protection beyond GitHub's server-side scanning.
