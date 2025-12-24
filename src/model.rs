use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ShapeType {
    Box,      // Rectangular
    Diamond,  // Decision
    Text,     // Borderless text
    Frame,    // Grouping frame with title
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: usize,
    pub shape: ShapeType,
    pub x: u16,      // Col
    pub y: u16,      // Row
    pub width: u16,
    pub height: u16,
    pub text: String,
    pub selected: bool,
}

impl Node {
    pub fn contains(&self, x: u16, y: u16) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub from_id: usize,
    pub from_offset: (u16, u16), // Relative to node top-left
    pub to_id: usize,
    pub to_offset: (u16, u16),   // Relative to node top-left
    pub has_arrow: bool,
}

impl Connection {
    pub fn contains(&self, mx: u16, my: u16, nodes: &[Node]) -> bool {
        let from = nodes.iter().find(|n| n.id == self.from_id);
        let to = nodes.iter().find(|n| n.id == self.to_id);

        if let (Some(f), Some(t)) = (from, to) {
            let x1 = f.x + self.from_offset.0;
            let y1 = f.y + self.from_offset.1;
            let x2 = t.x + self.to_offset.0;
            let y2 = t.y + self.to_offset.1;

            let vertical_first = self.from_offset.1 == 0 || self.from_offset.1 == f.height - 1;

            if vertical_first {
                let mid_y = (y1 + y2) / 2;
                // V1
                if mx == x1 && my >= y1.min(mid_y) && my <= y1.max(mid_y) { return true; }
                // H
                if my == mid_y && mx >= x1.min(x2) && mx <= x1.max(x2) { return true; }
                // V2
                if mx == x2 && my >= mid_y.min(y2) && my <= mid_y.max(y2) { return true; }
            } else {
                let mid_x = (x1 + x2) / 2;
                // H1
                if my == y1 && mx >= x1.min(mid_x) && mx <= x1.max(mid_x) { return true; }
                // V
                if mx == mid_x && my >= y1.min(y2) && my <= y1.max(y2) { return true; }
                // H2
                if my == y2 && mx >= mid_x.min(x2) && mx <= mid_x.max(x2) { return true; }
            }
        }
        false
    }
}

pub enum PartialConnection {
    Starting {
        from_id: usize,
        from_offset: (u16, u16),
        current_pos: (u16, u16),
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppMode {
    Normal,
    Insert(usize), // Node ID being edited
    Leader,        // Spacebar hit, waiting for command
    Resize(usize), // Node ID being resized
    Help,          // Showing command help
    ContextMenu { x: u16, y: u16, selected_index: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    pub title: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
}

pub struct AppState {
    pub title: String,
    pub nodes: Vec<Node>,
    pub connections: Vec<Connection>,
    pub dragging_node_id: Option<usize>,
    pub drag_offset: (u16, u16),
    pub camera_offset: (i32, i32),
    pub partial_connection: Option<PartialConnection>,
    pub selected_connection_index: Option<usize>,
    pub resizing_node_id: Option<usize>,
    pub connection_source_id: Option<usize>,
    pub connection_has_arrow: bool,
    pub mode: AppMode,
}

impl AppState {
    pub fn new(title: String) -> Self {
        Self {
            title,
            nodes: Vec::new(),
            connections: Vec::new(),
            dragging_node_id: None,
            drag_offset: (0, 0),
            camera_offset: (0, 0),
            partial_connection: None,
            selected_connection_index: None,
            resizing_node_id: None,
            connection_source_id: None,
            connection_has_arrow: false,
            mode: AppMode::Normal,
        }
    }

    pub fn from_diagram(diagram: Diagram) -> Self {
        let mut state = Self::new(diagram.title);
        state.nodes = diagram.nodes;
        state.connections = diagram.connections;
        state
    }

    pub fn to_diagram(&self) -> Diagram {
        Diagram {
            title: self.title.clone(),
            nodes: self.nodes.clone(),
            connections: self.connections.clone(),
        }
    }
}

pub fn wrap_text(text: &str, max_width: u16) -> Vec<String> {
    if max_width == 0 { return Vec::new(); }
    let mut all_lines = Vec::new();
    
    for paragraph in text.split('\n') {
        let mut current_line = String::new();
        let mut paragraph_lines = Vec::new();
        
        for word in paragraph.split_inclusive(' ') {
            let is_too_long = (current_line.len() + word.len()) > max_width as usize;
            
            if is_too_long && !current_line.is_empty() {
                paragraph_lines.push(current_line);
                current_line = String::new();
            }
            
            let mut w = word;
            while w.len() > max_width as usize {
                let (part, rest) = w.split_at(max_width as usize);
                paragraph_lines.push(part.to_string());
                w = rest;
            }
            current_line.push_str(w);
        }
        
        if !current_line.is_empty() {
            paragraph_lines.push(current_line);
        }
        
        if paragraph_lines.is_empty() {
            all_lines.push(String::new());
        } else {
            all_lines.extend(paragraph_lines);
        }
    }
    
    if all_lines.is_empty() && !text.is_empty() {
        all_lines.push(String::new());
    }

    all_lines
}
