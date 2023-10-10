use crate::app::App;
use crate::display::menuopts::{
    INPUT_OPT_AUTH_ANY, INPUT_OPT_AUTH_BASIC, INPUT_OPT_AUTH_BEARER, INPUT_OPT_BASIC,
    INPUT_OPT_HEADERS, INPUT_OPT_REC_DOWNLOAD,
};
use crate::display::DisplayOpts;
use crate::request::cmdtype::CmdType;
use crate::request::command::AppCmd;
use crate::request::curl::AuthKind;
use crate::screens::auth::AuthType;
use crate::screens::Screen;
use crate::{app::InputMode, display::inputopt::InputOpt};
use tui::prelude::Line;
use tui::style::Color;
use tui::widgets::Paragraph;
use tui::widgets::{Block, Borders};
use tui::{
    prelude::{Backend, Constraint, Direction, Layout},
    style::{Modifier, Style},
    text::{Span, Text},
    Frame,
};

// Takes the current option and returns a prompt for that screen
pub fn get_input_prompt(opt: InputOpt) -> Text<'static> {
    return match opt {
        InputOpt::URL(opt) => {
            let fmtstr = format!("Enter a URL for your {}\n and press Enter", opt);
            Text::from(Line::from(fmtstr))
        }
        InputOpt::Headers => Text::from(Line::from(INPUT_OPT_HEADERS)),
        InputOpt::RecursiveDownload => Text::from(INPUT_OPT_REC_DOWNLOAD),
        InputOpt::Auth(auth) => match auth {
            AuthType::Basic => Text::from(INPUT_OPT_AUTH_BASIC),
            AuthType::Bearer => Text::from(INPUT_OPT_AUTH_BEARER),
            _ => Text::from(INPUT_OPT_AUTH_ANY),
        },
        _ => Text::from(INPUT_OPT_BASIC),
    };
}

pub fn handle_default_input_screen<B: Backend>(
    app: &mut App,
    frame: &mut Frame<'_, B>,
    opt: InputOpt,
) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(frame.size());
    let (_msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press h"),
                Span::raw("to go back."),
                Span::styled("Press i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to start editing."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Editing => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to stop editing, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to submit."),
            ],
            Style::default(),
        ),
    };
    let mut prompt = get_input_prompt(opt.clone());
    prompt.patch_style(style);
    render_input_with_prompt(frame, prompt);

    let width = chunks[0].width.max(3) - 3; // keep 2 for borders and 1 for cursor
    let scroll = app.input.visual_scroll(width as usize);
    let input = Paragraph::new(app.input.value())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Editing => Style::default().fg(Color::LightBlue),
        })
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    frame.render_widget(input, chunks[1]);
    match app.input_mode {
        InputMode::Normal => {}
        InputMode::Editing => frame.set_cursor(
            chunks[1].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[1].y + 1,
        ),
    }
    // we have input (the user has typed something and pressed Enter while in insert mode)
    if !app.messages.is_empty() {
        app.input_mode = InputMode::Normal;
        // parse the input message with the opt to find out what to do with it
        parse_input(app.messages[0].clone(), opt, app);
        app.messages.remove(0);
    }
}

fn parse_input(message: String, opt: InputOpt, app: &mut App) {
    match opt {
        InputOpt::URL(opt) => {
            match opt {
                CmdType::Wget => {
                    app.add_display_option(DisplayOpts::URL(message));
                    app.goto_screen(Screen::Downloads);
                }
                CmdType::Curl => {
                    app.add_display_option(DisplayOpts::URL(message));
                    app.goto_screen(Screen::RequestMenu(String::new()));
                }
            };
        }
        InputOpt::ApiKey => {
            let _ = app.add_saved_key(message.clone());
            app.goto_screen(Screen::SavedKeys);
        }
        InputOpt::UnixSocket => {
            app.add_display_option(DisplayOpts::UnixSocket(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::Headers => {
            app.add_display_option(DisplayOpts::Headers(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        // Only downloads let you specify the output file prior to execution of the command
        InputOpt::Output => {
            app.add_display_option(DisplayOpts::Outfile(message.clone()));
            app.goto_screen(Screen::Downloads);
        }
        InputOpt::Cookie => {
            app.add_display_option(DisplayOpts::Cookie(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::Referrer => {
            app.add_display_option(DisplayOpts::Referrer(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::CaPath => {
            app.add_display_option(DisplayOpts::CaPath(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::UserAgent => {
            app.add_display_option(DisplayOpts::UserAgent(message.clone()));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::MaxRedirects => {
            let num = message.parse::<usize>().unwrap();
            app.add_display_option(DisplayOpts::MaxRedirects(num));
            app.goto_screen(Screen::RequestMenu(String::new()));
        }
        InputOpt::Execute => {
            // This means they have executed the HTTP Request, and want to write to a file
            match app.command.as_mut().unwrap() {
                AppCmd::CurlCmd(curl) => {
                    curl.set_outfile(&message);
                    curl.write_output().unwrap();
                }
                _ => {}
            }
        }
        InputOpt::RecursiveDownload => {
            let recursion_level = message.parse::<usize>().unwrap();
            app.add_display_option(DisplayOpts::RecDownload(recursion_level));
            app.goto_screen(Screen::Downloads);
        }
        InputOpt::Auth(auth) => {
            parse_auth(auth, app, &message);
        }
        _ => {}
    }
}

fn render_input_with_prompt<B: Backend>(frame: &mut Frame<'_, B>, prompt: Text) {
    // Render the input with the provided prompt
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
            ]
            .as_ref(),
        )
        .split(frame.size());

    let message = Paragraph::new(prompt);
    frame.render_widget(message, chunks[0]);
}

fn parse_auth(auth: AuthType, app: &mut App, message: &str) {
    if app.has_display_option(&DisplayOpts::Auth(String::new())) {
        app.remove_display_option(&DisplayOpts::Auth(String::new()));
    }
    match app.command.as_mut().unwrap() {
        AppCmd::CurlCmd(curl) => match auth {
            AuthType::Basic => {
                curl.set_auth(AuthKind::Basic(String::from(message)));
            }
            AuthType::Bearer => {
                curl.set_auth(AuthKind::Bearer(String::from(message)));
            }
            AuthType::Digest => {
                curl.set_auth(AuthKind::Digest(String::from(message)));
            }
            AuthType::AWSSignatureV4 => {
                curl.set_auth(AuthKind::AwsSigv4(String::from(message)));
            }
            AuthType::SPNEGO => {
                curl.set_auth(AuthKind::Spnego(String::from(message)));
            }
            AuthType::Kerberos => {
                curl.set_auth(AuthKind::Kerberos(String::from(message)));
            }
            AuthType::NTLM => {
                curl.set_auth(AuthKind::Ntlm(String::from(message)));
            }
            AuthType::NTLMWB => {
                curl.set_auth(AuthKind::NtlmWb(String::from(message)));
            }
        },
        _ => {}
    }
    app.add_display_option(DisplayOpts::Auth(String::from(message)));
    app.goto_screen(Screen::RequestMenu(String::new()));
}
