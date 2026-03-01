use std::{
    collections::VecDeque,
    io::{self, Write},
};

use anyhow::Context;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};

use crate::{
    coord::Direction,
    level::{Level, LevelPack},
    render::render,
    rules::try_step,
    state::GameState,
};

struct TerminalGuard;

impl TerminalGuard {
    fn enter() -> anyhow::Result<Self> {
        terminal::enable_raw_mode().context("failed to enable raw mode")?;
        execute!(io::stdout(), EnterAlternateScreen, Hide)
            .context("failed to setup terminal screen")?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
        let _ = terminal::disable_raw_mode();
    }
}

fn draw(
    level: &Level,
    state: &GameState,
    level_index: usize,
    total_levels: usize,
) -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    let frame = render(level, state).replace('\n', "\r\n");
    execute!(stdout, MoveTo(0, 0), Clear(ClearType::All))?;
    write!(stdout, "Level {}/{}\r\n", level_index + 1, total_levels)?;
    write!(stdout, "{}\r\n", frame)?;
    if state.is_won(level) {
        write!(
            stdout,
            "\r\nSolved! Press N for next level, R to restart, or Q to quit.\r\n"
        )?;
    }
    stdout.flush()?;
    Ok(())
}

pub fn run(pack: &LevelPack, start_level: usize) -> anyhow::Result<()> {
    const UNDO_LIMIT: usize = 10_000;

    let levels = pack.parse_levels()?;
    if levels.is_empty() {
        anyhow::bail!("level pack contains no levels");
    }

    let mut level_index = start_level.clamp(1, levels.len()) - 1;
    let mut state = GameState::from_level(&levels[level_index]);
    let mut undo_stack: VecDeque<GameState> = VecDeque::new();
    let _guard = TerminalGuard::enter()?;

    draw(&levels[level_index], &state, level_index, levels.len())?;

    loop {
        if !event::poll(std::time::Duration::from_millis(250))? {
            continue;
        }

        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => {
                let mut redraw = false;

                match key.code {
                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                    KeyCode::Char('r') | KeyCode::Char('R') => {
                        state = GameState::from_level(&levels[level_index]);
                        undo_stack.clear();
                        redraw = true;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') => {
                        level_index = (level_index + 1) % levels.len();
                        state = GameState::from_level(&levels[level_index]);
                        undo_stack.clear();
                        redraw = true;
                    }
                    KeyCode::Char('p') | KeyCode::Char('P') => {
                        level_index = if level_index == 0 {
                            levels.len() - 1
                        } else {
                            level_index - 1
                        };
                        state = GameState::from_level(&levels[level_index]);
                        undo_stack.clear();
                        redraw = true;
                    }
                    KeyCode::Char('u') | KeyCode::Char('U') => {
                        if let Some(previous) = undo_stack.pop_back() {
                            state = previous;
                            redraw = true;
                        }
                    }
                    KeyCode::Up | KeyCode::Char('w') | KeyCode::Char('W') => {
                        let before = state.clone();
                        let result = try_step(&mut state, &levels[level_index], Direction::Up);
                        if result.moved {
                            if undo_stack.len() == UNDO_LIMIT {
                                undo_stack.pop_front();
                            }
                            undo_stack.push_back(before);
                        }
                        redraw = result.moved;
                    }
                    KeyCode::Down | KeyCode::Char('s') | KeyCode::Char('S') => {
                        let before = state.clone();
                        let result = try_step(&mut state, &levels[level_index], Direction::Down);
                        if result.moved {
                            if undo_stack.len() == UNDO_LIMIT {
                                undo_stack.pop_front();
                            }
                            undo_stack.push_back(before);
                        }
                        redraw = result.moved;
                    }
                    KeyCode::Left | KeyCode::Char('a') | KeyCode::Char('A') => {
                        let before = state.clone();
                        let result = try_step(&mut state, &levels[level_index], Direction::Left);
                        if result.moved {
                            if undo_stack.len() == UNDO_LIMIT {
                                undo_stack.pop_front();
                            }
                            undo_stack.push_back(before);
                        }
                        redraw = result.moved;
                    }
                    KeyCode::Right | KeyCode::Char('d') | KeyCode::Char('D') => {
                        let before = state.clone();
                        let result = try_step(&mut state, &levels[level_index], Direction::Right);
                        if result.moved {
                            if undo_stack.len() == UNDO_LIMIT {
                                undo_stack.pop_front();
                            }
                            undo_stack.push_back(before);
                        }
                        redraw = result.moved;
                    }
                    _ => {}
                }

                if redraw {
                    draw(&levels[level_index], &state, level_index, levels.len())?;
                }
            }
            Event::Resize(_, _) => {
                draw(&levels[level_index], &state, level_index, levels.len())?;
            }
            _ => {}
        }
    }

    Ok(())
}
