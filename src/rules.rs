use crate::{
    coord::Direction,
    level::Level,
    state::{GameState, StepResult},
};

pub fn try_step(state: &mut GameState, level: &Level, dir: Direction) -> StepResult {
    let target = state.player.move_dir(dir);

    if level.is_wall(target) {
        return StepResult::default();
    }

    if state.boxes.contains(&target) {
        let beyond = target.move_dir(dir);

        if level.is_wall(beyond) || state.boxes.contains(&beyond) {
            return StepResult::default();
        }

        state.boxes.remove(&target);
        state.boxes.insert(beyond);
        state.player = target;
        state.moves += 1;
        state.pushes += 1;

        return StepResult {
            moved: true,
            pushed: true,
            won: state.is_won(level),
        };
    }

    state.player = target;
    state.moves += 1;

    StepResult {
        moved: true,
        pushed: false,
        won: state.is_won(level),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{
        coord::{Direction, Pos},
        level::Level,
        state::GameState,
    };

    use super::try_step;

    fn level_with(walls: &[Pos], goals: &[Pos], player: Pos, boxes: &[Pos]) -> (Level, GameState) {
        let level = Level {
            width: 5,
            height: 5,
            walls: walls.iter().copied().collect::<HashSet<_>>(),
            goals: goals.iter().copied().collect::<HashSet<_>>(),
            start_player: player,
            start_boxes: boxes.iter().copied().collect::<HashSet<_>>(),
            name: Some("test".to_string()),
        };
        let state = GameState::from_level(&level);
        (level, state)
    }

    #[test]
    fn blocked_by_wall_is_noop() {
        let (level, mut state) = level_with(&[Pos::new(2, 1)], &[], Pos::new(1, 1), &[]);

        let result = try_step(&mut state, &level, Direction::Right);

        assert!(!result.moved);
        assert!(!result.pushed);
        assert!(!result.won);
        assert_eq!(state.player, Pos::new(1, 1));
        assert_eq!(state.moves, 0);
        assert_eq!(state.pushes, 0);
    }

    #[test]
    fn pushes_single_box_when_space_is_free() {
        let (level, mut state) =
            level_with(&[], &[Pos::new(3, 1)], Pos::new(1, 1), &[Pos::new(2, 1)]);

        let result = try_step(&mut state, &level, Direction::Right);

        assert!(result.moved);
        assert!(result.pushed);
        assert!(result.won);
        assert_eq!(state.player, Pos::new(2, 1));
        assert!(state.boxes.contains(&Pos::new(3, 1)));
        assert_eq!(state.moves, 1);
        assert_eq!(state.pushes, 1);
    }

    #[test]
    fn push_blocked_by_wall_is_noop() {
        let (level, mut state) =
            level_with(&[Pos::new(3, 1)], &[], Pos::new(1, 1), &[Pos::new(2, 1)]);

        let result = try_step(&mut state, &level, Direction::Right);

        assert!(!result.moved);
        assert!(!result.pushed);
        assert!(!result.won);
        assert_eq!(state.player, Pos::new(1, 1));
        assert!(state.boxes.contains(&Pos::new(2, 1)));
        assert_eq!(state.moves, 0);
        assert_eq!(state.pushes, 0);
    }

    #[test]
    fn push_blocked_by_box_is_noop() {
        let (level, mut state) =
            level_with(&[], &[], Pos::new(1, 1), &[Pos::new(2, 1), Pos::new(3, 1)]);

        let result = try_step(&mut state, &level, Direction::Right);

        assert!(!result.moved);
        assert!(!result.pushed);
        assert!(!result.won);
        assert_eq!(state.player, Pos::new(1, 1));
        assert!(state.boxes.contains(&Pos::new(2, 1)));
        assert!(state.boxes.contains(&Pos::new(3, 1)));
        assert_eq!(state.moves, 0);
        assert_eq!(state.pushes, 0);
    }

    #[test]
    fn non_goal_box_configuration_is_not_won() {
        let (level, mut state) =
            level_with(&[], &[Pos::new(4, 4)], Pos::new(1, 1), &[Pos::new(2, 1)]);

        let result = try_step(&mut state, &level, Direction::Right);

        assert!(result.moved);
        assert!(result.pushed);
        assert!(!result.won);
    }
}
