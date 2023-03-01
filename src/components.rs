mod center;
mod dyn_child;
mod text;
mod unit;

use crate::{
  ArcView,
  ArcWidget,
  IntoView,
  Limits,
  Size,
  View,
  Widget,
};
pub use center::*;
use core::fmt;
pub use dyn_child::*;
use std::sync::{
  Arc,
  Mutex,
};
pub use text::*;
pub use unit::*;

#[derive(Debug)]
pub enum CoreComponent {
  Unit(Unit),
  DynChild(DynChild),
  Text(Text),
}

impl Default for CoreComponent {
  fn default() -> Self {
    Self::Unit(Unit)
  }
}

impl fmt::Display for CoreComponent {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::Unit(arg0) => writeln!(f, "<() />"),
      Self::DynChild(dyn_child) => dyn_child.fmt(f),
      Self::Text(text) => writeln!(f, "{}", text),
    }
  }
}

impl IntoView for CoreComponent {
  fn into_view(self, _: leptos_reactive::Scope) -> crate::View {
    View::CoreComponent(self)
  }
}

impl Widget for CoreComponent {
  fn name(&self) -> std::borrow::Cow<'static, str> {
    match self {
      CoreComponent::Unit(unit) => unit.name(),
      CoreComponent::DynChild(dyn_child) => dyn_child.name(),
      CoreComponent::Text(text) => text.name(),
    }
  }

  fn layout(&mut self, limits: crate::Limits) -> crate::Size {
    match self {
      Self::Unit(unit) => unit.layout(limits),
      Self::DynChild(dyn_child) => dyn_child.layout(limits),
      Self::Text(text) => text.layout(limits),
    }
  }

  fn draw(&self, surface: &mut dyn crate::DrawSurface) {
    match self {
      Self::Unit(unit) => unit.draw(surface),
      Self::DynChild(dyn_child) => dyn_child.draw(surface),
      Self::Text(text) => text.draw(surface),
    }
  }
}

#[track_caller]
pub fn debug_assert_size_within_limits(
  limits: Limits,
  size: Size,
  created_at: &'static std::panic::Location<'static>,
) {
  debug_assert!(
    limits.is_size_within_limits(size),
    "size exceeds limits\nmax size: {:?}\nmin size: {:?}\nsize: \
     {:?}\ncreated_at: {created_at}",
    limits.max_size(),
    limits.min_size(),
    size,
  );
}
