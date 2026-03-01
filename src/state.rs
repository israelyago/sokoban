use std::collections::HashSet;

use crate::{coord::Pos, level::Level};

#[derive(Debug, Clone)]
pub struct GameState {
    pub player: Pos,
    pub boxes: HashSet<Pos>,
    pub moves: u32,
    pub pushes: u32,
}

impl GameState {
    pub fn from_level(level: &Level) -> Self {
        Self {
            player: level.start_player,
            boxes: level.start_boxes.clone(),
            moves: 0,
            pushes: 0,
        }
    }

    pub fn is_won(&self, level: &Level) -> bool {
        self.boxes.is_subset(&level.goals)
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StepResult {
    pub moved: bool,
    pub pushed: bool,
    pub won: bool,
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{coord::Pos, level::Level};

    use super::GameState;

    fn sample_level(goals: &[Pos], boxes: &[Pos], player: Pos) -> Level {
        Level {
            width: 6,
            height: 6,
            walls: HashSet::new(),
            goals: goals.iter().copied().collect(),
            start_player: player,
            start_boxes: boxes.iter().copied().collect(),
            name: Some("state-test".to_string()),
        }
    }

    #[test]
    fn from_level_copies_start_positions_and_resets_counters() {
        let level = sample_level(&[Pos::new(1, 1)], &[Pos::new(2, 2)], Pos::new(0, 0));

        let state = GameState::from_level(&level);

        assert_eq!(state.player, Pos::new(0, 0));
        assert!(state.boxes.contains(&Pos::new(2, 2)));
        assert_eq!(state.moves, 0);
        assert_eq!(state.pushes, 0);
    }

    #[test]
    fn is_won_when_all_boxes_are_on_goals() {
        let level = sample_level(
            &[Pos::new(1, 1), Pos::new(2, 2)],
            &[Pos::new(1, 1)],
            Pos::new(0, 0),
        );
        let state = GameState::from_level(&level);

        assert!(state.is_won(&level));
    }

    #[test]
    fn is_not_won_when_any_box_is_off_goal() {
        let level = sample_level(&[Pos::new(1, 1)], &[Pos::new(2, 2)], Pos::new(0, 0));
        let state = GameState::from_level(&level);

        assert!(!state.is_won(&level));
    }
}
