/// Model represents the global states needed in FZF.
/// It will also define how the states will be shown on the terminal


use std::sync::{Arc, RwLock};
use item::{Item, MatchedItem};
use ncurses::*;
use std::cmp;
use std::cell::RefCell;
use orderedvec::OrderedVec;

pub struct Model {
    pub query: String,
    query_cursor: i32,  // > qu<query_cursor>ery
    num_matched: u64,
    num_total: u64,
    pub items: Arc<RwLock<Vec<Item>>>, // all items
    pub matched_items: RefCell<OrderedVec<MatchedItem>>,
    item_cursor: usize, // the index of matched item currently highlighted.
    line_cursor: usize, // line No.
    item_start_pos: usize, // for screen scroll.
    max_y: i32,
    max_x: i32,
}

impl Model {
    pub fn new() -> Self {
        let mut max_y = 0;
        let mut max_x = 0;
        getmaxyx(stdscr, &mut max_y, &mut max_x);

        Model {
            query: String::new(),
            query_cursor: 0,
            num_matched: 0,
            num_total: 0,
            items: Arc::new(RwLock::new(Vec::new())),
            matched_items: RefCell::new(OrderedVec::new()),
            item_cursor: 0,
            line_cursor: (max_y - 3) as usize,
            item_start_pos: 0,
            max_y: max_y,
            max_x: max_x,
        }
    }

    pub fn output(&self) {
        let items = self.items.read().unwrap();
        for item in items.iter() {
            if item.selected {
                println!("{}", item.text);
            }
        }
        //println!("{:?}", items[self.matched_items[self.item_cursor].index].text);
        //items[self.matched_items[self.item_cursor].index].selected = s;
    }

    pub fn toggle_select(&self, selected: Option<bool>) {
        let mut items = self.items.write().unwrap();
        let mut matched_items = self.matched_items.borrow_mut();
        if items.len() <= 0 {
            return;
        }

        items[matched_items.get(self.item_cursor).unwrap().index].toggle_select(selected);
    }

    pub fn update_query(&mut self, query: String, cursor: i32) {
        self.query = query;
        self.query_cursor = cursor;
    }

    pub fn update_process_info(&mut self, matched: u64, total: u64) {
        self.num_matched = matched;
        self.num_total = total;
    }

    pub fn push_item(&mut self, item: MatchedItem) {
        self.matched_items.borrow_mut().push(item);
    }

    pub fn clear_items(&mut self) {
        self.matched_items.borrow_mut().clear();
    }

    pub fn move_line_cursor(&mut self, diff: i32) {

        let y = self.line_cursor as i32 + diff;
        let item_y = cmp::max(0, self.item_cursor as i32 - diff);
        let screen_height = (self.max_y - 3) as usize;

        match y {
            y if y < 0 => {
                self.line_cursor = 0;
                self.item_cursor = cmp::min(item_y as usize, self.matched_items.borrow().len()-1);
                self.item_start_pos = self.item_cursor - screen_height;
            }

            y if y > screen_height as i32 => {
                self.line_cursor = screen_height;
                self.item_cursor = cmp::max(0, item_y as usize);
                self.item_start_pos = self.item_cursor;
            }

            y => {
                self.line_cursor = y as usize;
                self.item_cursor = item_y as usize;
            }
        }
    }

    pub fn print_query(&self) {
        // > query
        mv(self.max_y-1, 0);
        addstr("> ");
        addstr(&self.query);
        mv(self.max_y-1, self.query_cursor+2);
    }

    pub fn print_info(&self) {
        mv(self.max_y-2, 0);
        addstr(format!("  {}/{}", self.num_matched, self.num_total).as_str());
    }

    fn print_item(&self, item: &Item, matched: &MatchedItem) {
        //let shown_str: String = item.text.chars().take((self.max_x-1) as usize).collect();
        if item.selected {
            printw(">");
        } else {
            printw(" ");
        }

        for (idx, ch) in item.text.chars().enumerate() {
            if idx == matched.matched_range_chars.0 {
                attron(A_BOLD());
            }
            if idx == matched.matched_range_chars.1 {
                attroff(A_BOLD());
            }
            addch(ch as u64);
            if idx >= (self.max_x - 3) as usize {
                break;
            }
        }

        //addstr(&shown_str);
    }

    pub fn print_items(&self) {
        let items = self.items.read().unwrap();
        let mut matched_items = self.matched_items.borrow_mut();

        let mut y = self.max_y - 3;
        let mut i = self.item_start_pos;
        while let Some(matched) = matched_items.get(i) {
            i+=1;

            mv(y, 0);
            let is_current_line = y == self.line_cursor as i32;

            if is_current_line {
                printw(">");
            } else {
                printw(" ");
            }

            self.print_item(&items[matched.index], matched);

            y -= 1;
            if y < 0 {
                break;
            }
        }
    }

    pub fn refresh(&self) {
        refresh();
    }

    pub fn display(&self) {
        erase();
        self.print_items();
        self.print_info();
        self.print_query();
        self.refresh();
    }
}
