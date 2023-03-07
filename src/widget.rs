use crate::{
  DrawSurface,
  Limits,
  Size,
};
use crossterm::event::KeyEvent;
use std::{
  borrow::Cow,
  fmt,
};

pub trait Widget: fmt::Debug + Send + Sync {
  fn name(&self) -> Cow<'static, str> {
    std::any::type_name::<Self>().into()
  }

  fn layout(&mut self, limits: Limits) -> Size;

  fn draw(&self, surface: &mut dyn DrawSurface);

  fn needs_focus(&self) -> Option<bool> {
    None
  }

  fn on(&mut self, event: Event) -> EventHandlerResult {
    EventHandlerResult::Bubble
  }

  fn focus(&mut self) {}

  fn blur(&mut self) {}
}

pub enum Event {
  Key(KeyEvent),
  NextFocus,
  PrevFocus,
  Batch(Vec<Event>),
}

pub enum EventHandlerResult {
  /// The event was captured by the widget, and should
  /// not be handled by the parent.
  Captured,
  /// The event may or may not have been handled by this widget,
  /// but it should be handled by the parent.
  Bubble,
  /// This widget might have done something with this event, and now
  /// it wants the parent to respond to this new event instead.
  Mapped(Event),
}
