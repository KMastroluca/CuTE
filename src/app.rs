use std::{error, mem};

use crate::display::DisplayOpts;
use crate::request::command::Command;
use crate::screens::screen::Screen;
use crate::{database::db::DB, request::response::Response};
use tui::widgets::{ListItem, ListState};
use tui_input::Input;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    Editing,
}

/// Application.
#[derive(Debug)]
pub struct App<'a> {
    /// Is the application running?
    pub running: bool,
    pub cursor: usize,
    pub current_screen: Screen,
    pub screen_stack: Vec<Screen>,
    pub command: Option<Command<'a>>,
    pub selected: Option<usize>,
    pub opts: Vec<DisplayOpts>,
    pub input: Input,
    pub messages: Vec<String>,
    pub input_mode: InputMode,
    pub items: Vec<ListItem<'a>>,
    pub state: Option<ListState>,
    pub response: Option<String>,
    pub db: Option<Box<DB>>,
}

impl<'a> Default for App<'a> {
    fn default() -> Self {
        Self {
            running: true,
            cursor: 0,
            screen_stack: vec![Screen::Home],
            selected: None,
            command: None,
            input_mode: InputMode::Normal,
            messages: Vec::new(),
            opts: Vec::new(),
            items: Screen::Home.get_opts(),
            input: Input::default(),
            state: None,
            current_screen: Screen::Home,
            response: None,
            db: None,
        }
    }
}

impl<'a> App<'a> {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_command(&mut self, command: Command<'a>) {
        self.command = Some(command);
    }

    pub fn set_url(&mut self, url: String) {
        if let Some(ref mut cmd) = self.command {
            cmd.set_url(url);
        }
    }

    pub fn has_auth(&self) -> bool {
        match self.command.as_ref().unwrap() {
            Command::Curl(curl) => curl.has_auth(),
            _ => false,
        };
        false
    }

    pub fn tick(&self) {}

    pub fn goto_screen(&mut self, screen: Screen) {
        self.screen_stack.push(screen.clone());
        self.current_screen = screen.clone();

        self.cursor = 0;
        match screen {
            Screen::SavedKeys => {
                self.items = self
                    .get_saved_keys()
                    .unwrap_or(vec![])
                    .iter()
                    .map(|key| ListItem::new(key.clone()))
                    .collect();
            }
            Screen::SavedCommands => {
                self.items = self
                    .get_saved_commands()
                    .unwrap_or(vec![])
                    .iter()
                    .map(|cmd| ListItem::new(cmd.clone()))
                    .collect();
            }
            _ => {
                self.items = screen.get_opts();
            }
        }
        self.selected = None;
    }

    pub fn go_back_screen(&mut self) {
        match self.screen_stack.pop() {
            // we are not returning to an input menu, so we pop the last element that wasn't an input menu
            Some(Screen::InputMenu(_)) => {
                // we can unwrap, because if we have hit an input menu, it's guaranteed
                self.current_screen = self.screen_stack.last().unwrap().clone();
            }
            Some(_) => match self.screen_stack.last() {
                Some(screen) => {
                    self.cursor = 0;
                    self.selected = None;
                    self.items = screen.get_opts();
                    self.current_screen = screen.clone();
                }
                _ => {}
            },
            None => {}
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn move_cursor_down(&mut self) {
        // Lorenzo: I fixed a bug here with -1, where the cursor would roll off the screen.
        if self.items.is_empty() || self.cursor >= self.items.len() - 1 {
            return;
        }
        if let Some(res) = self.cursor.checked_add(1) {
            self.cursor = res;
        }
    }

    pub fn move_cursor_up(&mut self) {
        if self.items.is_empty() {
            return;
        }
        if let Some(res) = self.cursor.checked_sub(1) {
            self.cursor = res;
        }
    }

    pub fn select_item(&mut self) {
        let state = self.state.as_mut().unwrap();
        if let Some(selected) = state.selected() {
            // ^^^ returns usize index
            self.selected = Some(selected);
        }
    }

    pub fn remove_display_option(&mut self, opt: &DisplayOpts) {
        self.opts.retain(|x| x != opt);
    }
    pub fn remove_all_display_options(&mut self) {
        self.opts.clear();
    }

    pub fn write_response(&mut self) -> Result<(), String> {
        if let Some(ref mut cmd) = self.command {
            match cmd {
                Command::Curl(ref mut curl) => match curl.write_output() {
                    Ok(_) => Ok(()),
                    Err(e) => Err(e.to_string()),
                },
                _ => Err(String::from("downloads output is written by default")),
            }
        } else {
            Err(String::from("No command to write output for"))
        }
    }

    pub fn get_response_headers(&mut self) -> String {
        if let Ok(response) = Response::from_raw_string(&self.response.as_ref().unwrap().clone()) {
            response.get_headers().to_string()
        } else {
            return String::from("No headers found");
        }
    }

    pub fn execute_command(&mut self) -> Result<(), String> {
        match self.command.as_mut().unwrap() {
            Command::Curl(ref mut curl) => {
                // continue lazy loading by only opening connection if we need to
                if curl.will_store_command() && self.db.is_none() {
                    self.db = Some(Box::new(DB::new().unwrap()));
                }
                match curl.execute(&mut self.db) {
                    Ok(_) => {
                        self.response = curl.get_response();
                        Ok(())
                    }
                    Err(e) => Err(e.to_string()),
                }
            }
            Command::Wget(wget) => wget.execute(),
        }
    }

    pub fn get_saved_commands(&mut self) -> Result<Vec<String>, String> {
        if self.db.is_none() {
            self.db = Some(Box::new(DB::new().unwrap()));
        }
        let db = self.db.as_ref().unwrap();
        let commands = db.get_commands().unwrap();
        let mut saved_commands = Vec::new();
        for command in commands {
            saved_commands.push(format!("{}", command));
        }
        Ok(saved_commands)
    }

    pub fn get_saved_keys(&mut self) -> Result<Vec<String>, String> {
        if self.db.is_none() {
            self.db = Some(Box::new(DB::new().unwrap()));
        }
        let db = self.db.as_ref().unwrap();
        let keys = db.get_keys().unwrap();
        let mut saved_keys = Vec::new();
        for key in keys {
            saved_keys.push(format!("{}", key));
        }
        Ok(saved_keys)
    }

    pub fn add_saved_key(&mut self, key: String) -> Result<(), rusqlite::Error> {
        if self.db.is_none() {
            self.db = Some(Box::new(DB::new().unwrap()));
        }
        let db = self.db.as_ref().unwrap();
        db.add_key(&key)
    }

    // Display option is some state that requires us to display the users
    // current selection on the screen so they know what they have selected
    // Lorenzo - Changing this because I dont think its doing what I want it to do.
    pub fn has_display_option(&self, opt: &DisplayOpts) -> bool {
        for element in self.opts.iter() {
            // I only care if its the same KIND of option, not the same value
            // This is annoying, I tried to do this an easier way

            match *element {
                DisplayOpts::URL(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::Headers(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::Outfile(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::Response(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::SaveCommand => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::Verbose => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::RecDownload(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::Auth(_) => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
                DisplayOpts::SaveToken => {
                    if mem::discriminant(opt) == mem::discriminant(element) {
                        return true;
                    }
                }
            }
        }
        // Otherwise, its not there.
        false
    }

    // Lorenzo - Im adding this function as a slightly more
    // robust version of has_display_option, to test if we should be replacing a value or adding a new one
    fn should_add_option(&self, opt: &DisplayOpts) -> bool {
        // Lets match the type of display option
        // We know that only 1 URL should ever be added,
        // So if we're adding a URL we should replace it if it already exists
        match opt {
            DisplayOpts::URL(_) => !self.has_display_option(opt), // URL should be replaced if exists
            DisplayOpts::Headers(_) => true, // Headers should be "pushed" or added
            DisplayOpts::Outfile(_) => !self.has_display_option(opt), // Outfile should be replaced
            DisplayOpts::Verbose => !self.has_display_option(opt), // Verbose should be toggled
            DisplayOpts::SaveCommand => !self.has_display_option(opt), // Save command should be toggled
            DisplayOpts::Response(_) => !self.has_display_option(opt), // Response should be replaced
            DisplayOpts::RecDownload(_) => !self.has_display_option(opt), // Recursive download depth should be replaced
            DisplayOpts::Auth(_) => !self.has_display_option(opt),        // Auth should be replaced
            DisplayOpts::SaveToken => !self.has_display_option(opt), // Save token should be toggled
        }
    }

    pub fn set_response(&mut self, response: String) {
        self.response = Some(response.clone());
        if let Some(cmd) = &mut self.command {
            cmd.set_response(&response);
        }
    }

    fn should_toggle(&self, opt: &DisplayOpts) -> bool {
        match opt {
            DisplayOpts::Verbose => true,
            DisplayOpts::SaveCommand => true,
            DisplayOpts::SaveToken => true,
            _ => false,
        }
    }

    pub fn toggle_display_option(&mut self, opt: DisplayOpts) {
        match opt.clone() {
            DisplayOpts::Verbose => self
                .command
                .as_mut()
                .unwrap()
                .set_verbose(!self.opts.contains(&opt)),
            DisplayOpts::SaveCommand => self
                .command
                .as_mut()
                .unwrap()
                .save_command(!self.opts.contains(&opt)),
            DisplayOpts::SaveToken => {
                if self.command.is_some() && self.has_auth() {
                    self.command.as_mut().unwrap().save_token();
                }
            }
            _ => {}
        }
    }

    pub fn add_display_option(&mut self, opt: DisplayOpts) {
        // We either add the option or we replace the existing one

        // first we look and see if its an option we can just toggle..
        if self.should_toggle(&opt) {
            self.toggle_display_option(opt);
            return;
        }

        if self.should_add_option(&opt) {
            self.opts.push(opt.clone());
            match opt {
                DisplayOpts::Headers((key, value)) => {
                    // Push Header To Shareable Command
                    self.command
                        .as_mut()
                        .unwrap()
                        .add_headers(vec![format!("{}:{}", key, value)]);
                }
                DisplayOpts::URL(url) => {
                    self.command.as_mut().unwrap().set_url(url);
                }
                DisplayOpts::Outfile(outfile) => {
                    self.command.as_mut().unwrap().set_outfile(&outfile);
                }
                _ => {
                    // Nothing
                    // This display opt does not factor into the sharable command.
                }
            }
        } else {
            // We Should Replace An Option
            // The user has already added this command option, and so therefore, we should replace the old value with the new value.
            //self.opts.retain(|x| x != &opt);
            // Sorry, this is my way to do this idk if its the right way, but this is what makes sense to me in my head
            for element in self.opts.iter_mut() {
                // Same thing down here, I only care if its the same KIND of option, not the same value
                // Again, this is annoying, I tried to do this an easier way
                // but mem::discriminant doesnt like element as a comparison so I need to be particular
                // Sorry lets refactor this
                // TODO: Refactor This.

                // We Want To Just Replace A URL
                if let DisplayOpts::URL(_) = element {
                    *element = opt; // Copy The New URL Into The Old One
                    return;
                }

                // TODO: Headers Will Be Handled Differently.

                // TODO: Outfile Will Be Handled Differently.

                // TODO: Verbose & Save Command Will Be Handled Differently.

                // TODO: Other Shit
            }
        }
    }
}
