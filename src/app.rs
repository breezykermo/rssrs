use webbrowser;
use chrono::{TimeZone, Local};
use super::db::{Database, Item};
use super::util::{StatefulTable, load_feeds};
use regex::{Regex, RegexBuilder};

pub enum InputMode {
    Normal,
    Search,
}

pub enum Filter {
    All,
    Channel(String),
}

pub enum Status {
    Idle,
    Updating
}

pub struct App {
    db: Database,
    feeds_path: String,

    pub focus_reader: bool,
    pub status: Status,
    pub input_mode: InputMode,

    filter: Filter,
    pub items: Vec<Item>,
    pub table: StatefulTable,

    pub search_results: Vec<usize>,
    pub search_input_raw: String,
    pub search_input: Option<Regex>,
    pub search_query: Option<Regex>,

    pub reader_scroll: u16,
}

impl App {
    pub fn new(db_path: &String, feeds_path: &String) -> App {
        App {
            db: Database::new(db_path),
            feeds_path: feeds_path.clone(),

            input_mode: InputMode::Normal,
            focus_reader: false,
            status: Status::Idle,

            filter: Filter::All,
            items: Vec::new(),
            table: StatefulTable::new(),

            search_input: None,
            search_query: None,
            search_results: Vec::new(),
            search_input_raw: String::new(),

            reader_scroll: 0,
        }
    }

    pub fn load_items(&mut self) {
        // Load items according to filter
        // TODO use filter
        // TODO why do I need both flat map and flatten?
        self.items = load_feeds(&self.feeds_path)
            .flat_map(|(feed_url, _tags)| {
                self.db.get_channel_items(&feed_url).ok()
            }).flatten().collect();

        // Most recent first
        self.items.sort_by_cached_key(|i| match i.published_at {
            Some(ts) => -ts,
            None => 0
        });

        // Load item data into table
        self.table.set_items(self.items.iter().map(|i| {
            let pub_date = match i.published_at {
                Some(ts) => Local.timestamp(ts, 0).format("%m/%d/%y %H:%M").to_string(),
                None => "<no pub date>".to_string()
            };

            vec![
                i.title.as_deref().unwrap_or("<no title>").to_string(),
                pub_date,
                i.channel.clone(),
            ]
        }).collect());
    }

    pub fn mark_selected_read(&mut self) {
        match self.table.state.selected() {
            Some(i) => {
                self.items[i].read = true;
                self.db.set_item_read(&self.items[i], true);
            },
            None => {}
        }
    }

    pub fn mark_selected_unread(&mut self) {
        match self.table.state.selected() {
            Some(i) => {
                self.items[i].read = false;
                self.db.set_item_read(&self.items[i], false);
            },
            None => {}
        }
    }

    pub fn build_query(&self, query: &String) -> Regex {
        let regex = format!(r"({})", query);
        RegexBuilder::new(&regex).case_insensitive(true).build().expect("Invalid regex")
    }

    pub fn execute_search(&mut self, query: &Regex) {
        self.search_results = self.items.iter().enumerate().filter(|(_, item)| {
            match &item.title {
                Some(title) => query.is_match(title),
                None => false
            }
        }).map(|i| i.0).collect();
    }

    pub fn start_search(&mut self) {
        self.input_mode = InputMode::Search;
    }

    pub fn end_search(&mut self) {
        self.search_input_raw.clear();
        self.input_mode = InputMode::Normal;
    }

    pub fn scroll_items_up(&mut self) {
        self.table.previous();
        self.mark_selected_read();
        self.reset_reader_scroll();
    }

    pub fn scroll_items_down(&mut self) {
        self.table.next();
        self.mark_selected_read();
        self.reset_reader_scroll();
    }

    pub fn page_items_up(&mut self) {
        self.table.jump_backward(5);
        self.mark_selected_read();
        self.reset_reader_scroll();
    }

    pub fn page_items_down(&mut self) {
        self.table.jump_forward(5);
        self.mark_selected_read();
        self.reset_reader_scroll();
    }

    pub fn reset_reader_scroll(&mut self) {
        self.reader_scroll = 0;
    }

    pub fn scroll_reader_up(&mut self) {
        if self.reader_scroll > 0 {
            self.reader_scroll -= 1;
        }
    }

    pub fn scroll_reader_down(&mut self) {
        self.reader_scroll += 1;
    }

    pub fn toggle_focus_reader(&mut self) {
        self.focus_reader = !self.focus_reader;
    }

    pub fn open_selected(&self) {
        match self.table.state.selected() {
            Some(i) => {
                match &self.items[i].url {
                    Some(url) => {
                        webbrowser::open(&url);
                    },
                    None => {}
                }
            },
            None => {}
        };
    }

    pub fn jump_to_next_result(&mut self) {
        if self.search_results.len() > 0 {
            match self.table.state.selected() {
                Some(i) => {
                    if i >= *self.search_results.last().unwrap() {
                        self.table.state.select(Some(self.search_results[0]));
                    } else {
                        for si in &self.search_results {
                            if *si > i {
                                self.table.state.select(Some(*si));
                                break;
                            }
                        }
                    }
                },
                None => {
                    self.table.state.select(Some(self.search_results[0]));
                }
            }
        }
    }

    pub fn jump_to_prev_result(&mut self) {
        if self.search_results.len() > 0 {
            match self.table.state.selected() {
                Some(i) => {
                    if i <= self.search_results[0] {
                        let last = self.search_results.last().unwrap();
                        self.table.state.select(Some(*last));
                    } else {
                        for si in self.search_results.iter().rev() {
                            if *si < i {
                                self.table.state.select(Some(*si));
                                break;
                            }
                        }
                    }
                },
                None => {
                    self.table.state.select(Some(self.search_results[0]));
                }
            }
        }
    }
}