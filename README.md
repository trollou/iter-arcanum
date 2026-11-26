# Iter Arcanum

*Iter Arcanum* (Latin for "the journey of the arcana") is a single-player desktop game **heavily inspired by the Mage Knight board game**. You play a hero who explores a realm, fights enemies, recruits units, conquers sites and grows stronger, turn after turn, by playing cards.

It runs on **Windows**, **Linux** and **macOS**. The game is displayed in your system language when it is **French or English**, and in English otherwise. To force a language for one session, pass `--lang fr` or `--lang en` (for example `iter-arcanum --lang en`, or `cargo run -p arcanum-ui --release -- --lang en` from the sources). **Menu > Preferences…** (or **Ctrl+,**, **Cmd+,** on macOS, also on the launch screen) picks the language, whether the game starts in full screen, and four volumes (master, music, ambience, sound effects); these preferences are kept in `preferences.json` in your user configuration folder (`~/.config/iter-arcanum/` on Linux, `%APPDATA%\iter-arcanum\config\` on Windows, `~/Library/Application Support/iter-arcanum/` on macOS), or in the repository's `config/` folder when launched from the sources with `cargo run`. The `ARCANUM_CONFIG_DIR` environment variable picks another folder.

This is a work in progress, at its first public version (0.1.0): two scenarios of the base game are playable from start to finish, solo, with all the cards of the base game (see below).

> Unofficial, free and non-commercial fan project, not affiliated with or endorsed by WizKids or any publisher of Mage Knight. It contains no official artwork from the game. See [NOTICE](NOTICE).

## What you can do today

- **Play a full solo game** of one of two scenarios of the base game: "First Reconnaissance" (explore until you discover a city, then play the final turn) or "Solo Conquest" (find and conquer the two cities of the region within six rounds), against the Ghost player who sets the pace, and get your final score.
- **Pick your hero** among the four of the base game, and the scenario (or a training "sandbox" without enemies).
- **Choose a tactic** at the start of each round, and gain **skills** as your hero levels up.
- **Move** across a hexagonal map, by day and by night, with terrain costs that differ between the two.
- **Play your cards**: for their basic effect, their stronger effect (by paying mana from the Source, crystals or tokens), or sideways for a small bonus.
- **Fight** wandering enemies and site defenders: ranged and siege phase, block, damage and attack phases, elements and resistances, enemy abilities, Wounds.
- **Heal** your hero, rest, and earn Fame to gain levels.
- **Visit sites**: villages and monasteries (healing, recruiting, advanced actions), keeps and mage towers (assault and conquest), dungeons, tombs, monster dens, spawning grounds, magical glades and ruins.
- **Assault cities** and conquer them, then trade with their inhabitants.
- **Gain and play** advanced actions, spells and artifacts: every card of the base game has its effect.
- **Recruit units** and use them in combat.
- **Play through several rounds**, alternating day and night.
- **Save, load and undo** decision by decision.

### Not there yet

- The other scenarios of the base game, and the content of the expansions (heroes, enemies, cards), are not playable yet.
- No computer opponents besides the Ghost player, no multiplayer, no online play.
- The look is still evolving: a more polished map view is planned.

## Running the game

### Download

Download the archive for your system from the [latest release](https://github.com/trollou/iter-arcanum/releases/latest):

| System | Archive | How to launch |
|---|---|---|
| Windows 10 and 11 (64-bit) | `iter-arcanum-X.Y.Z-windows-x86_64.zip` | Extract the whole folder, then double-click `iter-arcanum.exe`. |
| Linux (64-bit, glibc 2.35 or later: Ubuntu 22.04, Linux Mint 21, Debian 12 and later) | `iter-arcanum-X.Y.Z-linux-x86_64.tar.gz` | Extract it (`tar xzf …`), then run `./iter-arcanum` from the extracted folder. |
| macOS 11 and later (Apple Silicon and Intel) | `iter-arcanum-X.Y.Z-macos-universal.dmg` | Open the `.dmg`, drag **Iter Arcanum** onto the **Applications** folder, then launch it from there. |

`SHA256SUMS.txt`, next to the archives, lets you check a download (`sha256sum -c SHA256SUMS.txt --ignore-missing` on Linux).

Keep the `assets/` folder next to the executable: the game reads its texts, images and sounds from it (on macOS, they are inside the application). Preferences, saves and logs go to your user folders (see below), never next to the executable, so you can replace the folder with a newer version without losing anything.

**The game is not signed**, so your system warns you the first time:

- **Windows**: SmartScreen shows "Windows protected your PC". Click **More info**, then **Run anyway**.
- **Windows 11 with Smart App Control**: it blocks unsigned applications, with no per-application exception. You can only play by turning it off (**Windows Security > App & browser control > Smart App Control settings**); decide whether you accept that for your computer. Signing the game is planned ([#70](https://github.com/trollou/iter-arcanum/issues/70)).
- **macOS**: Gatekeeper refuses to open an application from an unidentified developer. Open it once with a right click (or Ctrl+click) on **Iter Arcanum** in **Applications**, then **Open**, and confirm. On recent macOS versions where this is not offered, try to open it once, then go to **System Settings > Privacy & Security** and click **Open Anyway** next to the message about Iter Arcanum.

**On Linux**, the game needs a graphical session (X11 or Wayland) with OpenGL, which any desktop installation provides. It uses these system libraries, already installed on Ubuntu, Linux Mint or Debian with a desktop: `libc6`, `libgcc-s1`, `libasound2` (sound), `libxkbcommon0`, and, under X11, `libgl1`, `libx11-6`, `libx11-xcb1`, `libxcursor1`, `libxi6`, `libxkbcommon-x11-0`, or, under Wayland, `libegl1` and `libwayland-client0`. Without a sound card the game runs silently.

### From the sources

You can also compile the game yourself, on Windows, Linux or macOS.

1. Install **Rust** (stable) with [rustup](https://rustup.rs/).
2. Get this repository (`git clone`).
3. From the repository folder, run:

   ```
   cargo run -p arcanum-ui --release
   ```

   The first build takes a few minutes.

On Linux, compiling the sound also needs the ALSA development files (Debian, Ubuntu, Linux Mint: `sudo apt install libasound2-dev pkg-config`). On macOS, nothing else is needed.

Launched from the repository, the game finds its `assets/` folder by itself, and writes its preferences, saves and logs in the repository (`config/`, `saves/`, `logs/`). If you copy the executable elsewhere, put the `assets/` folder next to it, or point to it with the `ARCANUM_ASSETS_DIR` environment variable.

On macOS, keyboard shortcuts written « Ctrl+… » in this page use **Cmd** and are shown as « Cmd+… » in the game.

## Getting started

At launch, the **New game** screen lets you choose the hero, the scenario and, optionally, a seed: the same seed gives the same game, so you can replay it. Leave it empty for a random game.

Then, each turn: play cards to get movement, influence, attack or block, move your hero, explore, fight, and end your turn to draw new cards.

Hover over a button for a tooltip, which also shows its keyboard shortcut when it has one. Hovering over a map hex tells you what it contains and what it costs to enter.

## Controls and shortcuts

| Key | Effect |
|---|---|
| Mouse wheel, drag | Zoom, scroll the map |
| Arrow keys | Scroll the map |
| `+` / `-` | Zoom in / out |
| Home | Re-center the map |
| End | End the turn |
| Enter | Confirm (end the turn, confirm an attack or block, start a game…) |
| Esc | Cancel or close the current window |
| `O` | Open or close the offers (units, advanced actions, spells) |
| `D` | Open or close the discard pile |
| `I` | Interact with the inhabitants of a village or monastery |
| `E` | Enter an adventure site, or make an offering at an altar |
| `1`, `2`, `3`… | Pick among the proposed options (cards, artifacts, colors…) |
| Ctrl+Z | Undo |
| Ctrl+S | Save |
| Ctrl+O | Load a game |
| Ctrl+N | New game |
| Ctrl+Q | Quit |
| F11 | Full screen |

The window's menu lists these shortcuts too.

## Saving and resuming

- The game **saves automatically at the start of each turn** (the "autosave", overwritten each time).
- **Ctrl+S** creates a dated save (`YYYY-MM-DD_HH-MM-SS.json`). **Ctrl+O** opens the list of saves, most recent first, with the autosave on top.
- A loaded game resumes exactly where you left it, even in the middle of a combat.
- **Ctrl+Z** undoes decision by decision, back to the "point of no return": once you have revealed a tile or the turn has started, you can't go back further.
- Saves go in a `saves` subfolder of your user data folder; when launched from the repository with `cargo run`, the game writes to the repository's `saves/` folder. The `ARCANUM_SAVE_DIR` environment variable picks another folder.
- The game is under development: a save made before version 0.1.0 may not load. From version 0.1.0 on, saves are meant to keep loading in later versions.

## Troubleshooting

The game writes a log file at each launch:

- Windows: `%LOCALAPPDATA%\iter-arcanum\data\logs`
- Linux: `~/.local/state/iter-arcanum/logs` (or `$XDG_STATE_HOME/iter-arcanum/logs`)
- macOS: `~/Library/Logs/iter-arcanum` (also shown by the Console app)
- Launched from the repository with `cargo run`: the repository's `logs/` folder.

Attach it when you report a problem in the [repository issues](https://github.com/trollou/iter-arcanum/issues).

## Contributing

For development (method, architecture, tests), see [`CLAUDE.md`](CLAUDE.md).

## License

The source code is under the [GNU General Public License v3](LICENSE) or later. The images and texts in `assets/` are under [CC BY-SA 4.0](assets/LICENSE). Names and values that come from the original game are not covered by these licenses: see [NOTICE](NOTICE).
