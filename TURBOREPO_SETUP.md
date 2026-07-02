# Turborepo Setup Documentation

## Overview

This document describes the Turborepo integration for the ILN Smart Contract monorepo, providing intelligent task caching, parallel execution, and improved build times.

## What is Turborepo?

Turborepo is a high-performance build system for JavaScript/TypeScript monorepos that:
- **Caches task outputs** locally and remotely
- **Runs tasks in parallel** respecting dependencies
- **Only rebuilds what changed** using content-based hashing
- **Provides remote caching** for CI/CD and team collaboration

## Performance Improvements

### Before Turborepo
- **Sequential execution**: Tasks ran one after another
- **No caching**: Every task ran from scratch on every execution
- **Manual dependency management**: Had to manually ensure SDK built before CLI
- **Estimated CI time**: ~8-12 minutes for full test suite

### After Turborepo
- **Parallel execution**: Independent tasks run simultaneously
- **Intelligent caching**: Unchanged code uses cached results
- **Automatic dependency resolution**: Turbo ensures SDK builds before CLI automatically
- **Expected CI time**: ~3-5 minutes for full test suite (60%+ improvement on cache hit)
- **Local development**: Near-instant rebuilds for unchanged packages

## Architecture

### Workspace Structure

```
ILN-Smart-Contract/
├── package.json          # Root workspace config
├── turbo.json           # Turborepo pipeline configuration
├── .turborc             # Remote cache configuration
├── sdk/                 # @iln/sdk - TypeScript SDK (Jest)
├── cli/                 # @iln/cli - Command-line interface (Jest)
├── indexer/             # @iln/indexer - REST API indexer (Vitest)
└── notifications/       # @iln/notifications - Notification service (Vitest)
```

### Dependency Graph

```
CLI depends on SDK (must build SDK first)
├── SDK (build) → CLI (build)
├── SDK (test) runs in parallel with CLI (test)
└── Indexer and Notifications are independent
```

## Configuration

### turbo.json Pipeline

The `turbo.json` file defines task pipelines with dependencies:

```json
{
  "pipeline": {
    "build": {
      "dependsOn": ["^build"],        // Wait for dependencies to build first
      "outputs": ["dist/**"],         // Cache these directories
      "env": ["NODE_ENV"]            // Invalidate cache if env changes
    },
    "test": {
      "dependsOn": ["build"],         // Run after local build
      "outputs": ["coverage/**"],     // Cache coverage reports
      "cache": true
    },
    "test:ci": {
      "dependsOn": ["build"],
      "outputs": ["coverage/**"],
      "env": ["NODE_ENV", "CI"]      // CI-specific configuration
    },
    "lint": {
      "outputs": [],                  // No outputs to cache
      "cache": true                   // But cache the lint results
    },
    "typecheck": {
      "dependsOn": ["^build"],        // Need built types from dependencies
      "cache": true
    }
  }
}
```

#### Dependency Operators

- `^build` - Wait for `build` in **dependencies** to finish first
- `build` - Wait for `build` in **same package** to finish first
- No operator - Can run in parallel with other tasks

### Package Scripts

Each package has standardized scripts:

```json
{
  "scripts": {
    "build": "...",           // Build the package
    "test": "...",            // Run tests
    "test:ci": "...",         // Run tests in CI mode
    "lint": "...",            // Run linter
    "typecheck": "tsc --noEmit", // Type checking without emit
    "clean": "rm -rf dist coverage node_modules/.cache"
  }
}
```

## Usage

### Local Development

#### Install Dependencies

```bash
npm install
```

This installs Turborepo at the root and all workspace dependencies.

#### Build All Packages

```bash
npm run build
```

Turborepo will:
1. Build SDK first (no dependencies)
2. Build CLI after SDK completes (depends on SDK)
3. Build indexer and notifications in parallel

#### Run All Tests

```bash
npm run test
```

Runs tests across all packages in parallel, respecting build dependencies.

#### Run Tests in CI Mode

```bash
npm run test:ci
```

Uses CI-specific test configurations (no watch mode, coverage reports, etc.).

#### Type Check

```bash
npm run typecheck
```

Runs TypeScript type checking across all packages.

#### Lint

```bash
npm run lint
```

Runs linters across all packages (currently only CLI has ESLint configured).

#### Development Mode

```bash
npm run dev
```

Starts all services in development/watch mode in parallel.

#### Clean Everything

```bash
npm run clean
```

Removes all build artifacts, coverage reports, and caches.

### Filtering

Run tasks for specific packages:

```bash
# Build only SDK
npx turbo run build --filter=@iln/sdk

# Test SDK and CLI
npx turbo run test --filter=@iln/sdk --filter=@iln/cli

# Build SDK and everything that depends on it
npx turbo run build --filter=...@iln/sdk
```

### Caching

#### View Cache Status

```bash
# Run with verbose output
npx turbo run build --verbose

# Output will show:
# ✓ @iln/sdk:build [CACHE HIT]  <-- Used cached result
# ⠋ @iln/cli:build [RUNNING]    <-- Building fresh
```

#### Clear Local Cache

```bash
# Clear turbo cache
rm -rf .turbo

# Or use turbo command
npx turbo prune --force
```

#### Disable Cache (for debugging)

```bash
# Run without cache
npx turbo run test --force

# Skip cache for specific package
npx turbo run build --filter=@iln/sdk --force
```

## Remote Caching

Remote caching shares build artifacts across team members and CI runs.

### Vercel Remote Cache (Recommended)

1. **Sign up for Vercel** (free for open source):
   ```bash
   npx turbo login
   ```

2. **Link your repository**:
   ```bash
   npx turbo link
   ```

3. **Configure team** (updates `.turborc`):
   The `.turborc` file stores your team configuration.

4. **In CI, add token**:
   ```yaml
   env:
     TURBO_TOKEN: ${{ secrets.TURBO_TOKEN }}
     TURBO_TEAM: team_iln
   ```

### Self-Hosted Remote Cache

For private deployments, you can run your own cache server:

1. **Deploy turbo-cache server**:
   ```bash
   docker run -p 3000:3000 \
     -e PORT=3000 \
     -e STORAGE_PROVIDER=s3 \
     -e S3_BUCKET=your-turbo-cache \
     ghcr.io/ducktors/turbo-cache
   ```

2. **Update `.turborc`**:
   ```json
   {
     "apiUrl": "https://your-cache-server.com",
     "teamId": "your-team"
   }
   ```

3. **Set token in environment**:
   ```bash
   export TURBO_TOKEN=your-auth-token
   ```

## CI/CD Integration

### GitHub Actions

The updated `.github/workflows/ci.yml` uses Turborepo:

```yaml
- name: Setup Turborepo cache
  uses: actions/cache@v4
  with:
    path: .turbo
    key: ${{ runner.os }}-turbo-${{ github.sha }}
    restore-keys: |
      ${{ runner.os }}-turbo-

- name: Run typecheck (Turbo)
  run: npm run typecheck

- name: Run build (Turbo)
  run: npm run build

- name: Run tests (Turbo)
  run: npm run test:ci
```

### Key Benefits in CI

1. **Faster Builds**: Cache hits from previous runs
2. **Parallel Execution**: Multiple jobs run simultaneously
3. **Smart Rebuilds**: Only changed packages rebuild
4. **Consistent Results**: Same cache key = same output

## Performance Metrics

### Measurement Methodology

#### Before Turborepo (Baseline)

Run without cache:
```bash
time (cd sdk && npm run build && npm run test && cd ../cli && npm run build && npm run test && cd ../indexer && npm run build && npm run test && cd ../notifications && npm run build && npm run test)
```

#### After Turborepo (First Run)

Cold cache (no speedup expected):
```bash
time npm run build && npm run test
```

#### After Turborepo (Subsequent Runs)

Warm cache (expect 60-90% speedup):
```bash
# Make a change to one package
echo "// comment" >> sdk/src/index.ts

# Rebuild everything
time npm run build && npm run test
```

### Expected Improvements

| Scenario | Before | After | Improvement |
|----------|--------|-------|-------------|
| **Cold build** (no cache) | ~8 min | ~5 min | 37% faster (parallelization) |
| **Warm build** (cache hit) | ~8 min | ~30 sec | 94% faster (caching) |
| **Single package change** | ~8 min | ~2 min | 75% faster (incremental) |
| **CI with remote cache** | ~12 min | ~4 min | 67% faster |

### Real-World Results

Document actual measurements here after implementation:

```bash
# Measure cold build
npm run clean
time npm run build && npm run test

# Measure warm build (no changes)
time npm run build && npm run test

# Measure incremental build (small change)
echo "// comment" >> sdk/src/index.ts
time npm run build && npm run test
```

## Troubleshooting

### Cache Not Working

**Problem**: Tasks always rebuild, never hit cache

**Solutions**:
1. Check `.turbo` directory exists
2. Verify `outputs` in `turbo.json` match actual build outputs
3. Check for dynamic environment variables (use `globalEnv`)
4. Look for non-deterministic builds (timestamps, random values)

### Incorrect Dependencies

**Problem**: CLI builds before SDK is ready

**Solutions**:
1. Verify `^build` dependency in `turbo.json`
2. Check package.json workspace configuration
3. Run with `--dry-run` to see execution order:
   ```bash
   npx turbo run build --dry-run
   ```

### Remote Cache Not Connecting

**Problem**: Can't push/pull from remote cache

**Solutions**:
1. Verify `TURBO_TOKEN` is set
2. Check `.turborc` configuration
3. Test connection: `npx turbo login`
4. Check network/firewall settings

### Disk Space Issues

**Problem**: `.turbo` directory growing too large

**Solutions**:
1. Clean periodically: `rm -rf .turbo`
2. Configure pruning in CI:
   ```yaml
   - name: Prune cache
     run: npx turbo prune --force
   ```
3. Limit cache size (future feature)

## Migration Guide

### From Manual Sequencing

**Before**:
```bash
cd sdk && npm run build && cd ..
cd cli && npm run build && cd ..
cd sdk && npm run test &
cd cli && npm run test &
wait
```

**After**:
```bash
npm run build  # Handles dependencies automatically
npm run test   # Runs in parallel automatically
```

### From Lerna/Nx

Turborepo is simpler and faster:

1. Remove Lerna/Nx config files
2. Add `turbo.json`
3. Update `package.json` scripts to use `turbo run`
4. No need for complex workspace topology files

### From Yarn Workspaces Only

Keep your workspace config, add Turbo on top:

1. Keep existing `package.json` workspaces
2. Add `turbo.json` for task orchestration
3. Update scripts to use `turbo run`
4. Enjoy caching and parallelization

## Best Practices

### 1. Deterministic Builds

Ensure builds produce the same output for the same input:
- Don't embed timestamps or random values
- Use fixed version dependencies
- Avoid relying on system state

### 2. Proper Output Configuration

List all build artifacts in `outputs`:
```json
{
  "build": {
    "outputs": [
      "dist/**",
      "build/**",
      ".next/**",
      "!.next/cache/**"  // Exclude cache directories
    ]
  }
}
```

### 3. Environment Variable Management

Use `globalEnv` for shared variables:
```json
{
  "globalEnv": ["NODE_ENV", "API_URL"],
  "pipeline": {
    "test": {
      "env": ["TEST_DATABASE_URL"]  // Task-specific
    }
  }
}
```

### 4. Cache Strategy

- **Development**: Use local cache only
- **CI**: Use remote cache with GitHub Actions cache
- **Production**: Use remote cache with Vercel or self-hosted

### 5. Debugging

Use `--dry-run` to see what will execute:
```bash
npx turbo run build --dry-run=json
```

Use `--graph` to visualize dependencies:
```bash
npx turbo run build --graph
```

## Additional Resources

- [Turborepo Documentation](https://turbo.build/repo/docs)
- [Turborepo Examples](https://github.com/vercel/turbo/tree/main/examples)
- [Remote Caching Guide](https://turbo.build/repo/docs/core-concepts/remote-caching)
- [Pipeline Configuration](https://turbo.build/repo/docs/core-concepts/monorepos/running-tasks)

## Support

For issues or questions:
1. Check this documentation
2. Review Turborepo docs
3. Check GitHub issues
4. Ask in team chat/discussions

## Changelog

### 2024-06-29 - Initial Setup
- Installed Turborepo v2.3.3
- Created `turbo.json` with pipeline configuration
- Updated all package.json scripts
- Updated CI workflow with Turbo integration
- Configured remote caching
- Documented performance expectations
