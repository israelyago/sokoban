use anyhow::{Result, bail};

use crate::editor_model::normalize_map_name;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum EditorTile {
    Box,
    BoxOnGoal,
    Floor,
    Goal,
    Wall,
    Player,
    PlayerOnGoal,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Cell {
    Floor,
    Goal,
    Wall,
    Box,
    BoxOnGoal,
    Player,
    PlayerOnGoal,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EditorMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Cell>,
    pub map_name: String,
}

pub type EditorUndoSnapshot = EditorMap;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum SimpleWarning {
    MissingBoxes,
    BoxesMoreThanGoals { boxes: usize, goals: usize },
    AlreadySolved,
    MissingPlayer,
}

impl EditorMap {
    pub fn from_raw_lines(name: Option<&str>, raw_lines: &[String]) -> Result<Self> {
        if raw_lines.is_empty() {
            bail!("map has no rows");
        }

        let height = raw_lines.len();
        let width = raw_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        if width == 0 {
            bail!("map has no columns");
        }

        let mut cells = Vec::with_capacity(width * height);
        for line in raw_lines {
            let mut chars = line.chars().collect::<Vec<_>>();
            while chars.len() < width {
                chars.push(' ');
            }
            for ch in chars {
                cells.push(char_to_cell(ch)?);
            }
        }

        let mut map = Self {
            width,
            height,
            cells,
            map_name: normalize_or_default_name(name)?,
        };
        map.ensure_single_player();
        Ok(map)
    }

    pub fn to_raw_lines(&self) -> Vec<String> {
        let mut out = Vec::with_capacity(self.height);
        for y in 0..self.height {
            let mut line = String::with_capacity(self.width);
            for x in 0..self.width {
                line.push(cell_to_char(self.get(x, y).expect("in bounds")));
            }
            out.push(line);
        }
        out
    }

    pub fn paint(&mut self, x: usize, y: usize, tile: EditorTile) -> Result<()> {
        if !self.in_bounds(x, y) {
            bail!("position out of bounds");
        }

        let target = tile_to_cell(tile);
        if matches!(target, Cell::Player | Cell::PlayerOnGoal) {
            self.remove_all_players();
        }
        self.set(x, y, target)?;
        Ok(())
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) -> Result<()> {
        validate_canvas_size(new_width, new_height)?;

        let old_width = self.width;
        let old_height = self.height;
        let old_cells = self.cells.clone();
        let growing = new_width > old_width || new_height > old_height;

        self.width = new_width;
        self.height = new_height;
        self.cells = vec![Cell::Floor; new_width * new_height];

        let copy_w = old_width.min(new_width);
        let copy_h = old_height.min(new_height);
        for y in 0..copy_h {
            for x in 0..copy_w {
                let old = old_cells[y * old_width + x];
                self.cells[y * new_width + x] = old;
            }
        }

        if growing {
            self.enforce_new_border_walls(old_width, old_height);
            // New interior cells already default to floor.
        }

        self.ensure_single_player();
        Ok(())
    }

    pub fn rename_map(&mut self, raw_name: &str) -> Result<()> {
        self.map_name = normalize_map_name(raw_name)?;
        Ok(())
    }

    pub fn simple_warning(&self) -> Option<SimpleWarning> {
        let mut boxes = 0usize;
        let mut boxes_off_goal = 0usize;
        let mut goals = 0usize;
        let mut players = 0usize;

        for cell in &self.cells {
            match cell {
                Cell::Box => {
                    boxes += 1;
                    boxes_off_goal += 1;
                }
                Cell::BoxOnGoal => {
                    boxes += 1;
                    goals += 1;
                }
                Cell::Goal => goals += 1,
                Cell::Player => players += 1,
                Cell::PlayerOnGoal => {
                    players += 1;
                    goals += 1;
                }
                Cell::Floor | Cell::Wall => {}
            }
        }

        if boxes == 0 {
            return Some(SimpleWarning::MissingBoxes);
        }
        if boxes > goals {
            return Some(SimpleWarning::BoxesMoreThanGoals { boxes, goals });
        }
        if boxes > 0 && boxes_off_goal == 0 {
            return Some(SimpleWarning::AlreadySolved);
        }
        if players == 0 {
            return Some(SimpleWarning::MissingPlayer);
        }
        None
    }

    pub fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<Cell> {
        if !self.in_bounds(x, y) {
            return None;
        }
        Some(self.cells[y * self.width + x])
    }

    fn set(&mut self, x: usize, y: usize, cell: Cell) -> Result<()> {
        if !self.in_bounds(x, y) {
            bail!("position out of bounds");
        }
        let idx = y * self.width + x;
        self.cells[idx] = cell;
        Ok(())
    }

    fn remove_all_players(&mut self) {
        for cell in &mut self.cells {
            *cell = match *cell {
                Cell::Player => Cell::Floor,
                Cell::PlayerOnGoal => Cell::Goal,
                other => other,
            };
        }
    }

    fn ensure_single_player(&mut self) {
        let mut seen = false;
        for cell in &mut self.cells {
            match *cell {
                Cell::Player if !seen => {
                    seen = true;
                }
                Cell::PlayerOnGoal if !seen => {
                    seen = true;
                }
                Cell::Player => {
                    *cell = Cell::Floor;
                }
                Cell::PlayerOnGoal => {
                    *cell = Cell::Goal;
                }
                _ => {}
            }
        }
    }

    fn enforce_new_border_walls(&mut self, old_width: usize, old_height: usize) {
        if self.width == 0 || self.height == 0 {
            return;
        }

        for y in 0..self.height {
            for x in 0..self.width {
                let is_new_cell = x >= old_width || y >= old_height;
                if !is_new_cell {
                    continue;
                }

                let is_border =
                    x == 0 || y == 0 || x == self.width - 1 || y == self.height - 1;
                if is_border {
                    self.cells[y * self.width + x] = Cell::Wall;
                }
            }
        }
    }
}

pub fn validate_canvas_size(width: usize, height: usize) -> Result<()> {
    if width == 0 || height == 0 {
        bail!("size must be positive");
    }
    if width * height < 3 {
        bail!("canvas area must be at least 3");
    }
    Ok(())
}

fn normalize_or_default_name(name: Option<&str>) -> Result<String> {
    match name {
        Some(name) if !name.trim().is_empty() => Ok(normalize_map_name(name)?),
        _ => Ok("Map".to_string()),
    }
}

fn tile_to_cell(tile: EditorTile) -> Cell {
    match tile {
        EditorTile::Box => Cell::Box,
        EditorTile::BoxOnGoal => Cell::BoxOnGoal,
        EditorTile::Floor => Cell::Floor,
        EditorTile::Goal => Cell::Goal,
        EditorTile::Wall => Cell::Wall,
        EditorTile::Player => Cell::Player,
        EditorTile::PlayerOnGoal => Cell::PlayerOnGoal,
    }
}

fn char_to_cell(ch: char) -> Result<Cell> {
    Ok(match ch {
        '#' => Cell::Wall,
        ' ' | '-' => Cell::Floor,
        '.' => Cell::Goal,
        '@' => Cell::Player,
        '+' => Cell::PlayerOnGoal,
        '$' => Cell::Box,
        '*' => Cell::BoxOnGoal,
        _ => bail!("invalid map char '{ch}'"),
    })
}

fn cell_to_char(cell: Cell) -> char {
    match cell {
        Cell::Wall => '#',
        Cell::Floor => '-',
        Cell::Goal => '.',
        Cell::Player => '@',
        Cell::PlayerOnGoal => '+',
        Cell::Box => '$',
        Cell::BoxOnGoal => '*',
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, EditorMap, EditorTile, SimpleWarning};

    #[test]
    fn raw_round_trip_preserves_map_shape_and_content() {
        let raw = vec![
            "#####".to_string(),
            "#@$.#".to_string(),
            "#-*.#".to_string(),
            "#####".to_string(),
        ];
        let map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        let serialized = map.to_raw_lines();
        assert_eq!(serialized, raw);
    }

    #[test]
    fn resize_up_enforces_wall_border_and_new_interior_floor() {
        let raw = vec!["###".to_string(), "#@#".to_string(), "###".to_string()];
        let mut map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        map.resize(5, 4).expect("resize should work");

        assert_eq!(map.width, 5);
        assert_eq!(map.height, 4);
        for x in 0..map.width {
            assert_eq!(map.get(x, 0), Some(Cell::Wall));
            assert_eq!(map.get(x, map.height - 1), Some(Cell::Wall));
        }
        for y in 0..map.height {
            assert_eq!(map.get(0, y), Some(Cell::Wall));
            assert_eq!(map.get(map.width - 1, y), Some(Cell::Wall));
        }

        // This coordinate did not exist before and is interior after growth.
        assert_eq!(map.get(3, 2), Some(Cell::Floor));
    }

    #[test]
    fn resize_up_keeps_existing_border_cells_unchanged() {
        let raw = vec!["---@".to_string(), "----".to_string(), "----".to_string()];
        let mut map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        map.resize(6, 5).expect("resize should work");

        // Existing cell that was on old border remains as it was.
        assert_eq!(map.get(0, 0), Some(Cell::Floor));
        assert_eq!(map.get(3, 0), Some(Cell::Player));

        // Newly added border cells become walls.
        assert_eq!(map.get(5, 0), Some(Cell::Wall));
        assert_eq!(map.get(5, 4), Some(Cell::Wall));
        assert_eq!(map.get(0, 4), Some(Cell::Wall));

        // Newly added interior stays floor.
        assert_eq!(map.get(4, 2), Some(Cell::Floor));
    }

    #[test]
    fn resize_down_truncates() {
        let raw = vec![
            "#####".to_string(),
            "#@$.#".to_string(),
            "#..*#".to_string(),
            "#####".to_string(),
        ];
        let mut map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        map.resize(3, 2).expect("resize should work");
        assert_eq!(map.width, 3);
        assert_eq!(map.height, 2);
        assert_eq!(
            map.to_raw_lines(),
            vec!["###".to_string(), "#@$".to_string()]
        );
    }

    #[test]
    fn resize_rejects_area_below_three() {
        let raw = vec!["###".to_string(), "#@#".to_string(), "###".to_string()];
        let mut map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        let err = map.resize(1, 2).expect_err("should reject small area");
        assert!(err.to_string().contains("at least 3"));
    }

    #[test]
    fn painting_player_enforces_uniqueness() {
        let raw = vec![
            "#####".to_string(),
            "#@--#".to_string(),
            "#---#".to_string(),
            "#####".to_string(),
        ];
        let mut map = EditorMap::from_raw_lines(Some("A"), &raw).expect("parse should work");
        map.paint(2, 2, EditorTile::Player)
            .expect("paint should work");

        let mut players = 0usize;
        for cell in &map.cells {
            if matches!(*cell, Cell::Player | Cell::PlayerOnGoal) {
                players += 1;
            }
        }
        assert_eq!(players, 1);
        assert_eq!(map.get(2, 2), Some(Cell::Player));
    }

    #[test]
    fn simple_warning_detects_basic_invalid_states() {
        let no_box = EditorMap::from_raw_lines(
            Some("A"),
            &[
                "#####".to_string(),
                "#@..#".to_string(),
                "#####".to_string(),
            ],
        )
        .expect("parse should work");
        assert_eq!(no_box.simple_warning(), Some(SimpleWarning::MissingBoxes));
    }

    #[test]
    fn simple_warning_boxes_more_than_goals() {
        let map = EditorMap::from_raw_lines(
            Some("A"),
            &[
                "#####".to_string(),
                "#@$$#".to_string(),
                "#--.#".to_string(),
                "#####".to_string(),
            ],
        )
        .expect("parse should work");
        assert_eq!(
            map.simple_warning(),
            Some(SimpleWarning::BoxesMoreThanGoals { boxes: 2, goals: 1 })
        );
    }

    #[test]
    fn simple_warning_missing_player() {
        let map = EditorMap::from_raw_lines(
            Some("A"),
            &[
                "#####".to_string(),
                "#$..#".to_string(),
                "#####".to_string(),
            ],
        )
        .expect("parse should work");
        assert_eq!(map.simple_warning(), Some(SimpleWarning::MissingPlayer));
    }

    #[test]
    fn simple_warning_already_solved_when_all_boxes_are_on_goals() {
        let map = EditorMap::from_raw_lines(
            Some("A"),
            &[
                "#####".to_string(),
                "#@*.#".to_string(),
                "#####".to_string(),
            ],
        )
        .expect("parse should work");
        assert_eq!(map.simple_warning(), Some(SimpleWarning::AlreadySolved));
    }
}
