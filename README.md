# ┌────────────────────────────────────────────────────────────────────────┐
# │ dxgmr: THE_ASCII_ARCHITECT                                             │
# └────────────────────────────────────────────────────────────────────────┘

> "Why use a 50MB SVG when 79 characters of ASCII can explain your logic?"

**dxgmr** (pronounced "diagrammer") is a lightweight, terminal-based tool designed for developers who want to create professional-grade ASCII diagrams without leaving their environment.

```text
                                  +-----------------+
                                  |                 |
                                  |      User       |
                                  |                 |
                                  +--------o--------+
                                           |
                                           v
                                          \+/
                                        \\   //
                                      \\       //
                                    +\   Input   \+
                                      //       \\
                                        //   \\
                                          /o\
                                            |
                    +---------+------------+----------+---------+
                    |         |                       |         |
                    v         v                       v         v
               +----------+  +----------+        +----------+  +----------+
               |          |  |          |        |          |  |          |
               | Database |  |  Cache   |        |  Auth    |  | Logic    |
               |          |  |          |        |          |  |          |
               +----------+  +----------+        +----------+  +----------+
```

## ┌──────────────┐
## │ RATIONALE    │
## └──────────────┘

*   **⚡ Speed**: Built for developers who hate reaching for the mouse. With a "Type, Tab, Link" workflow, you can map out a complex system in seconds.
*   **🧩 Persistence**: Unlike simple ASCII drawers, **dxgmr** saves your model to `.json`. You can open, edit, and move shapes in previously saved diagrams.
*   **🔗 Analog Aesthetics**: There is a unique clarity to fixed-width ASCII. It belongs in your code comments, READMEs, and technical documentation.

## ┌──────────────┐
## │ FEATURES     │
## └──────────────┘

*   **README Optimized**: Automatically constrained to a **79-character width**, ensuring your diagrams never wrap or break layout in GitHub READMEs.
*   **Smart Staircase Routing**: Implements professional routing with automatic right-angles. It's not just lines; it's architecture.
*   **Modal Editing**: Inspired by Vim. Switch between `Normal`, `Insert`, `Leader`, `Resize`, and `Help` modes.
*   **Vim-like CLI**: Use subcommands like `new` and `open` to manage your files.
*   **Dual-Format Export**: One click saves both a `.txt` (for documentation) and a `.json` (for future editing).

## ┌──────────────┐
## │ HOW TO USE   │
## └──────────────┘

### 🎬 Operations
The CLI supports Vim-like file management:
```bash
# Start a new diagram
dxgmr new "System Architecture"

# Open an existing diagram (.json must exist)
dxgmr open "System Architecture"

# Quick open (auto-detects .json)
dxgmr "System Architecture"
```

### ⌨️ Keyboard Workflow
The primary power of **dxgmr** lies in its **Leader Key** system (the `Spacebar`).

| Key | Action | Mode |
| :--- | :--- | :--- |
| `Space` | Open the **Leader Menu** (Commands) | Normal |
| `Space` → `h` | Show **Full Help Menu** | Leader |
| `Space` → `n` | Create a new **Box** | Leader |
| `Space` → `d` | Create a new **Diamond** | Leader |
| `Space` → `t` | Create a new **Text Node** | Leader |
| `Space` → `w` | **Write** (Save .txt and .json) | Leader |
| `i` | Enter **Insert Mode** to type inside a shape | Normal |
| `r` | Enter **Resize Mode** (Use `+` / `-` keys) | Normal |
| `Esc` | Return to **Normal Mode** / Clear Selection | Any |

### 🔗 Making Connections
Connectors in **dxgmr** are smart. They automatically choose the best "entry/exit" point:
1.  **Select** your source node (use `Tab` to cycle).
2.  Press **`c`** (plain line) or **`a`** (arrow).
3.  Press **`Tab`** to highlight the target node.
4.  Press **`Enter`** to snap the link into place.
5.  *Tip: Select an existing connection and press `a` to toggle its arrowhead.*

## ┌──────────────────────────────────────┐
## │ KEYBOARD SHORTCUTS REFERENCE         │
## └──────────────────────────────────────┘

**Normal Mode**
*   `Arrows`: Move selected node (or pan the infinite canvas if nothing is selected).
*   `Tab` / `Shift+Tab`: Cycle selection between nodes.
*   `Backspace` / `Delete`: Delete the selected node (and its links) or a highlighted connection.
*   `i`: Edit text in selected node.
*   `r`: Resize selected node.

**Leader Menu (`Space`)**
*   `w`: Write (Save) the diagram as `.txt` and `.json`.
*   `c`: Copy the ASCII to the clipboard.
*   `h`: Toggle the Full Help Reference.
*   `q`: Quit.

## ┌──────────────┐
## │ INSTALLATION │
## └──────────────┘

### From Source (Recommended)
```bash
git clone https://github.com/AgemaLabs/dxgmr.git
cd dxgmr
cargo install --path .
```

---
*Built with ❤️ by a pair of Humans @AgemaLabs and AI Architects. Open Source and terminal-optimized.*