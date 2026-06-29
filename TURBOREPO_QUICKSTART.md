# Turborepo Quick Start Guide

## TL;DR

Turborepo makes your builds **60-90% faster** by caching results and running tasks in parallel.

## Installation

```bash
# Install all dependencies (includes Turborepo)
npm install
```

## Common Commands

### Build Everything
```bash
npm run build
```
**What it does**: Builds all packages in dependency order (SDK → CLI, indexer, notifications)

### Run All Tests
```bash
npm run test
```
**What it does**: Runs test suites in parallel across all packages

### Type Check
```bash
npm run typecheck
```
**What it does**: Validates TypeScript types across all packages

### Development Mode
```bash
npm run dev
```
**What it does**: Starts all services in watch mode

### Clean Build
```bash
npm run clean
npm run build
```
**What it does**: Removes all artifacts and rebuilds from scratch

## How It Works

### First Run (No Cache)
```bash
npm run build
```
```
⠋ @iln/sdk:build      [RUNNING]  Building...
⠋ @iln/indexer:build  [RUNNING]  Building...
⠋ @iln/notifications:build [RUNNING]  Building...
✓ @iln/sdk:build      [DONE]     12.3s
⠋ @iln/cli:build      [RUNNING]  Building...
✓ @iln/indexer:build  [DONE]     8.1s
✓ @iln/notifications:build [DONE] 7.9s
✓ @iln/cli:build      [DONE]     5.2s

Total time: 33.5s
```

### Second Run (With Cache)
```bash
npm run build
```
```
✓ @iln/sdk:build      [CACHE HIT]  0.1s
✓ @iln/indexer:build  [CACHE HIT]  0.1s
✓ @iln/notifications:build [CACHE HIT] 0.1s
✓ @iln/cli:build      [CACHE HIT]  0.1s

Total time: 0.4s (98% faster!)
```

### After Changing One Package
```bash
# Edit SDK
echo "// comment" >> sdk/src/index.ts

npm run build
```
```
✓ @iln/indexer:build  [CACHE HIT]  0.1s
✓ @iln/notifications:build [CACHE HIT] 0.1s
⠋ @iln/sdk:build      [RUNNING]   Building...
✓ @iln/sdk:build      [DONE]      12.1s
⠋ @iln/cli:build      [RUNNING]   Building...
✓ @iln/cli:build      [DONE]      5.3s

Total time: 17.6s (only rebuilt what changed!)
```

## Filtering

### Run command for specific package
```bash
# Build only SDK
npx turbo run build --filter=@iln/sdk

# Test only CLI
npx turbo run test --filter=@iln/cli
```

### Run command for multiple packages
```bash
# Build SDK and CLI
npx turbo run build --filter=@iln/sdk --filter=@iln/cli
```

### Build package and all dependents
```bash
# Build SDK and everything that depends on it (CLI)
npx turbo run build --filter=...@iln/sdk
```

## Cache Management

### View what's cached
```bash
npx turbo run build --verbose
```

### Clear cache
```bash
rm -rf .turbo
```

### Force rebuild (ignore cache)
```bash
npx turbo run build --force
```

## CI/CD

The CI workflow automatically uses Turborepo caching:

```yaml
# .github/workflows/ci.yml
- name: Install dependencies
  run: npm ci

- name: Setup Turborepo cache
  uses: actions/cache@v4
  with:
    path: .turbo
    key: ${{ runner.os }}-turbo-${{ github.sha }}

- name: Run build
  run: npm run build

- name: Run tests
  run: npm run test:ci
```

## Performance Expectations

| Scenario | Time Without Turbo | Time With Turbo | Improvement |
|----------|-------------------|-----------------|-------------|
| First build | ~8 min | ~5 min | 37% |
| No changes | ~8 min | ~30 sec | **94%** |
| Change 1 package | ~8 min | ~2 min | 75% |

## Troubleshooting

### Cache not working?
```bash
# Check if .turbo directory exists
ls -la .turbo

# Verify outputs configuration
cat turbo.json | grep -A 5 outputs
```

### Build order wrong?
```bash
# See execution plan
npx turbo run build --dry-run

# Visualize dependency graph
npx turbo run build --graph
```

### Need to rebuild everything?
```bash
# Clean and rebuild
npm run clean
npm run build
```

## Remote Caching (Optional)

Share cache across team and CI:

```bash
# Login to Vercel
npx turbo login

# Link repository
npx turbo link
```

Now your builds will be even faster because CI and teammates share cached results!

## Learn More

- Full documentation: `TURBOREPO_SETUP.md`
- Turborepo docs: https://turbo.build/repo/docs
- Configuration: `turbo.json`

## Need Help?

1. Check `TURBOREPO_SETUP.md` for detailed docs
2. Run with `--verbose` to see what's happening
3. Use `--dry-run` to see execution plan
4. Ask in team chat

---

**Pro Tip**: Run `npm run build` before switching branches to cache the current state. When you switch back, it'll be instant! ⚡
