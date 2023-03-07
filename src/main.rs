#![allow(warnings)]

use crossterm as ct;
use ct::{
  cursor as c,
  execute,
  style as s,
  terminal as t,
};
use leptos_tui::*;

fn main() {
  run(|cx| Center::new().child(Button::new("Hello, World!")));
}
