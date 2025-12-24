# dxgmr: The ASCII Architect

> "Why use a 50MB SVG when 79 characters of ASCII can explain your logic?"

**dxgmr** (pronounced "diagrammer") is a lightweight, terminal-based tool designed for developers who want to create professional-grade ASCII diagrams without leaving their environment. It bridges the gap between high-overhead GUI tools and the painstaking manual labor of "drawing" with your spacebar.

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

## 🎨 The Rationale

*   **⚡ Speed**: Built for developers who hate reaching for the mouse. With a "Type, Tab, Link" workflow, you can map out a complex system in seconds.
*   **🧩 Simplicity**: The ultimate "analog" tool. It doesn't save to proprietary formats; it saves to `.txt`. Your diagrams are version-controllable, searchable, and instantly viewable on GitHub.
*   **🔗 Analog Aesthetics**: There is a unique clarity to fixed-width ASCII. It belongs in your code comments, READMEs, and technical documentation.

## ✨ Features

*   **Seamless Grid**: Automatically constrained to a **79-character width**, ensuring your diagrams never wrap or break layout in GitHub READMEs or mobile TUI views.
*   **Smart Staircase Routing**: Implements professional "Z-mode" and "S-mode" routing with automatic right-angles. It's not just lines; it's architecture.
*   **Modal Editing**: Inspired by Vim. Switch between `Normal`, `Insert`, `Leader`, and `Resize` modes effortlessly.
*   **Hybrid Input**: Fully mouse-draggable for fine-tuning, but 100% keyboard-operable for high-speed drafting.
*   **One-Click Export**: Save direct to a file named after your diagram or copy the raw ASCII to your clipboard instantly.

## 🚀 How to Use

### 1. The Startup
Run the binary and provide a title for your diagram. This title becomes the filename when you save.
```bash
dxgmr "System Architecture"
```

### 2. The Keyboard Workflow
The primary power of **dxgmr** lies in its **Leader Key** system (the `Spacebar`).

| Key | Action | Mode |
| :--- | :--- | :--- |
| `Space` | Open the **Leader Menu** (Commands) | Normal |
| `Space` → `n` | Create a new **Box** (Process) | Leader |
| `Space` → `d` | Create a new **Diamond** (Decision) | Leader |
| `Space` → `t` | Create a new **Text Node** (Borderless annotation) | Leader |
| `i` | Enter **Insert Mode** to type inside a shape | Normal |
| `Tab` | (In Insert Mode) Finish text & skip to next node | Insert |
| `Esc` | Return to **Normal Mode** / Clear Selection | Any |
| `r` | Enter **Resize Mode** (Use `+` / `-` keys) | Normal |

### 3. Making Connections
Connectors in **dxgmr** are smart. They automatically choose the best "entry/exit" point based on the relative position of the nodes:
1.  **Select** your source node.
2.  Press **`c`** (for a plain line) or **`a`** (for an arrow).
3.  Press **`Tab`** to highlight the target node.
4.  Press **`Enter`** to snap the link into place.

## 💻 Keyboard Shortcuts Reference

**Normal Mode**
*   `Arrows`: Move selected node (or pan the infinite canvas if nothing is selected).
*   `Tab` / `Shift+Tab`: Cycle selection between nodes.
*   `Backspace` / `Delete`: Delete the selected node (and its links) or a highlighted connection.
*   `a`: Toggle an arrowhead on an already selected connection.

**Leader Menu (`Space`)**
*   `w`: Write (Save) the diagram as a `.txt` file.
*   `c`: Copy the diagram ASCII to the clipboard.
*   `q`: Quit the application.

## 🛠 Installation
Since **dxgmr** is built in Rust, you can install it via cargo:

```bash
cargo install dxgmr
```

---
*Built with ❤️ by a pair of Human and AI Architects. Open Source and terminal-optimized.*