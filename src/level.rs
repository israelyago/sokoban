use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

use crate::coord::Pos;

#[derive(Debug, Clone)]
pub struct Level {
    pub width: i32,
    pub height: i32,
    pub walls: std::collections::HashSet<Pos>,
    pub goals: std::collections::HashSet<Pos>,
    pub start_player: Pos,
    pub start_boxes: std::collections::HashSet<Pos>,
    pub name: Option<String>,
}

impl Level {
    pub fn is_wall(&self, pos: Pos) -> bool {
        self.walls.contains(&pos)
    }

    pub fn is_goal(&self, pos: Pos) -> bool {
        self.goals.contains(&pos)
    }
}

#[derive(Debug, Clone)]
pub struct LevelPack {
    pub source: PathBuf,
    pub levels: Vec<RawLevel>,
}

#[derive(Debug, Clone)]
pub struct RawLevel {
    pub name: Option<String>,
    pub lines: Vec<String>,
}

impl LevelPack {
    pub fn load(path: impl AsRef<Path>) -> anyhow::Result<Self> {
        let path = path.as_ref();
        let text = std::fs::read_to_string(path)?;

        let mut levels = Vec::new();
        let mut pending_name: Option<String> = None;
        let mut current_lines: Vec<String> = Vec::new();

        let flush = |levels: &mut Vec<RawLevel>,
                     pending_name: &mut Option<String>,
                     current_lines: &mut Vec<String>| {
            if current_lines.is_empty() {
                *pending_name = None;
                return;
            }
            levels.push(RawLevel {
                name: pending_name.take(),
                lines: std::mem::take(current_lines),
            });
        };

        for raw_line in text.lines() {
            let line = raw_line.trim_end_matches(['\r', '\n']);
            let trimmed = line.trim();

            if trimmed.is_empty() {
                flush(&mut levels, &mut pending_name, &mut current_lines);
                continue;
            }

            if let Some(rest) = line.strip_prefix(';') {
                let rest = rest.trim();
                if let Some(name) = rest.strip_prefix("name:") {
                    pending_name = Some(name.trim().to_string());
                }
                continue;
            }

            if let Some(title) = parse_title_line(trimmed) {
                if current_lines.is_empty() {
                    pending_name = Some(title);
                } else {
                    pending_name = Some(title);
                    flush(&mut levels, &mut pending_name, &mut current_lines);
                }
                continue;
            }

            if is_grid_line(line) {
                current_lines.push(line.to_string());
                continue;
            }

            // Ignore non-grid metadata/prose lines.
            flush(&mut levels, &mut pending_name, &mut current_lines);
        }

        flush(&mut levels, &mut pending_name, &mut current_lines);

        Ok(Self {
            source: path.to_path_buf(),
            levels,
        })
    }

    pub fn parse_levels(&self) -> anyhow::Result<Vec<Level>> {
        self.levels
            .iter()
            .enumerate()
            .map(|(idx, raw)| {
                raw.to_level()
                    .with_context(|| format!("failed to parse level {}", idx + 1))
            })
            .collect()
    }
}

impl RawLevel {
    pub fn to_level(&self) -> anyhow::Result<Level> {
        if self.lines.is_empty() {
            bail!("level has no grid lines");
        }

        let height = self.lines.len() as i32;
        let width = self
            .lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0) as i32;

        let mut walls = std::collections::HashSet::new();
        let mut goals = std::collections::HashSet::new();
        let mut start_boxes = std::collections::HashSet::new();
        let mut start_player: Option<Pos> = None;

        for (y, line) in self.lines.iter().enumerate() {
            let mut chars = line.chars().collect::<Vec<_>>();
            while chars.len() < width as usize {
                chars.push(' ');
            }

            for (x, ch) in chars.into_iter().enumerate() {
                let pos = Pos::new(x as i32, y as i32);
                match ch {
                    '#' => {
                        walls.insert(pos);
                    }
                    ' ' | '-' => {}
                    '.' => {
                        goals.insert(pos);
                    }
                    '@' => {
                        if start_player.replace(pos).is_some() {
                            bail!(
                                "level has more than one player (duplicate at line {}, column {})",
                                y + 1,
                                x + 1
                            );
                        }
                    }
                    '+' => {
                        if start_player.replace(pos).is_some() {
                            bail!(
                                "level has more than one player (duplicate at line {}, column {})",
                                y + 1,
                                x + 1
                            );
                        }
                        goals.insert(pos);
                    }
                    '$' => {
                        start_boxes.insert(pos);
                    }
                    '*' => {
                        start_boxes.insert(pos);
                        goals.insert(pos);
                    }
                    _ => {
                        bail!(
                            "invalid grid character '{}' at line {}, column {}",
                            ch,
                            y + 1,
                            x + 1
                        );
                    }
                }
            }
        }

        let start_player = start_player.context("missing player (@ or +)")?;
        if start_boxes.is_empty() {
            bail!("level must include at least one box ($ or *)");
        }
        if goals.len() < start_boxes.len() {
            bail!(
                "number of goals ({}) must be >= number of boxes ({})",
                goals.len(),
                start_boxes.len()
            );
        }

        Ok(Level {
            width,
            height,
            walls,
            goals,
            start_player,
            start_boxes,
            name: self.name.clone(),
        })
    }
}

fn is_grid_line(line: &str) -> bool {
    line.chars()
        .all(|ch| matches!(ch, '#' | ' ' | '-' | '.' | '@' | '+' | '$' | '*'))
}

fn parse_title_line(line: &str) -> Option<String> {
    line.strip_prefix("Title:")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LevelPack;

    fn write_temp(content: &str, suffix: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "sokoban_level_test_{}_{}_{}.txt",
            std::process::id(),
            suffix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        ));
        std::fs::write(&path, content).expect("failed to write temp level file");
        path
    }

    #[test]
    fn loads_drfogh_style_levels_with_title_after_grid() {
        let text = "\
Collection: Numbers

-####
-#-.#
-#-$#
-#-@#
-####
Title: One
Date: 2016-06-21

-####
-#-.#
-#-$#
-#-@#
-####
Title: Two
";
        let path = write_temp(text, "drfogh");
        let pack = LevelPack::load(&path).expect("load should succeed");
        std::fs::remove_file(&path).expect("cleanup should succeed");

        assert_eq!(pack.levels.len(), 2);
        assert_eq!(pack.levels[0].name.as_deref(), Some("One"));
        assert_eq!(pack.levels[1].name.as_deref(), Some("Two"));
    }

    #[test]
    fn parses_dash_as_floor_in_runtime_level() {
        let text = "\
-####
-#-.#
-#-$#
-#-@#
-####
Title: Dash Floor
";
        let path = write_temp(text, "dash");
        let pack = LevelPack::load(&path).expect("load should succeed");
        std::fs::remove_file(&path).expect("cleanup should succeed");
        let levels = pack.parse_levels().expect("parse levels should succeed");
        let level = &levels[0];

        assert_eq!(level.name.as_deref(), Some("Dash Floor"));
        assert!(level.goals.contains(&crate::coord::Pos::new(3, 1)));
        assert!(level.start_boxes.contains(&crate::coord::Pos::new(3, 2)));
        assert_eq!(level.start_player, crate::coord::Pos::new(3, 3));
    }

    #[test]
    fn parse_levels_rejects_missing_player() {
        let text = "\
####
# .#
# $#
####
Title: Missing Player
";
        let path = write_temp(text, "missing_player");
        let pack = LevelPack::load(&path).expect("load should succeed");
        std::fs::remove_file(&path).expect("cleanup should succeed");

        let err = pack.parse_levels().expect_err("parse should fail");
        assert!(format!("{err:#}").contains("missing player"));
    }

    #[test]
    fn parse_levels_rejects_duplicate_player() {
        let text = "\
#####
#@+ #
#$ .#
#####
Title: Duplicate Player
";
        let path = write_temp(text, "duplicate_player");
        let pack = LevelPack::load(&path).expect("load should succeed");
        std::fs::remove_file(&path).expect("cleanup should succeed");

        let err = pack.parse_levels().expect_err("parse should fail");
        assert!(format!("{err:#}").contains("more than one player"));
    }

    #[test]
    fn parse_levels_rejects_more_boxes_than_goals() {
        let text = "\
######
#@$$ #
# .  #
######
Title: Too Many Boxes
";
        let path = write_temp(text, "boxes_vs_goals");
        let pack = LevelPack::load(&path).expect("load should succeed");
        std::fs::remove_file(&path).expect("cleanup should succeed");

        let err = pack.parse_levels().expect_err("parse should fail");
        assert!(format!("{err:#}").contains("number of goals"));
    }
}
