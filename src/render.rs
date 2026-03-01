use crate::{coord::Pos, level::Level, state::GameState};

pub fn render(level: &Level, state: &GameState) -> String {
    let title = level.name.as_deref().unwrap_or("(unnamed)");
    let mut out = String::new();
    out.push_str(&format!(
        "Level: {} | Moves: {} | Pushes: {}\n",
        title, state.moves, state.pushes
    ));

    for y in 0..level.height {
        for x in 0..level.width {
            let pos = Pos::new(x, y);
            let ch = if level.is_wall(pos) {
                '#'
            } else if state.player == pos {
                if level.is_goal(pos) { '+' } else { '@' }
            } else if state.boxes.contains(&pos) {
                if level.is_goal(pos) { '*' } else { '$' }
            } else if level.is_goal(pos) {
                '.'
            } else {
                ' '
            };
            out.push(ch);
        }
        out.push('\n');
    }

    out.push_str("WASD/arrows move | R restart | U undo | N/P level | Q quit");
    out
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::{coord::Pos, level::Level, state::GameState};

    use super::render;

    fn level_with(
        walls: &[Pos],
        goals: &[Pos],
        player: Pos,
        boxes: &[Pos],
        name: Option<&str>,
    ) -> Level {
        Level {
            width: 4,
            height: 3,
            walls: walls.iter().copied().collect::<HashSet<_>>(),
            goals: goals.iter().copied().collect::<HashSet<_>>(),
            start_player: player,
            start_boxes: boxes.iter().copied().collect::<HashSet<_>>(),
            name: name.map(str::to_string),
        }
    }

    #[test]
    fn renders_board_with_header_and_footer() {
        let level = level_with(
            &[Pos::new(0, 0), Pos::new(3, 2)],
            &[Pos::new(2, 0), Pos::new(2, 1)],
            Pos::new(1, 1),
            &[Pos::new(2, 1), Pos::new(0, 2)],
            Some("Demo"),
        );
        let mut state = GameState::from_level(&level);
        state.moves = 7;
        state.pushes = 3;

        let rendered = render(&level, &state);

        let expected = "\
Level: Demo | Moves: 7 | Pushes: 3
# . 
 @* 
$  #
WASD/arrows move | R restart | U undo | N/P level | Q quit";
        assert_eq!(rendered, expected);
    }

    #[test]
    fn renders_player_and_box_on_goals_with_plus_and_star() {
        let level = level_with(
            &[],
            &[Pos::new(1, 1), Pos::new(2, 1)],
            Pos::new(1, 1),
            &[Pos::new(2, 1)],
            None,
        );
        let state = GameState {
            player: level.start_player,
            boxes: level.start_boxes.clone(),
            moves: 0,
            pushes: 0,
        };

        let rendered = render(&level, &state);

        let expected = "\
Level: (unnamed) | Moves: 0 | Pushes: 0
    
 +* 
    
WASD/arrows move | R restart | U undo | N/P level | Q quit";
        assert_eq!(rendered, expected);
    }
}
