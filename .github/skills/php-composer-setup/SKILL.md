---
name: php-composer-setup
description: Install PHP and Composer, verify they are available in PATH, and confirm the PHP tooling required by this repository works. Use this when PHP or Composer are missing, or when commands under agdb_api/php fail because the environment is not set up.
argument-hint: "[windows|linux|macos] [optional package manager or version]"
disable-model-invocation: true
---

# PHP and Composer Setup

Use this skill when the workspace needs PHP and Composer installed or verified before running commands in `agdb_api/php`.

Repository context:

- The PHP API client lives in `agdb_api/php`.
- Common PHP validation commands in this repository are:
  - `cd agdb_api/php/ && ./ci.sh coverage`
  - `cd agdb_api/php/ && ./ci.sh analyse`
  - `cd agdb_api/php/ && ./ci.sh format`

## Goals

1. Detect whether PHP and Composer are already installed.
2. Install missing tools using the most appropriate package manager for the current OS.
3. Verify that both tools are available in `PATH`.
4. Confirm that the repository's PHP tooling can run.

## General procedure

1. Detect the operating system and check whether `php` and `composer` are already available.
2. Prefer the native package manager for the OS.
3. Do not reinstall tools that are already working unless the user explicitly asks for a version change.
4. If installation updates `PATH`, tell the user a new terminal might be required and re-run verification.
5. After installation, verify both binaries before attempting repository commands.
6. If a command requires elevated privileges, explain that clearly before running it.

## Verification commands

Run these first:

- `php --version`
- `composer --version`

If both succeed, move directly to repository verification.

## Windows

Preferred package manager order:

1. `winget`
2. `choco`

Suggested commands:

- Check for `winget`: `winget --version`
- Install PHP with winget: `winget install --id PHP.PHP.8.4 --source winget`
- Install Composer with winget: `winget install --id Composer.Composer --source winget`
- Install PHP with Chocolatey: `choco install php`
- Install Composer with Chocolatey: `choco install composer`

If the exact winget package ID changes, search for it with:

- `winget search php`
- `winget search composer`

After installation, re-run:

- `php --version`
- `composer --version`

If the current shell still does not see the binaries, open a new terminal and verify again.

## Linux

Prefer the system package manager. Typical commands include:

- Debian or Ubuntu: `sudo apt update && sudo apt install -y php-cli composer`
- Fedora: `sudo dnf install -y php-cli composer`
- Arch: `sudo pacman -Sy php composer`

After installation, re-run:

- `php --version`
- `composer --version`

## macOS

Prefer Homebrew if available:

- Check for Homebrew: `brew --version`
- Install PHP: `brew install php`
- Install Composer: `brew install composer`

After installation, re-run:

- `php --version`
- `composer --version`

## Repository verification

Once PHP and Composer are available, verify the repository setup with the least expensive relevant command first.

Recommended order:

1. `cd agdb_api/php/ && ./ci.sh format`
2. `cd agdb_api/php/ && ./ci.sh analyse`
3. `cd agdb_api/php/ && ./ci.sh coverage`

Choose the command that best matches the user's goal. If the user only asked for installation, stop after binary verification unless they want a repo-level check.

## Operating rules

- Prefer detection before installation.
- Prefer non-destructive verification before long-running checks.
- Report exact installation blockers instead of guessing.
- If package installation is not possible in the current environment, tell the user what command they need to run locally.
- When verification fails, distinguish between missing system tools and repository-level PHP dependency problems.

## Example invocations

- `/php-composer-setup windows`
- `/php-composer-setup windows using winget`
- `/php-composer-setup verify php tooling for agdb_api/php`
- `/php-composer-setup linux`
- `/php-composer-setup macos with Homebrew`
