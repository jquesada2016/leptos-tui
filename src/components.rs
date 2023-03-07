#[cfg(test)]
#[macro_use]
mod text_macros;
mod button;
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
pub use button::*;
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

#[cfg(test)]
fn get_view<V: IntoView>(
  f: impl FnOnce(leptos_reactive::Scope) -> V + 'static,
) -> View {
  let rt = leptos_reactive::create_runtime();

  let view = leptos_reactive::run_scope(rt, move |cx| f(cx).into_view(cx));

  view
}

#[cfg(test)]
fn get_widget_output(
  mut widget: impl Widget,
  max_size: impl Into<Size>,
  min_size: impl Into<Size>,
) -> (Size, String) {
  let max_size = max_size.into();
  let min_size = min_size.into();

  let mut buf = vec![];

  let mut surface = crate::BufDrawSurface::new(&mut buf, max_size);

  let size = widget.layout(Limits {
    min_width: min_size.width,
    max_width: max_size.width,
    min_height: min_size.height,
    max_height: max_size.height,
  });

  widget.draw(&mut surface);

  drop(surface);

  let output = std::str::from_utf8(&buf).unwrap().to_owned();

  (size, output)
}
