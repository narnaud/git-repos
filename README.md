# Git-Repos - 🔍 Scan and manage git repositories

![Demo](assets/demo.gif)

## About This Project

This project is an **experimentation** built entirely using **vibe-coding** with GitHub Copilot. The goal was to explore how easy it is to develop a complete TUI application using only AI assistance, and to evaluate whether the resulting code is maintainable and follows good practices.

**Vibe-coding approach:**

- All code was generated through natural language conversations with GitHub Copilot
- Features were added incrementally by describing the desired functionality
- The AI handled architecture decisions, refactoring, and code organization
- Following the DTDP (Detailed Technical Development Process) with investigation, discussion, action, and summary phases

**Key learnings:**

- TUI development with `ratatui` was straightforward through conversation
- Code quality remained high with proper use of Rust idioms (edition 2024)
- Refactoring and separation of concerns was handled naturally
- Git commits followed conventional commits format consistently

The project serves as a case study for AI-assisted development and demonstrates that maintainable, well-structured code can be created through conversational programming.

> [!NOTE]
> Even the README has been AI generated, except for this line. Apparently Copilot has a high opinion of itself :laughing:

---

## Overview

Git-Repos is a command-line tool with a Text User Interface (TUI) that helps you discover and manage git repositories on your system. It recursively scans directories to find all git repositories and displays them in an interactive table with current branch information.

## Features

- 🔎 **Recursive scanning** - Find all git repositories in a directory tree
- 🎯 **Smart filtering** - Excludes nested repositories (submodules) for cleaner results
- 📊 **Interactive TUI** - Beautiful table interface with rounded borders
- 🌿 **Branch detection** - Shows the current branch for each repository
- 📡 **Remote status** - Displays ahead/behind status, local-only, or up-to-date
- 📝 **Working tree status** - Shows clean, modified, or staged changes
- 📅 **Last commit info** - Display last commit time (relative) and author
- ⚡ **Async loading** - Fast startup with background data loading
- 🔄 **Auto-fetch** - Automatically fetch all repositories with remotes asynchronously
- 🔀 **Auto-update** - Optionally fast-forward merge local branches after fetch
- 🔍 **Search filter** - Press `/` to search repositories by name
- 📋 **View modes** - Filter repositories by: All, No Upstream, Behind, Modified
- 🎨 **Color-coded display** - Visual indicators for repository states
- ⌨️ **Keyboard navigation** - Vim-style (j/k) and arrow key navigation
- 🚀 **Quick navigation** - Press Enter to change directory to selected repository
- 💾 **Persistent cache** - Saves repository list for cross-machine sharing
- 🗑️ **Repository management** - Delete repositories with 'd' key
- 📥 **Clone missing repos** - Clone repositories marked as missing with 'c' key
- 🔧 **Configuration** - Set root path and auto-update preferences
- ⚡ **Fast and efficient** - Written in Rust for optimal performance

## Installation

### Installation via [Cargo](https://doc.rust-lang.org/cargo/)

```
cargo install git-repos-manager
```

### Installation via [Scoop](https://scoop.sh/)

Install **git-repos** with [scoop](<https://scoop.sh/>):

```powershell
scoop bucket add narnaud https://github.com/narnaud/scoop-bucket
scoop install git-repos
```

### Prebuilt binaries

Download the latest archive for your platform from the [Releases](https://github.com/narnaud/git-repos/releases) page, extract it, and add the binary to your PATH.

| Platform      | Archive                                  |
|---------------|------------------------------------------|
| Windows (x86_64) | `git-repos-x86_64-pc-windows-msvc.zip` |
| Linux (x86_64) | `git-repos-x86_64-unknown-linux-gnu.tar.gz` |
| Linux (aarch64) | `git-repos-aarch64-unknown-linux-gnu.tar.gz` |
| macOS (Apple Silicon) | `git-repos-aarch64-apple-darwin.tar.gz` |

### Build from source

```powershell
git clone https://github.com/narnaud/git-repos.git
cd git-repos
cargo build --release
```

The compiled binary will be in `target/release/git-repos.exe`

## Usage

### Basic usage

Scan the current directory:

```powershell
git-repos
```

Scan a specific directory:

```powershell
git-repos D:\projects
```

By default, the tool automatically fetches all repositories with remotes. To disable this:

```powershell
git-repos --no-fetch
```

To also update local branches with fast-forward merge after fetching:

```powershell
git-repos --update
```

When auto-fetch is enabled (default), the tool runs `git fetch --all --prune` for each repository that has a remote configured. A spinner animation in the status bar shows the progress. With `--update`, it also performs `git merge --ff-only` to update local branches when possible.

### Configuration

Set the root path to scan by default:

```powershell
git-repos set root D:\projects
```

Enable auto-update by default (fast-forward merge after fetch):

```powershell
git-repos set update true
```

The configuration is stored in:

- Windows: `%APPDATA%\git-repos\config.toml`
- Linux/macOS: `~/.config/git-repos/config.toml`

### Repository cache

The tool maintains a cache of discovered repositories in `repos.yaml` (same directory as config). This cache:

- Saves the list of all repositories with their remote URLs
- Persists across sessions for cross-machine sharing
- Tracks deleted repositories as "missing" (shown in gray)
- Merges with newly discovered repositories when scanning

Missing repositories can be:

- Cloned back using the 'c' key
- Permanently removed from cache using the 'd' key

### Shell integration (recommended)

Shell integration (recommended)
--------------------------------

Since a program cannot change the shell's working directory, you need a wrapper that uses a temp file and the `--cwd-file` flag.

#### PowerShell

Add this to your PowerShell profile (`$PROFILE`):

```powershell
function gr {
    $tmp = "$env:TEMP\git-repos-cwd.txt"
    Remove-Item $tmp -ErrorAction SilentlyContinue
    git-repos --cwd-file $tmp $args
    if (Test-Path $tmp) {
        $path = Get-Content $tmp -Raw
        if ($path) { Set-Location $path }
        Remove-Item $tmp
    }
}
```

#### Bash/Zsh

Add this to your `.bashrc` or `.zshrc`:

```bash
gr() {
    tmp="/tmp/git-repos-cwd.txt"
    rm -f "$tmp"
    git-repos --cwd-file "$tmp" "$@"
    if [ -s "$tmp" ]; then
        cd "$(cat "$tmp")"
        rm -f "$tmp"
    fi
}
```

Now you can use `gr` to interactively select and navigate to a repository:

```powershell
gr              # Scan current directory
gr D:\projects  # Scan specific directory
```

### Keyboard controls

- **↑/↓** or **j/k** - Navigate through the repository list
- **[** / **]** - Switch between view modes (All, Needs Attention, Behind, Modified)
- **/** - Enter search mode to filter repositories by name
- **Esc** - Exit search mode and clear search filter
- **d** - Delete selected repository (marks as missing) or remove from cache if already missing
- **c** - Clone selected missing repository (auto-detects GitHub for `gh` vs `git clone`)
- **u** - Update selected repository (fetch + status)
- **Enter** - Change directory to selected repository (exits the app)
- **q** or **Ctrl-C** - Quit the application

### View Modes

- **All** - Show all repositories
- **No Upstream** - Show repositories that are local-only or have no tracking branch
- **Behind** - Show only repositories that are behind their upstream
- **Modified** - Show only repositories with uncommitted changes

The current mode is highlighted at the bottom right of the table.

### Example output

```text
╭─ Git Repositories - D:\projects ────────────────────────────────────────────────────────────────────────╮
│ Repository              │ Branch  │ Remote Status │ Status     │ Last Commit                          │
├─────────────────────────┼─────────┼───────────────┼────────────┼──────────────────────────────────────┤
│ > kdab/knut             │ main    │ ↑2 ↓0         │ 3M         │ 2 days ago by John Doe               │
│   kdab/training-material│ develop │ up-to-date    │ clean      │ 1 week ago by Jane Smith             │
│   narnaud/git-repos     │ main    │ local-only    │ 1S 2M      │ 5 minutes ago by Nicolas Arnaud      │
│   oss/ratatui           │ main    │ ⟳ loading...  │ clean      │ ⟳ loading...                         │
│   user/deleted-repo     │ -       │ -             │ missing    │ -                                    │
├─────────────────────────┴─────────┴───────────────┴────────────┴──All─[No Upstream]─Behind─Modified────┤
╰──────────────────────────────────────────────────────────────────────────────────────────────────────────╯
Found 5 repositories (1 missing) | ⠋ Fetching 2 repositories... | Mode: [/] | Search: / | Quit: q or Ctrl-C
```

#### Color indicators

**Remote Status:**

- 🟢 Green - `up-to-date`
- 🔵 Cyan - `↑X ↓Y` (ahead/behind)
- 🟡 Yellow - `no-tracking`
- 🔴 Red - `local-only`
- ⚫ DarkGray - `⟳ loading...`

**Working Tree Status:**

- 🟢 Green - `clean`
- 🟡 Yellow - `XM` (modified), `XS` (staged), `XS YM` (both)
- ⚫ DarkGray - `⟳ loading...` or `unknown`

**Missing Repositories:**

- ⚫ DarkGray - Repository deleted or not present on this machine
- ⚪ White - Missing repository when selected (for better visibility)

### Repository lifecycle

**Deleting repositories:**

1. Select a repository and press 'd'
2. Confirm the deletion (default: No)
3. The repository is deleted from disk and marked as "missing" in the cache
4. Missing repositories appear in gray at the bottom of the list

**Cloning missing repositories:**

1. Select a missing repository (shown in gray)
2. Press 'c' to clone it back
3. The tool automatically detects GitHub repos and uses `gh repo clone` if available
4. Progress is shown with an animated spinner
5. Once cloned, the repository appears normally in the list

**Removing from cache:**

1. Select a missing repository
2. Press 'd' again to permanently remove it from the cache
3. It will no longer appear in the list

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - Copyright (c) Nicolas Arnaud-Cormos

See [LICENSE](LICENSE) file for details.
