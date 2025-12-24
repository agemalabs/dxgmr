use crate::model::{AppState, Node, ShapeType};

pub struct Canvas {
    pub width: u16,
    pub height: u16,
    pub grid: Vec<Vec<char>>,
}

impl Canvas {
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            width,
            height,
            grid: vec![vec![' '; width as usize]; height as usize],
        }
    }

    pub fn set(&mut self, x: u16, y: u16, c: char) {
        if x < self.width && y < self.height {
            self.grid[y as usize][x as usize] = c;
        }
    }

    pub fn draw_text_node(&mut self, node: &Node) {
        let available_width = node.width;
        let available_height = node.height;
        let lines = crate::model::wrap_text(&node.text, available_width);
        let total_lines = lines.len() as u16;
        let start_y = node.y + (available_height.saturating_sub(total_lines)) / 2;
        
        for (i, line) in lines.iter().enumerate().take(available_height as usize) {
            let ty = start_y + i as u16;
            let text_start_x = node.x + (available_width.saturating_sub(line.len() as u16)) / 2;
            for (j, c) in line.chars().enumerate() {
                let tx = text_start_x + j as u16;
                self.set(tx, ty, c);
            }
        }
        
        if node.selected {
            self.set(node.x.saturating_sub(1), node.y, '[');
            self.set(node.x + node.width, node.y + node.height - 1, ']');
        }
    }

    pub fn draw_box(&mut self, node: &Node) {
        let x1 = node.x;
        let y1 = node.y;
        let x2 = x1 + node.width - 1;
        let y2 = y1 + node.height - 1;

        let corner = if node.selected { '#' } else { '+' };
        let horiz = if node.selected { '=' } else { '-' };
        let vert = if node.selected { '#' } else { '|' };

        // Corners
        self.set(x1, y1, corner);
        self.set(x2, y1, corner);
        self.set(x1, y2, corner);
        self.set(x2, y2, corner);

        // Horizontal lines
        for x in (x1 + 1)..x2 {
            self.set(x, y1, horiz);
            self.set(x, y2, horiz);
        }

        // Vertical lines
        for y in (y1 + 1)..y2 {
            self.set(x1, y, vert);
            self.set(x2, y, vert);
        }

        // Text wrapping
        let available_width = node.width.saturating_sub(2);
        let available_height = node.height.saturating_sub(2);
        if available_width > 0 && available_height > 0 {
            let lines = crate::model::wrap_text(&node.text, available_width);
            let total_lines = lines.len() as u16;
            
            // Start Y to center vertically
            let start_y = y1 + 1 + (available_height.saturating_sub(total_lines)) / 2;
            
            for (i, line) in lines.iter().enumerate().take(available_height as usize) {
                let ty = start_y + i as u16;
                if ty > y1 && ty < y2 {
                    let text_start_x = x1 + 1 + (available_width.saturating_sub(line.len() as u16)) / 2;
                    for (j, c) in line.chars().enumerate() {
                        let tx = text_start_x + j as u16;
                        if tx > x1 && tx < x2 {
                            self.set(tx, ty, c);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_diamond(&mut self, node: &Node) {
        let x1 = node.x;
        let y1 = node.y;
        let x2 = node.x + node.width - 1;
        let y2 = node.y + node.height - 1;
        let cx = x1 + node.width / 2;
        let cy = y1 + node.height / 2;

        let point = if node.selected { '#' } else { '+' };

        // Top to Right
        self.draw_line(cx, y1, x2, cy, if node.selected { '#' } else { '/' });
        // Right to Bottom
        self.draw_line(x2, cy, cx, y2, if node.selected { '#' } else { '\\' });
        // Bottom to Left
        self.draw_line(cx, y2, x1, cy, if node.selected { '#' } else { '/' });
        // Left to Top
        self.draw_line(x1, cy, cx, y1, if node.selected { '#' } else { '\\' });

        // Points
        self.set(cx, y1, point);
        self.set(cx, y2, point);
        self.set(x1, cy, point);
        self.set(x2, cy, point);

        // Text wrapping for Diamond
        let available_width = node.width.saturating_sub(6).max(1);
        let available_height = node.height.saturating_sub(2).max(1);
        
        let lines = crate::model::wrap_text(&node.text, available_width);
        let total_lines = lines.len() as u16;
        
        let start_y = y1 + 1 + (available_height.saturating_sub(total_lines)) / 2;
        
        for (i, line) in lines.iter().enumerate().take(available_height as usize) {
            let ty = start_y + i as u16;
            let text_start_x = x1 + (node.width.saturating_sub(line.len() as u16)) / 2;
            
            for (j, c) in line.chars().enumerate() {
                let tx = text_start_x + j as u16;
                if ty > y1 && ty < y2 && tx > x1 + 1 && tx < x2 - 1 {
                    self.set(tx, ty, c);
                }
            }
        }
    }

    fn draw_line(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, c: char) {
        let dx = (x2 as i32 - x1 as i32).abs();
        let dy = (y2 as i32 - y1 as i32).abs();
        let sx = if x1 < x2 { 1 } else { -1 };
        let sy = if y1 < y2 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x1 as i32;
        let mut y = y1 as i32;

        loop {
            if (x as u16 != x1 || y as u16 != y1) && (x as u16 != x2 || y as u16 != y2) {
                self.set(x as u16, y as u16, c);
            }

            if x == x2 as i32 && y == y2 as i32 {
                break;
            }
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    pub fn draw_connection(&mut self, state: &AppState, index: usize) {
        let conn = &state.connections[index];
        let from = state.nodes.iter().find(|n| n.id == conn.from_id);
        let to = state.nodes.iter().find(|n| n.id == conn.to_id);

        if let (Some(f), Some(t)) = (from, to) {
            let x1 = f.x + conn.from_offset.0;
            let y1 = f.y + conn.from_offset.1;
            let mut x2 = t.x + conn.to_offset.0;
            let mut y2 = t.y + conn.to_offset.1;

            // Offset the arrowhead so it sits just outside the node border
            if conn.has_arrow {
                if conn.to_offset.1 == 0 {
                    y2 = y2.saturating_sub(1);
                } else if conn.to_offset.1 == t.height - 1 {
                    y2 += 1;
                } else if conn.to_offset.0 == 0 {
                    x2 = x2.saturating_sub(1);
                } else if conn.to_offset.0 == t.width - 1 {
                    x2 += 1;
                }
            }

            let vertical_first = conn.from_offset.1 == 0 || conn.from_offset.1 == f.height - 1;
            let is_selected = state.selected_connection_index == Some(index);
            self.draw_route(x1, y1, x2, y2, conn.has_arrow, is_selected, vertical_first);
        }
    }

    pub fn draw_partial_connection(&mut self, from_node: &Node, offset: (u16, u16), target: (u16, u16)) {
        let x1 = from_node.x + offset.0;
        let y1 = from_node.y + offset.1;
        let x2 = target.0;
        let y2 = target.1;

        let vertical_first = offset.1 == 0 || offset.1 == from_node.height - 1;
        self.draw_route(x1, y1, x2, y2, true, true, vertical_first); // Active partial is highlighted
    }

    fn draw_route(&mut self, x1: u16, y1: u16, x2: u16, y2: u16, arrow: bool, highlighted: bool, vertical_first: bool) {
        let horiz = if highlighted { '=' } else { '-' };
        let vert = if highlighted { '#' } else { '|' };
        let join = if highlighted { '#' } else { '+' };
        let start = if highlighted { '@' } else { 'o' };

        let mid_y = (y1 + y2) / 2;
        let mid_x = (x1 + x2) / 2;

        if vertical_first {
            // Vertical -> Horizontal -> Vertical (Z-route)
            
            // First vertical segment
            let iy1 = y1.min(mid_y);
            let iy2 = y1.max(mid_y);
            for y in iy1..=iy2 { self.set_conn(x1, y, vert); }
            
            // Horizontal segment
            let ix1 = x1.min(x2);
            let ix2 = x1.max(x2);
            for x in ix1..=ix2 { self.set_conn(x, mid_y, horiz); }
            
            // Second vertical segment
            let iy3 = mid_y.min(y2);
            let iy4 = mid_y.max(y2);
            for y in iy3..=iy4 { self.set_conn(x2, y, vert); }
            
            // Corners
            if x1 != x2 {
                self.set_conn(x1, mid_y, join);
                self.set_conn(x2, mid_y, join);
            }
        } else {
            // Horizontal -> Vertical -> Horizontal (S-route)
            
            // First horizontal segment
            let ix1 = x1.min(mid_x);
            let ix2 = x1.max(mid_x);
            for x in ix1..=ix2 { self.set_conn(x, y1, horiz); }
            
            // Vertical segment
            let iy1 = y1.min(y2);
            let iy2 = y1.max(y2);
            for y in iy1..=iy2 { self.set_conn(mid_x, y, vert); }
            
            // Second horizontal segment
            let ix3 = mid_x.min(x2);
            let ix4 = mid_x.max(x2);
            for x in ix3..=ix4 { self.set_conn(x, y2, horiz); }
            
            // Corners
            if y1 != y2 {
                self.set_conn(mid_x, y1, join);
                self.set_conn(mid_x, y2, join);
            }
        }
        
        // Re-render start
        self.set_conn(x1, y1, start);
        
        if arrow {
            let arrow_char = if vertical_first {
                if y2 != mid_y {
                    if y1 < y2 { 'v' } else { '^' }
                } else {
                    if x1 < x2 { '>' } else { '<' }
                }
            } else {
                if x2 != mid_x {
                    if x1 < x2 { '>' } else { '<' }
                } else {
                    if y1 < y2 { 'v' } else { '^' }
                }
            };
            self.set_conn(x2, y2, arrow_char);
        } else {
            self.set_conn(x2, y2, start);
        }
    }

    // Special set that doesn't overwrite node boundaries or text if we want,
    fn set_conn(&mut self, x: u16, y: u16, c: char) {
        self.set(x, y, c);
    }

    pub fn to_string(&self) -> String {
        let mut out = String::new();
        for row in &self.grid {
            let line: String = row.iter().collect();
            out.push_str(&line);
            out.push('\n');
        }
        out
    }
}

pub fn render_to_canvas(state: &AppState, width: u16, height: u16) -> Canvas {
    let mut canvas = Canvas::new(width, height);
    
    // Create temp state with camera-adjusted positions
    let mut nodes = Vec::new();
    for n in &state.nodes {
        let mut node = n.clone();
        node.x = (n.x as i32 - state.camera_offset.0).max(0) as u16;
        node.y = (n.y as i32 - state.camera_offset.1).max(0) as u16;
        nodes.push(node);
    }

    let mut temp_state = AppState::new(state.title.clone());
    temp_state.nodes = nodes;
    temp_state.connections = state.connections.clone();
    temp_state.selected_connection_index = state.selected_connection_index;
    
    if let Some(crate::model::PartialConnection::Starting { from_id, from_offset, current_pos }) = &state.partial_connection {
        temp_state.partial_connection = Some(crate::model::PartialConnection::Starting {
            from_id: *from_id,
            from_offset: *from_offset,
            current_pos: (
                (current_pos.0 as i32 - state.camera_offset.0).max(0) as u16,
                (current_pos.1 as i32 - state.camera_offset.1).max(0) as u16,
            ),
        });
    }

    // Draw nodes
    for node in &temp_state.nodes {
        match node.shape {
            ShapeType::Box => canvas.draw_box(node),
            ShapeType::Diamond => canvas.draw_diamond(node),
            ShapeType::Text => canvas.draw_text_node(node),
        }
    }

    // Draw connections after nodes
    for i in 0..temp_state.connections.len() {
        canvas.draw_connection(&temp_state, i);
    }

    if let Some(crate::model::PartialConnection::Starting { from_id, from_offset, current_pos }) = &temp_state.partial_connection {
        if let Some(node) = temp_state.nodes.iter().find(|n| n.id == *from_id) {
            canvas.draw_partial_connection(node, *from_offset, *current_pos);
        }
    }

    canvas
}
