use super::debug_assert_size_within_limits;
use crate::{
  ArcView,
  ArcWidget,
  DrawSurface,
  IntoView,
  Limits,
  Size,
  View,
  Widget,
};
use leptos_reactive::Scope;
use std::{
  fmt,
  marker::PhantomData,
  sync::{
    Arc,
    Mutex,
  },
};

pub struct Center<State> {
  state: PhantomData<State>,
  child_fn: Option<Box<dyn FnOnce(Scope) -> View + Send + Sync>>,
  child: Option<ArcView>,
  child_size: Size,
  #[cfg(debug_assertions)]
  created_at: &'static std::panic::Location<'static>,
}

impl fmt::Debug for Center<WithChild> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Center")
      .field("state", &self.state)
      .field("child", &self.child)
      .field("child_size", &self.child_size)
      .field("created_at", &self.created_at)
      .finish()
  }
}

impl Default for Center<MissingChild> {
  #[track_caller]
  fn default() -> Self {
    Self {
      state: PhantomData,
      child_fn: None,
      child: None,
      child_size: Default::default(),
      created_at: std::panic::Location::caller(),
    }
  }
}

impl IntoView for Center<WithChild> {
  fn into_view(mut self, cx: Scope) -> View {
    let child = self.child_fn.take().unwrap()(cx);

    self.child = Some(Arc::new(Mutex::new(child)));

    View::Widget(Arc::new(Mutex::new(self)))
  }
}

impl Widget for Center<WithChild> {
  fn name(&self) -> std::borrow::Cow<'static, str> {
    "Center".into()
  }

  fn layout(&mut self, limits: Limits) -> Size {
    let child_limits = Limits {
      min_width: 0,
      min_height: 0,
      ..limits
    };

    let child_size = self
      .child
      .as_ref()
      .unwrap()
      .lock()
      .unwrap()
      .layout(child_limits);

    self.child_size = child_size;

    limits.max_size()
  }

  fn draw(&self, surface: &mut dyn DrawSurface) {
    let size = surface.size();

    let child_size = self.child_size;

    surface.shrink_centered(
      child_size,
      Box::new(|surface| {
        self.child.as_ref().unwrap().lock().unwrap().draw(surface);
      }),
    );
  }
}

impl Center<MissingChild> {
  #[track_caller]
  pub fn new() -> Self {
    Self::default()
  }
}

impl Center<MissingChild> {
  pub fn child(
    self,
    child: impl IntoView + Send + Sync + 'static,
  ) -> Center<WithChild> {
    Center {
      state: PhantomData,
      child_fn: Some(Box::new(|cx| child.into_view(cx))),
      child: None,
      child_size: Size::default(),
      created_at: self.created_at,
    }
  }
}

#[derive(Debug)]
pub struct MissingChild;

#[derive(Debug)]
pub struct WithChild;
