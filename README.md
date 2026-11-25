# Iter Arcanum

*Iter Arcanum* (Latin for "the journey of the arcana") is a single-player desktop game **heavily inspired by the Mage Knight board game**. You play a hero who explores a realm, fights enemies, recruits units, conquers sites and grows stronger, turn after turn, by playing cards.

It runs on **Windows**, **Linux** and **macOS**. The game is displayed in your system language when it is **French or English**, and in English otherwise. To force a language for one session, pass `--lang fr` or `--lang en` (for example `cargo run -p arcanum-ui --release -- --lang en`). **Menu > Preferences…** (or **Ctrl+,**, **Cmd+,** on macOS, also on the launch screen) picks the language, whether the game starts in full screen, and four volumes (master, music, ambience, sound effects); these preferences are kept in `preferences.json` in your user configuration folder (`~/.config/iter-arcanum/` on Linux, `%APPDATA%\iter-arcanum\config\` on Windows, `~/Library/Application Support/iter-arcanum/` on macOS), or in the repository's `config/` folder when launched with `cargo run`. The `ARCANUM_CONFIG_DIR` environment variable picks another folder.

This is a work in progress: a full game from start to finish is playable, but many cards don't have their effect yet (see below).

> Unofficial, free and non-commercial fan project, not affiliated with or endorsed by WizKids or any publisher of Mage Knight. It contains no official artwork from the game. See [NOTICE](NOTICE).

## What you can do today

- **Play a full game of "First Reconnaissance"** solo: explore the map by revealing tiles until you discover a city, play the final turn and get your final score.
- **Pick your hero** among the four of the base game, and the scenario (First Reconnaissance, or a training "sandbox" without enemies).
- **Move** across a hexagonal map, by day and by night, with terrain costs that differ between the two.
- **Play your cards**: for their basic effect, their stronger effect (by paying mana from the Source, crystals or tokens), or sideways for a small bonus.
- **Fight** wandering enemies and site defenders: ranged and siege phase, block, damage and attack phases, elements and resistances, enemy abilities, Wounds.
- **Heal** your hero, rest, and earn Fame to gain levels.
- **Visit sites**: villages and monasteries (healing, recruiting, advanced actions), keeps and mage towers (assault and conquest), dungeons, tombs, monster dens, spawning grounds, magical glades and ruins.
- **Recruit units** and use them in combat.
- **Play through several rounds**, alternating day and night.
- **Save, load and undo** decision by decision.

### Not there yet

- Many cards (advanced actions, spells, artifacts) don't do what they say yet: a card whose effect isn't coded can only be played sideways.
- Hero skills, tactics, the Ghost player, conquerable cities and other scenarios are not playable yet.
- No multiplayer, no online play.
- The look is still evolving: a more polished map view is planned.

## Running the game

There is no downloadable build yet: you compile the game from this repository, on Windows, Linux or macOS. On macOS, keyboard shortcuts written « Ctrl+… » elsewhere use **Cmd** and are shown as « Cmd+… ».

1. Install **Rust** (stable) with [rustup](https://rustup.rs/).
2. Get this repository (`git clone`).
3. From the repository folder, run:

   ```
   cargo run -p arcanum-ui --release
   ```

   The first build takes a few minutes.

On Linux you need a graphical session (X11 or Wayland) with OpenGL, and the ALSA development files to compile the sound (Debian, Ubuntu, Linux Mint: `sudo apt install libasound2-dev pkg-config`). Without a sound card the game runs silently. On macOS (Apple Silicon is what the CI checks), nothing else is needed; there is no `.app` bundle yet, so launch it from the repository.

The game reads its resources (texts, images) from the `assets/` folder: it finds it by itself when launched from the repository. If you copy the executable elsewhere, put the `assets/` folder next to it, or point to it with the `ARCANUM_ASSETS_DIR` environment variable.

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
- The game is under development: a save made with an older version may no longer load after an update.

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
