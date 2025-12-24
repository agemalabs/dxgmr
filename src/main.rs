use std::{io, time::Duration, fs};

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::{Backend, CrosstermBackend},
    widgets::{Block, Borders, Paragraph},
    Terminal,
};

mod model;
mod renderer;

use crate::model::{AppState, Node, ShapeType, AppMode};
use crate::renderer::render_to_canvas;

fn main() -> io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    let state = if args.len() > 1 {
        let cmd = &args[1];
        match cmd.as_str() {
            "new" => {
                let title = if args.len() > 2 { args[2..].join(" ") } else { "Untitled Diagram".to_string() };
                AppState::new(title)
            }
            "open" => {
                let title = if args.len() > 2 { args[2..].join(" ") } else { 
                    println!("Usage: dxgmr open <title>");
                    return Ok(());
                };
                let filename = format!("{}.json", title);
                match fs::read_to_string(&filename) {
                    Ok(data) => match serde_json::from_str(&data) {
                        Ok(diagram) => AppState::from_diagram(diagram),
                        Err(_) => {
                            println!("Error: Failed to parse {}. Starting new instead.", filename);
                            AppState::new(title)
                        }
                    },
                    Err(_) => {
                        println!("Error: File {} not found. Starting new instead.", filename);
                        AppState::new(title)
                    }
                }
            }
            _ => {
                let title = args[1..].join(" ");
                let filename = format!("{}.json", title);
                if let Ok(data) = fs::read_to_string(&filename) {
                    if let Ok(diagram) = serde_json::from_str(&data) {
                        AppState::from_diagram(diagram)
                    } else {
                        AppState::new(title)
                    }
                } else {
                    AppState::new(title)
                }
            }
        }
    } else {
        println!("Enter a title for your diagram:");
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let title = input.trim().to_string();
        if title.is_empty() {
            AppState::new("Untitled Diagram".to_string())
        } else {
            AppState::new(title)
        }
    };
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let res = run_app(&mut terminal, state);

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut state: AppState) -> io::Result<()> {
    let mut status_msg = String::from("Press <Space> for commands");
    
    loop {
        let mut inner_area_cache = ratatui::layout::Rect::default();
        let mut cursor_pos: Option<(u16, u16)> = None;
        let size = terminal.size()?;
        let area = ratatui::layout::Rect::new(0, 0, size.width, size.height);

        terminal.draw(|f| {
            let horizontal_chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Horizontal)
                .constraints([
                    ratatui::layout::Constraint::Length(area.width.saturating_sub(79) / 2),
                    ratatui::layout::Constraint::Length(79.min(area.width)),
                    ratatui::layout::Constraint::Min(0),
                ])
                .split(area);
            let display_area = horizontal_chunks[1];

            let chunks = ratatui::layout::Layout::default()
                .direction(ratatui::layout::Direction::Vertical)
                .constraints([
                    ratatui::layout::Constraint::Min(0),
                    ratatui::layout::Constraint::Length(1),
                ])
                .split(display_area);

            let main_area = chunks[0];
            let status_bar_area = chunks[1];

            // MAIN CANVAS
            let block = Block::default()
                .title(format!(" {} ", state.title))
                .borders(Borders::ALL)
                .border_style(match state.mode {
                    AppMode::Normal => ratatui::style::Style::default().fg(ratatui::style::Color::Blue),
                    AppMode::Insert(_) => ratatui::style::Style::default().fg(ratatui::style::Color::Green),
                    AppMode::Leader => ratatui::style::Style::default().fg(ratatui::style::Color::Yellow),
                    AppMode::Resize(_) => ratatui::style::Style::default().fg(ratatui::style::Color::Magenta),
                    AppMode::Help => ratatui::style::Style::default().fg(ratatui::style::Color::Cyan),
                    AppMode::ContextMenu { .. } => ratatui::style::Style::default().fg(ratatui::style::Color::White),
                });
            inner_area_cache = block.inner(main_area);
            f.render_widget(block, main_area);

            let canvas = render_to_canvas(&state, inner_area_cache.width, inner_area_cache.height);
            f.render_widget(Paragraph::new(canvas.to_string()), inner_area_cache);

            // STATUS BAR
            let (mode_text, mode_color) = match state.mode {
                AppMode::Normal => (" NORMAL ", ratatui::style::Color::Blue),
                AppMode::Insert(_) => (" INSERT ", ratatui::style::Color::Green),
                AppMode::Leader => (" LEADER ", ratatui::style::Color::Yellow),
                AppMode::Resize(_) => (" RESIZE ", ratatui::style::Color::Magenta),
                AppMode::Help => (" HELP ", ratatui::style::Color::Cyan),
                AppMode::ContextMenu { .. } => (" MENU ", ratatui::style::Color::White),
            };

            let status_bar = Paragraph::new(ratatui::text::Line::from(vec![
                ratatui::text::Span::styled(mode_text, ratatui::style::Style::default().bg(mode_color).fg(ratatui::style::Color::Black).add_modifier(ratatui::style::Modifier::BOLD)),
                ratatui::text::Span::raw(format!(" | {}", status_msg)),
            ])).style(ratatui::style::Style::default().bg(ratatui::style::Color::Indexed(235)));
            f.render_widget(status_bar, status_bar_area);

            // LEADER MENU (POPUP)
            if state.mode == AppMode::Leader {
                let popup_area = ratatui::layout::Rect {
                    x: area.width / 2 - 15,
                    y: area.height / 2 - 5,
                    width: 30,
                    height: 9,
                };
                let menu_block = Block::default()
                    .title(" Commands ")
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Yellow));
                
                let menu_text = vec![
                    ratatui::text::Line::from("  n -> New Box"),
                    ratatui::text::Line::from("  d -> New Diamond"),
                    ratatui::text::Line::from("  t -> New Text"),
                    ratatui::text::Line::from("  f -> New Frame"),
                    ratatui::text::Line::from(format!("  w -> Write ({}.txt/.json)", state.title)),
                    ratatui::text::Line::from("  c -> Copy to Clipboard"),
                    ratatui::text::Line::from("  h -> Help Menu"),
                    ratatui::text::Line::from("  q -> Quit"),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from("  <Esc> -> Cancel"),
                ];
                let menu = Paragraph::new(menu_text).block(menu_block);
                f.render_widget(ratatui::widgets::Clear, popup_area);
                f.render_widget(menu, popup_area);
            }

            // HELP MENU (POPUP)
            if state.mode == AppMode::Help {
                let popup_area = ratatui::layout::Rect {
                    x: area.width / 2 - 25,
                    y: area.height / 2 - 12,
                    width: 50,
                    height: 24,
                };
                let help_block = Block::default()
                    .title(" Full Command Reference ")
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));
                
                let help_text = vec![
                    ratatui::text::Line::from(ratatui::text::Span::styled("--- NAVIGATION & SELECTION ---", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))),
                    ratatui::text::Line::from("  Tab / BackTab   : Cycle through shapes"),
                    ratatui::text::Line::from("  Arrows          : Move shape or pan canvas"),
                    ratatui::text::Line::from("  Esc             : Clear selection / Back to Normal"),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(ratatui::text::Span::styled("--- EDITING ---", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))),
                    ratatui::text::Line::from("  i               : Enter Insert mode (Edit text)"),
                    ratatui::text::Line::from("  r               : Enter Resize mode (+/- to scale)"),
                    ratatui::text::Line::from("  Del / Backspace : Delete selected shape/connection"),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(ratatui::text::Span::styled("--- CONNECTORS ---", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))),
                    ratatui::text::Line::from("  c               : Start plain connector from shape"),
                    ratatui::text::Line::from("  a               : Start arrow connector from shape"),
                    ratatui::text::Line::from("  Enter           : Finish connector on target shape"),
                    ratatui::text::Line::from("  a (on conn)     : Toggle arrow on selection"),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(ratatui::text::Span::styled("--- COMMANDS (<Leader> = Space) ---", ratatui::style::Style::default().add_modifier(ratatui::style::Modifier::BOLD))),
                    ratatui::text::Line::from("  <Leader> + n    : Create new Box"),
                    ratatui::text::Line::from("  <Leader> + d    : Create new Diamond"),
                    ratatui::text::Line::from("  <Leader> + t    : Create new Text"),
                    ratatui::text::Line::from("  <Leader> + f    : Create new Frame"),
                    ratatui::text::Line::from("  <Leader> + w    : Save (.json and .txt)"),
                    ratatui::text::Line::from("  <Leader> + c    : Copy ASCII to clipboard"),
                    ratatui::text::Line::from(""),
                    ratatui::text::Line::from(ratatui::text::Span::styled("  Press <Esc> or <Space> to close Help", ratatui::style::Style::default().fg(ratatui::style::Color::Yellow))),
                ];
                let help = Paragraph::new(help_text).block(help_block);
                f.render_widget(ratatui::widgets::Clear, popup_area);
                f.render_widget(help, popup_area);
            }

            // CONTEXT MENU (MOUSE)
            if let AppMode::ContextMenu { x, y, selected_index } = state.mode {
                let items = vec![
                    " New Box ",
                    " New Diamond ",
                    " New Text ",
                    " New Frame ",
                    "---------",
                    " Start Connector ",
                    " Start Arrow ",
                    " Delete ",
                    "---------",
                    " Cancel "
                ];
                
                let width = 21;
                let height = items.len() as u16 + 2;
                
                // Adjust for terminal positioning
                let screen_x = inner_area_cache.x + x;
                let screen_y = inner_area_cache.y + y;

                // Keep menu on screen
                let menu_x = if screen_x + width > area.width { area.width.saturating_sub(width) } else { screen_x };
                let menu_y = if screen_y + height > area.height { area.height.saturating_sub(height) } else { screen_y };

                let popup_area = ratatui::layout::Rect {
                    x: menu_x,
                    y: menu_y,
                    width,
                    height,
                };

                let menu_block = Block::default()
                    .borders(Borders::ALL)
                    .border_style(ratatui::style::Style::default().fg(ratatui::style::Color::White));
                
                let menu_text: Vec<ratatui::text::Line> = items.iter().enumerate().map(|(i, &item)| {
                    if i == selected_index {
                        ratatui::text::Line::from(ratatui::text::Span::styled(
                            format!("> {}", item),
                            ratatui::style::Style::default().bg(ratatui::style::Color::White).fg(ratatui::style::Color::Black)
                        ))
                    } else {
                        ratatui::text::Line::from(format!("  {}", item))
                    }
                }).collect();

                let menu = Paragraph::new(menu_text).block(menu_block);
                f.render_widget(ratatui::widgets::Clear, popup_area);
                f.render_widget(menu, popup_area);
            }

            // CURSOR
            if let AppMode::Insert(id) = state.mode {
                if let Some(node) = state.nodes.iter().find(|n| n.id == id) {
                    let available_width = match node.shape {
                        ShapeType::Box => node.width.saturating_sub(2),
                        ShapeType::Diamond => node.width.saturating_sub(6).max(1),
                        ShapeType::Text => node.width,
                        ShapeType::Frame => node.width.saturating_sub(2),
                    };
                    let lines = crate::model::wrap_text(&node.text, available_width);
                    let lines = if lines.is_empty() { vec![String::new()] } else { lines };
                    let total_lines = lines.len() as u16;
                    let (_available_height, start_y) = match node.shape {
                        ShapeType::Text => (node.height, node.y),
                        _ => {
                            let ah = node.height.saturating_sub(2).max(1);
                            let sy = node.y + 1 + (ah.saturating_sub(total_lines)) / 2;
                            (ah, sy)
                        }
                    };
                    
                    let last_line_idx = lines.len().saturating_sub(1);
                    let last_line = &lines[last_line_idx];
                    let ty = start_y + last_line_idx as u16;
                    let text_start_x = node.x + (node.width.saturating_sub(last_line.len() as u16)) / 2;
                    let tx = text_start_x + last_line.len() as u16;
                    cursor_pos = Some((inner_area_cache.x + tx, inner_area_cache.y + ty));
                }
            }
        })?;

        if let Some((cx, cy)) = cursor_pos {
            terminal.show_cursor()?;
            terminal.set_cursor_position((cx, cy))?;
        } else {
            terminal.hide_cursor()?;
        }

        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    match state.mode {
                        AppMode::Insert(id) => {
                            match key.code {
                                KeyCode::Esc => { 
                                    state.mode = AppMode::Normal; 
                                    for n in &mut state.nodes { n.selected = false; }
                                    continue; 
                                }
                                KeyCode::Tab => {
                                    state.mode = AppMode::Normal;
                                    if !state.nodes.is_empty() {
                                        let current_idx = state.nodes.iter().position(|n| n.id == id);
                                        let next_idx = match current_idx {
                                            Some(idx) => (idx + 1) % state.nodes.len(),
                                            None => 0,
                                        };
                                        for (i, n) in state.nodes.iter_mut().enumerate() { n.selected = i == next_idx; }
                                        state.selected_connection_index = None;
                                    }
                                    continue;
                                }
                                _ => {}
                            }

                            if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                                match key.code {
                                    KeyCode::Char(c) => {
                                        node.text.push(c);
                                        if node.shape == ShapeType::Text {
                                            let lines: Vec<&str> = node.text.split('\n').collect();
                                            node.width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
                                            node.height = lines.len() as u16;
                                        }
                                    }
                                    KeyCode::Backspace => {
                                        node.text.pop();
                                        if node.shape == ShapeType::Text {
                                            let lines: Vec<&str> = node.text.split('\n').collect();
                                            node.width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
                                            node.height = lines.len() as u16;
                                        }
                                    }
                                    KeyCode::Enter => {
                                        node.text.push('\n');
                                        if node.shape == ShapeType::Text {
                                            let lines: Vec<&str> = node.text.split('\n').collect();
                                            node.width = lines.iter().map(|l| l.len()).max().unwrap_or(0) as u16;
                                            node.height = lines.len() as u16;
                                        }
                                    }
                                    _ => {}
                                }
                            } else {
                                state.mode = AppMode::Normal;
                            }
                        }
                        AppMode::Leader => {
                            match key.code {
                                KeyCode::Char('n') | KeyCode::Char('d') | KeyCode::Char('t') => {
                                    let mut spawn_x = 10;
                                    let mut spawn_y = 10;

                                    if let Some(last) = state.nodes.last() {
                                        spawn_x = last.x;
                                        spawn_y = last.y + last.height + 2;
                                    }

                                    let world_x = spawn_x as i32;
                                    let world_y = spawn_y as i32;

                                    let id = state.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                                    let shape = match key.code {
                                        KeyCode::Char('n') => ShapeType::Box,
                                        KeyCode::Char('d') => ShapeType::Diamond,
                                        KeyCode::Char('f') => ShapeType::Frame,
                                        _ => ShapeType::Text,
                                    };
                                    state.nodes.push(Node {
                                        id,
                                        shape,
                                        x: world_x.max(0) as u16,
                                        y: world_y.max(0) as u16,
                                        width: if shape == ShapeType::Text { 10 } else if shape == ShapeType::Box { 20 } else if shape == ShapeType::Frame { 30 } else { 15 },
                                        height: if shape == ShapeType::Text { 1 } else if shape == ShapeType::Box { 5 } else if shape == ShapeType::Frame { 10 } else { 7 },
                                        text: String::new(),
                                        selected: true,
                                    });
                                    state.mode = AppMode::Insert(id);
                                    for n in &mut state.nodes { if n.id != id { n.selected = false; } }
                                    state.selected_connection_index = None;
                                    status_msg = String::from("New shape created below previous");
                                }
                                KeyCode::Char('h') => {
                                    state.mode = AppMode::Help;
                                }
                                KeyCode::Char('w') | KeyCode::Char('c') => {
                                    if key.code == KeyCode::Char('c') {
                                        let canvas = render_to_canvas(&state, 79, inner_area_cache.height);
                                        let text = canvas.to_string();
                                        if let Ok(mut clipboard) = arboard::Clipboard::new() {
                                            let _ = clipboard.set_text(text);
                                            status_msg = String::from("Copied to clipboard!");
                                        }
                                    } else {
                                        // Save ASCII .txt
                                        let canvas = render_to_canvas(&state, 79, inner_area_cache.height);
                                        let text = canvas.to_string();
                                        let txt_filename = format!("{}.txt", state.title);
                                        let _ = fs::write(&txt_filename, text);

                                        // Save Model .json
                                        let diagram = state.to_diagram();
                                        if let Ok(json) = serde_json::to_string_pretty(&diagram) {
                                            let json_filename = format!("{}.json", state.title);
                                            if fs::write(&json_filename, json).is_ok() {
                                                status_msg = format!("Saved {} and {}!", txt_filename, json_filename);
                                            }
                                        }
                                    }
                                    state.mode = AppMode::Normal;
                                }
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Esc => { state.mode = AppMode::Normal; }
                                _ => {}
                            }
                        }
                        AppMode::Help => {
                            match key.code {
                                KeyCode::Esc | KeyCode::Char(' ') | KeyCode::Enter => {
                                    state.mode = AppMode::Normal;
                                }
                                _ => {}
                            }
                        }
                        AppMode::Resize(id) => {
                            if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                                match key.code {
                                    KeyCode::Char('+') | KeyCode::Char('=') => {
                                        node.width += 2;
                                        node.height += 1;
                                        status_msg = format!("Resized: {}x{}", node.width, node.height);
                                    }
                                    KeyCode::Char('-') | KeyCode::Char('_') => {
                                        node.width = (node.width.saturating_sub(2)).max(3);
                                        node.height = (node.height.saturating_sub(1)).max(1);
                                        status_msg = format!("Resized: {}x{}", node.width, node.height);
                                    }
                                    KeyCode::Esc | KeyCode::Enter => {
                                        state.mode = AppMode::Normal;
                                        status_msg = String::from("Resize finished");
                                    }
                                    _ => {}
                                }
                            } else {
                                state.mode = AppMode::Normal;
                            }
                        }
                        AppMode::ContextMenu { x, y, mut selected_index } => {
                            match key.code {
                                KeyCode::Up => {
                                    if selected_index > 0 {
                                        selected_index -= 1;
                                        if selected_index == 4 || selected_index == 8 { selected_index -= 1; }
                                        state.mode = AppMode::ContextMenu { x, y, selected_index };
                                    }
                                }
                                KeyCode::Down => {
                                    if selected_index < 9 {
                                        selected_index += 1;
                                        if selected_index == 4 || selected_index == 8 { selected_index += 1; }
                                        state.mode = AppMode::ContextMenu { x, y, selected_index };
                                    }
                                }
                                KeyCode::Enter | KeyCode::Char(' ') => {
                                    let id = state.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                                    let world_x = (x as i32 + state.camera_offset.0).max(0) as u16;
                                    let world_y = (y as i32 + state.camera_offset.1).max(0) as u16;
                                    
                                    match selected_index {
                                        0 => { // New Box
                                            state.nodes.push(Node { id, shape: ShapeType::Box, x: world_x, y: world_y, width: 20, height: 5, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        1 => { // New Diamond
                                            state.nodes.push(Node { id, shape: ShapeType::Diamond, x: world_x, y: world_y, width: 15, height: 7, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        2 => { // New Text
                                            state.nodes.push(Node { id, shape: ShapeType::Text, x: world_x, y: world_y, width: 10, height: 1, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        3 => { // New Frame
                                            state.nodes.push(Node { id, shape: ShapeType::Frame, x: world_x, y: world_y, width: 30, height: 10, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        5 => { // Start Connector
                                            if let Some(node) = state.nodes.iter().rev().find(|n| n.contains(world_x, world_y)) {
                                                state.connection_source_id = Some(node.id);
                                                state.connection_has_arrow = false;
                                                status_msg = format!("Connector source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                            } else {
                                                status_msg = String::from("No node at click position");
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        6 => { // Start Arrow
                                            if let Some(node) = state.nodes.iter().rev().find(|n| n.contains(world_x, world_y)) {
                                                state.connection_source_id = Some(node.id);
                                                state.connection_has_arrow = true;
                                                status_msg = format!("Arrow source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                            } else {
                                                status_msg = String::from("No node at click position");
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        7 => { // Delete
                                            if let Some(idx) = state.nodes.iter().position(|n| n.contains(world_x, world_y)) {
                                                let node_id = state.nodes[idx].id;
                                                state.nodes.remove(idx);
                                                state.connections.retain(|c| c.from_id != node_id && c.to_id != node_id);
                                                status_msg = String::from("Shape and connections deleted");
                                            } else {
                                                for (i, conn) in state.connections.iter().enumerate().rev() {
                                                    if conn.contains(world_x, world_y, &state.nodes) {
                                                        state.connections.remove(i);
                                                        status_msg = String::from("Connection deleted");
                                                        break;
                                                    }
                                                }
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        9 => { state.mode = AppMode::Normal; }
                                        _ => { state.mode = AppMode::Normal; }
                                    }
                                    if selected_index < 4 {
                                        for n in &mut state.nodes { if n.id != id { n.selected = false; } }
                                        state.selected_connection_index = None;
                                    }
                                }
                                KeyCode::Esc => {
                                    state.mode = AppMode::Normal;
                                }
                                _ => {}
                            }
                        }
                        AppMode::Normal => {
                            match key.code {
                                KeyCode::Esc => {
                                    state.connection_source_id = None;
                                    state.selected_connection_index = None;
                                    for n in &mut state.nodes { n.selected = false; }
                                    status_msg = String::from("Selection cleared");
                                }
                                KeyCode::Char(' ') => { state.mode = AppMode::Leader; }
                                KeyCode::Char('q') => return Ok(()),
                                KeyCode::Char('i') => {
                                    if let Some(node) = state.nodes.iter().find(|n| n.selected) {
                                        state.mode = AppMode::Insert(node.id);
                                    }
                                }
                                KeyCode::Tab => {
                                    if !state.nodes.is_empty() {
                                        let current_idx = state.nodes.iter().position(|n| n.selected);
                                        let next_idx = match current_idx {
                                            Some(idx) => (idx + 1) % state.nodes.len(),
                                            None => 0,
                                        };
                                        for (i, n) in state.nodes.iter_mut().enumerate() { n.selected = i == next_idx; }
                                        state.selected_connection_index = None;
                                    }
                                }
                                KeyCode::BackTab => {
                                    if !state.nodes.is_empty() {
                                        let current_idx = state.nodes.iter().position(|n| n.selected);
                                        let next_idx = match current_idx {
                                            Some(idx) => (idx + state.nodes.len() - 1) % state.nodes.len(),
                                            None => state.nodes.len() - 1,
                                        };
                                        for (i, n) in state.nodes.iter_mut().enumerate() { n.selected = i == next_idx; }
                                        state.selected_connection_index = None;
                                    }
                                }
                                KeyCode::Char('r') => {
                                    if let Some(node) = state.nodes.iter().find(|n| n.selected) {
                                        state.mode = AppMode::Resize(node.id);
                                        status_msg = String::from("Resize Mode: Use +/- to scale, Esc to finish");
                                    }
                                }
                                KeyCode::Delete | KeyCode::Backspace => {
                                    if let Some(idx) = state.selected_connection_index {
                                        state.connections.remove(idx);
                                        state.selected_connection_index = None;
                                        status_msg = String::from("Connection deleted");
                                    } else if let Some(idx) = state.nodes.iter().position(|n| n.selected) {
                                        let node_id = state.nodes[idx].id;
                                        state.nodes.remove(idx);
                                        state.connections.retain(|c| c.from_id != node_id && c.to_id != node_id);
                                        status_msg = String::from("Shape and connections deleted");
                                    }
                                }
                                KeyCode::Char('c') => {
                                    if let Some(node) = state.nodes.iter().find(|n| n.selected) {
                                        state.connection_source_id = Some(node.id);
                                        state.connection_has_arrow = false;
                                        status_msg = format!("Connector source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                    }
                                }
                                KeyCode::Enter => {
                                    if let Some(src_id) = state.connection_source_id {
                                        if let Some(target_node) = state.nodes.iter().find(|n| n.selected) {
                                            if target_node.id != src_id {
                                                if let Some(src_node) = state.nodes.iter().find(|n| n.id == src_id) {
                                                    // Smart heuristic based on relative position
                                                    let from_offset;
                                                    let to_offset;
                                                    
                                                    if target_node.y >= src_node.y + src_node.height {
                                                        // Target is below
                                                        from_offset = (src_node.width / 2, src_node.height - 1);
                                                        to_offset = (target_node.width / 2, 0);
                                                    } else if target_node.x >= src_node.x + src_node.width {
                                                        // Target is to the right
                                                        from_offset = (src_node.width - 1, src_node.height / 2);
                                                        to_offset = (0, target_node.height / 2);
                                                    } else if src_node.y >= target_node.y + target_node.height {
                                                        // Target is above
                                                        from_offset = (src_node.width / 2, 0);
                                                        to_offset = (target_node.width / 2, target_node.height - 1);
                                                    } else {
                                                        // Target is to the left
                                                        from_offset = (0, src_node.height / 2);
                                                        to_offset = (target_node.width - 1, target_node.height / 2);
                                                    }

                                                    state.connections.push(crate::model::Connection {
                                                        from_id: src_id,
                                                        from_offset,
                                                        to_id: target_node.id,
                                                        to_offset,
                                                        has_arrow: state.connection_has_arrow,
                                                    });
                                                    state.connection_source_id = None;
                                                    status_msg = String::from("Keyboard connection created!");
                                                }
                                            }
                                        }
                                    }
                                }
                                KeyCode::Char('a') => {
                                    if let Some(idx) = state.selected_connection_index {
                                        state.connections[idx].has_arrow = !state.connections[idx].has_arrow;
                                        status_msg = if state.connections[idx].has_arrow { String::from("Arrow enabled") } else { String::from("Arrow disabled") };
                                    } else if let Some(node) = state.nodes.iter().find(|n| n.selected) {
                                        state.connection_source_id = Some(node.id);
                                        state.connection_has_arrow = true;
                                        status_msg = format!("Arrow source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                    } else {
                                        status_msg = String::from("Select a node (a) for Arrow or connection (a) to toggle");
                                    }
                                }
                                KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                                    if let Some(node) = state.nodes.iter_mut().find(|n| n.selected) {
                                        match key.code {
                                            KeyCode::Up => node.y = node.y.saturating_sub(1),
                                            KeyCode::Down => node.y += 1,
                                            KeyCode::Left => node.x = node.x.saturating_sub(1),
                                            KeyCode::Right => node.x += 1,
                                            _ => {}
                                        }
                                    } else {
                                        // Pan the camera if no node is selected
                                        match key.code {
                                            KeyCode::Up => state.camera_offset.1 = state.camera_offset.1.saturating_sub(1),
                                            KeyCode::Down => state.camera_offset.1 += 1,
                                            KeyCode::Left => state.camera_offset.0 = state.camera_offset.0.saturating_sub(1),
                                            KeyCode::Right => state.camera_offset.0 += 1,
                                            _ => {}
                                        }
                                        status_msg = format!("Canvas Pan: {}, {}", state.camera_offset.0, state.camera_offset.1);
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                Event::Mouse(mouse) => {
                    if mouse.column < inner_area_cache.x || mouse.row < inner_area_cache.y {
                        continue;
                    }
                    let mx_screen = mouse.column - inner_area_cache.x;
                    let my_screen = mouse.row - inner_area_cache.y;
                    
                    let mx = (mx_screen as i32 + state.camera_offset.0).max(0) as u16;
                    let my = (my_screen as i32 + state.camera_offset.1).max(0) as u16;

                    // --- CONTEXT MENU HANDLING ---
                    if let AppMode::ContextMenu { x, y, .. } = state.mode {
                        let width = 21;
                        let height = 11; // items.len() + 2
                        let screen_x = inner_area_cache.x + x;
                        let screen_y = inner_area_cache.y + y;
                        let menu_x = if screen_x + width > area.width { area.width.saturating_sub(width) } else { screen_x };
                        let menu_y = if screen_y + height > area.height { area.height.saturating_sub(height) } else { screen_y };

                        if mouse.column >= menu_x && mouse.column < menu_x + width &&
                           mouse.row >= menu_y && mouse.row < menu_y + height {
                            let local_y = mouse.row.saturating_sub(menu_y).saturating_sub(1);
                            if local_y < 10 && local_y != 4 && local_y != 8 {
                                state.mode = AppMode::ContextMenu { x, y, selected_index: local_y as usize };
                                if matches!(mouse.kind, event::MouseEventKind::Down(event::MouseButton::Left)) {
                                    let id = state.nodes.iter().map(|n| n.id).max().unwrap_or(0) + 1;
                                    let world_x = (x as i32 + state.camera_offset.0).max(0) as u16;
                                    let world_y = (y as i32 + state.camera_offset.1).max(0) as u16;
                                    
                                    match local_y {
                                        0 => { // New Box
                                            state.nodes.push(Node { id, shape: ShapeType::Box, x: world_x, y: world_y, width: 20, height: 5, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        1 => { // New Diamond
                                            state.nodes.push(Node { id, shape: ShapeType::Diamond, x: world_x, y: world_y, width: 15, height: 7, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        2 => { // New Text
                                            state.nodes.push(Node { id, shape: ShapeType::Text, x: world_x, y: world_y, width: 10, height: 1, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        3 => { // New Frame
                                            state.nodes.push(Node { id, shape: ShapeType::Frame, x: world_x, y: world_y, width: 30, height: 10, text: String::new(), selected: true });
                                            state.mode = AppMode::Insert(id);
                                        }
                                        5 => { // Start Connector
                                            if let Some(node) = state.nodes.iter().rev().find(|n| n.contains(world_x, world_y)) {
                                                state.connection_source_id = Some(node.id);
                                                state.connection_has_arrow = false;
                                                status_msg = format!("Connector source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                            } else {
                                                status_msg = String::from("No node at click position");
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        6 => { // Start Arrow
                                            if let Some(node) = state.nodes.iter().rev().find(|n| n.contains(world_x, world_y)) {
                                                state.connection_source_id = Some(node.id);
                                                state.connection_has_arrow = true;
                                                status_msg = format!("Arrow source: {}. Tab to target, Enter to finish.", node.text.split_whitespace().next().unwrap_or("Node"));
                                            } else {
                                                status_msg = String::from("No node at click position");
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        7 => { // Delete
                                            if let Some(idx) = state.nodes.iter().position(|n| n.contains(world_x, world_y)) {
                                                let node_id = state.nodes[idx].id;
                                                state.nodes.remove(idx);
                                                state.connections.retain(|c| c.from_id != node_id && c.to_id != node_id);
                                                status_msg = String::from("Shape and connections deleted");
                                            } else {
                                                for (i, conn) in state.connections.iter().enumerate().rev() {
                                                    if conn.contains(world_x, world_y, &state.nodes) {
                                                        state.connections.remove(i);
                                                        status_msg = String::from("Connection deleted");
                                                        break;
                                                    }
                                                }
                                            }
                                            state.mode = AppMode::Normal;
                                        }
                                        9 => { state.mode = AppMode::Normal; }
                                        _ => { state.mode = AppMode::Normal; }
                                    }
                                    if local_y < 4 {
                                        for n in &mut state.nodes { if n.id != id { n.selected = false; } }
                                        state.selected_connection_index = None;
                                    }
                                    continue;
                                }
                            }
                            if !matches!(mouse.kind, event::MouseEventKind::Down(event::MouseButton::Right)) {
                                continue;
                            }
                        } else if matches!(mouse.kind, event::MouseEventKind::Down(event::MouseButton::Left)) {
                            state.mode = AppMode::Normal;
                        }
                    }

                    if matches!(mouse.kind, event::MouseEventKind::Down(event::MouseButton::Right)) {
                        state.mode = AppMode::ContextMenu { x: mx_screen, y: my_screen, selected_index: 0 };
                        continue;
                    }
                    // --- END CONTEXT MENU HANDLING ---

                    match mouse.kind {
                        event::MouseEventKind::Down(event::MouseButton::Left) => {
                            state.dragging_node_id = None;
                            state.resizing_node_id = None;
                            state.partial_connection = None;
                            
                            let mut hit_node_id = None;
                            let mut is_border = false;
                            let mut is_corner = false;
                            let mut node_offset = (0, 0);

                            for node in state.nodes.iter().rev() {
                                if node.contains(mx, my) {
                                    hit_node_id = Some(node.id);
                                    node_offset = (mx - node.x, my - node.y);
                                    if mx == node.x + node.width - 1 && my == node.y + node.height - 1 {
                                        is_corner = true;
                                    } else if mx == node.x || mx == node.x + node.width - 1 || 
                                              my == node.y || my == node.y + node.height - 1 {
                                        is_border = true;
                                    }
                                    break;
                                }
                            }

                            if let Some(id) = hit_node_id {
                                if is_corner {
                                    state.resizing_node_id = Some(id);
                                } else if is_border {
                                    if let Some(node) = state.nodes.iter().find(|n| n.id == id) {
                                        let snapped_offset = if node_offset.1 == 0 { (node.width / 2, 0) }
                                            else if node_offset.1 == node.height - 1 { (node.width / 2, node.height - 1) }
                                            else if node_offset.0 == 0 { (0, node.height / 2) }
                                            else { (node.width - 1, node.height / 2) };

                                        state.partial_connection = Some(crate::model::PartialConnection::Starting {
                                            from_id: id,
                                            from_offset: snapped_offset,
                                            current_pos: (mx, my),
                                        });
                                    }
                                } else {
                                    state.dragging_node_id = Some(id);
                                    state.drag_offset = node_offset;
                                    if let Some(idx) = state.nodes.iter().position(|n| n.id == id) {
                                        for n in &mut state.nodes { n.selected = false; }
                                        state.nodes[idx].selected = true;
                                        let node = state.nodes.remove(idx);
                                        state.nodes.push(node);
                                    }
                                }
                            } else {
                                state.mode = AppMode::Normal;
                                state.selected_connection_index = None;
                                for n in &mut state.nodes { n.selected = false; }
                                for (i, conn) in state.connections.iter().enumerate().rev() {
                                    if conn.contains(mx, my, &state.nodes) {
                                        state.selected_connection_index = Some(i);
                                        status_msg = String::from("Connection selected | 'a': Arrow | 'Del': Remove");
                                        break;
                                    }
                                }
                            }
                        }
                        event::MouseEventKind::Drag(event::MouseButton::Left) => {
                            if let Some(pc) = &mut state.partial_connection {
                                match pc { crate::model::PartialConnection::Starting { current_pos, .. } => { *current_pos = (mx, my); } }
                            } else if let Some(id) = state.resizing_node_id {
                                if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                                    node.width = (mx.saturating_sub(node.x) + 1).max(3);
                                    node.height = (my.saturating_sub(node.y) + 1).max(3);
                                }
                            } else if let Some(id) = state.dragging_node_id {
                                if let Some(node) = state.nodes.iter_mut().find(|n| n.id == id) {
                                    node.x = mx.saturating_sub(state.drag_offset.0);
                                    node.y = my.saturating_sub(state.drag_offset.1);
                                    node.x = node.x.min(inner_area_cache.width.saturating_sub(node.width));
                                    node.y = node.y.min(inner_area_cache.height.saturating_sub(node.height));
                                }
                            }
                        }
                        event::MouseEventKind::Up(event::MouseButton::Left) => {
                            if let Some(crate::model::PartialConnection::Starting { from_id, from_offset, .. }) = state.partial_connection {
                                for node in &state.nodes {
                                    if node.id != from_id && node.contains(mx, my) {
                                        let dx_left = mx.saturating_sub(node.x);
                                        let dx_right = (node.x + node.width - 1).saturating_sub(mx);
                                        let dy_top = my.saturating_sub(node.y);
                                        let dy_bottom = (node.y + node.height - 1).saturating_sub(my);
                                        let min_dist = dx_left.min(dx_right).min(dy_top).min(dy_bottom);
                                        let to_offset = if min_dist == dy_top { (node.width / 2, 0) }
                                            else if min_dist == dy_bottom { (node.width / 2, node.height - 1) }
                                            else if min_dist == dx_left { (0, node.height / 2) }
                                            else { (node.width - 1, node.height / 2) };

                                        state.connections.push(crate::model::Connection { from_id, from_offset, to_id: node.id, to_offset, has_arrow: true });
                                        break;
                                    }
                                }
                            } else if let Some(id) = state.dragging_node_id {
                                state.mode = AppMode::Insert(id);
                            }
                            state.dragging_node_id = None;
                            state.resizing_node_id = None;
                            state.partial_connection = None;
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
