use super::{
  debug_assert_size_within_limits,
  CoreComponent,
  Unit,
};
use crate::{
  ArcView,
  ArcWidget,
  IntoView,
  View,
  Widget,
};
use core::fmt;
use leptos_reactive::{
  create_effect,
  Scope,
};
use std::sync::Arc;

pub struct DynChild {
  child_fn: Arc<dyn Fn(Scope) -> View + Send + Sync>,
  child: ArcView,
  #[cfg(debug_assertions)]
  created_at: &'static std::panic::Location<'static>,
}

impl fmt::Debug for DynChild {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("DynChild")
      .field("child_fn", &"Fn() -> View")
      .field("child", &self.child)
      .field("created_at", &self.created_at)
      .finish()
  }
}

impl fmt::Display for DynChild {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    writeln!(f, "<DynChild>")?;
    (self.child.lock().unwrap()).fmt(f)?;
    writeln!(f, "</DynChild>")
  }
}

impl IntoView for DynChild {
  fn into_view(self, cx: Scope) -> View {
    let child_fn = self.child_fn.clone();
    let child = self.child.clone();

    create_effect(cx, move |_| {
      let new_child = child_fn(cx);

      *child.lock().unwrap() = new_child;
    });

    CoreComponent::DynChild(self).into_view(cx)
  }
}

impl Widget for DynChild {
  fn name(&self) -> std::borrow::Cow<'static, str> {
    "DynChild".into()
  }

  fn layout(&mut self, limits: crate::Limits) -> crate::Size {
    let child_size = self.child.lock().unwrap().layout(limits);

    debug_assert_size_within_limits(limits, child_size, self.created_at);

    child_size
  }

  fn draw(&self, surface: &mut dyn crate::DrawSurface) {
    self.child.lock().unwrap().draw(surface)
  }
}

impl DynChild {
  #[track_caller]
  pub fn new<V: IntoView>(
    child_fn: impl Fn() -> V + Send + Sync + 'static,
  ) -> Self {
    Self {
      child_fn: Arc::new(move |cx| child_fn().into_view(cx)),
      child: Default::default(),
      created_at: std::panic::Location::caller(),
    }
  }
}
