use tui::widgets::Block;
use std::io;
use std::time::{Duration, Instant};
use clipboard::{ClipboardContext, ClipboardProvider};
use crossterm::{event, execute};
use crossterm::event::{DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use log::trace;
use tui::{Frame, Terminal};
use tui::backend::CrosstermBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{Borders, List, ListItem, Paragraph};
use crate::app::App;

pub fn init_terminal_app(app: App) -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let res = run_terminal_app(&mut terminal, app);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        trace!("{:?}", err);
    }

    Ok(())
}

pub fn run_terminal_app<B: tui::backend::Backend>(terminal: &mut Terminal<B>, mut app: App) -> io::Result<()> {
    let mut last_key_press = Instant::now();
    let cooldown_duration = Duration::from_millis(200);

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            let now = Instant::now();
            if now.duration_since(last_key_press) >= cooldown_duration {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Down => app.next(),
                    KeyCode::Up => app.prev(),
                    KeyCode::Enter => app.toggle_selected(),
                    KeyCode::Char('c') => {
                        if let Some(value) = app.copy_selected_value() {
                            let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
                            ctx.set_contents(value).unwrap();
                        }
                    },
                    KeyCode::Char('f') => app.first(),
                    KeyCode::Char('l') => app.last(),
                    _ => {}
                }
                last_key_press = now;
            }
        }
    }
}

pub fn ui<B: tui::backend::Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let items: Vec<ListItem> = app.view
        .iter_mut()
        .filter(|node| node.visible == true)
        .map(|node| {
            let content = node.get_view_content();
            ListItem::new(vec![Spans::from(vec![Span::styled(content, Style::default())])])
        })
        .collect();

    let items = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("ASN.1 Structure"))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD).fg(Color::Yellow))
        .highlight_symbol("> ");

    f.render_stateful_widget(items, chunks[0], &mut app.state);

    let instructions = Paragraph::new("↑↓: Navigate | f: to first | l: to last | Enter: Expand/Collapse | c: Copy Value | q: Quit")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, chunks[0]);
}