# agdb_studio

## Conventions

- **No barrel exports** — do not add `index.ts` barrel files to library packages. Import directly from source paths (e.g. `@agdb-studio/api/src/api`).
- **`type` not `interface`** — use `type` for all TypeScript type definitions.
- **Co-located tests** — place `.spec.ts` test files next to the source file they test.
- **CSS custom properties** — use design tokens from `app/src/assets/base.css`, not hardcoded color values.
- **Scoped styles** — all Vue component `<style>` blocks should use `scoped`.

## Architecture

Three-tier monorepo under `libs/`:

- `config/` — build and test configuration (`tsconfig`, `testing`)
- `core/` — platform packages (`api`, `auth`, `router`, `design`, `utils`, `profile`)
- `features/` — shared (`common`, `notification`) and domain (`db`, `query`, `user`, `cluster`) packages

Boundary rules enforced by ESLint (`eslint.boundaries.mjs`) and contract tests (`app/src/boundaries.contract.spec.ts`).

## Commands

- Install: `pnpm i --frozen-lockfile`
- Test: `pnpm run test --filter agdb_studio`
- Lint: `pnpm run lint --filter agdb_studio`
- Build: `pnpm run build --filter agdb_studio`
- Functional: `pnpm run test:functional --filter agdb_studio`
