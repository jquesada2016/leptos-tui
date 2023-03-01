use super::CoreComponent;
use crate::{
  IntoView,
  View,
  Widget,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Unit;

impl Widget for Unit {
  fn name(&self) -> std::borrow::Cow<'static, str> {
    "Unit".into()
  }

  fn layout(&mut self, limits: crate::Limits) -> crate::Size {
    limits.min_size()
  }

  fn draw(&self, surface: &mut dyn crate::DrawSurface) {}
}

impl IntoView for Unit {
  fn into_view(self, cx: leptos_reactive::Scope) -> crate::View {
    CoreComponent::Unit(self).into_view(cx)
  }
}
