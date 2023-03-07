#![feature(panic_update_hook, closure_track_caller)]
#![allow(warnings)]

mod components;
mod surface;

pub use components::*;
use crossterm::{
  cursor::{
    Hide,
    MoveTo,
    MoveToNextLine,
    Show,
  },
  event::{
    DisableMouseCapture,
    EnableMouseCapture,
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers,
  },
  style::{
    Attribute,
    Color,
    Print,
    Stylize,
  },
  terminal::{
    BeginSynchronizedUpdate,
    Clear,
    ClearType,
    EndSynchronizedUpdate,
    EnterAlternateScreen,
    LeaveAlternateScreen,
  },
  QueueableCommand,
};
use leptos_reactive::Scope;
use std::{
  borrow::Cow,
  fmt,
  io::{
    self,
    BufWriter,
    Write,
  },
  sync::{
    Arc,
    Mutex,
  },
};
pub use surface::*;

#[macro_use]
mod utils;

api_planning! {
  let (sig, set_sig) = create_signal(cx);

  view! { cx,
    <Row>
      {sig}

      <button
        on:click=|| set_sig.update(|sig| *sig = !sig)
      >"Click"</button>
    </Row>
  }
}

pub type ArcWidget = Arc<Mutex<dyn Widget>>;
pub type ArcView = Arc<Mutex<View>>;

pub trait Widget: fmt::Debug + Send + Sync {
  fn name(&self) -> Cow<'static, str> {
    std::any::type_name::<Self>().into()
  }

  fn layout(&mut self, limits: Limits) -> Size;

  fn draw(&self, surface: &mut dyn DrawSurface);
}

pub trait IntoView {
  fn into_view(self, cx: Scope) -> View;
}

impl<F, V> IntoView for F
where
  F: Fn() -> V + Send + Sync + 'static,
  V: IntoView,
{
  #[track_caller]
  fn into_view(self, cx: Scope) -> View {
    DynChild::new(self).into_view(cx)
  }
}

impl IntoView for () {
  fn into_view(self, cx: Scope) -> View {
    Unit.into_view(cx)
  }
}

impl<T: IntoView> IntoView for Option<T> {
  fn into_view(self, cx: Scope) -> View {
    if let Some(view) = self {
      view.into_view(cx)
    } else {
      Unit.into_view(cx)
    }
  }
}

impl IntoView for String {
  fn into_view(self, cx: Scope) -> View {
    Text::new(self).into_view(cx)
  }
}

impl IntoView for &'static str {
  fn into_view(self, cx: Scope) -> View {
    Text::new(self).into_view(cx)
  }
}

#[derive(Debug)]
pub enum View {
  CoreComponent(CoreComponent),
  Widget(ArcWidget),
}

impl Default for View {
  fn default() -> Self {
    Self::CoreComponent(CoreComponent::default())
  }
}

impl fmt::Display for View {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::CoreComponent(component) => component.fmt(f),
      Self::Widget(arg0) => todo!(),
    }
  }
}

impl IntoView for View {
  fn into_view(self, _: Scope) -> View {
    self
  }
}

impl Widget for View {
  fn name(&self) -> Cow<'static, str> {
    match self {
      View::CoreComponent(component) => component.name(),
      View::Widget(widget) => widget.lock().unwrap().name(),
    }
  }

  fn layout(&mut self, limits: Limits) -> Size {
    match self {
      View::Widget(widget) => widget.lock().unwrap().layout(limits),
      View::CoreComponent(component) => component.layout(limits),
    }
  }

  fn draw(&self, surface: &mut dyn DrawSurface) {
    match self {
      View::Widget(widget) => widget.lock().unwrap().draw(surface),
      View::CoreComponent(component) => component.draw(surface),
    }
  }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq)]
pub struct Limits {
  pub min_width: u16,
  pub max_width: u16,
  pub min_height: u16,
  pub max_height: u16,
}

/// Converts the tuple using strict `[Limits]` of `(width, height)`.
impl From<(u16, u16)> for Limits {
  fn from((width, height): (u16, u16)) -> Self {
    Self::strict(width, height)
  }
}

/// Converts the tuple using strict `[Limits]` of
/// `(max width, min_width, max_height, min_height)`.
impl From<(u16, u16, u16, u16)> for Limits {
  fn from(
    (max_width, min_width, max_height, min_height): (u16, u16, u16, u16),
  ) -> Self {
    Self {
      min_width,
      max_width,
      min_height,
      max_height,
    }
  }
}

/// Converts the tuple using `(max_size, min_size)`.
impl<S1, S2> From<(S1, S2)> for Limits
where
  S1: Into<Size>,
  S2: Into<Size>,
{
  fn from((max_size, min_size): (S1, S2)) -> Self {
    let Size {
      width: max_width,
      height: max_height,
    } = max_size.into();
    let Size {
      width: min_width,
      height: min_height,
    } = min_size.into();

    Self {
      min_width,
      max_width,
      min_height,
      max_height,
    }
  }
}

impl Limits {
  /// Gets the [`Size`] of the area represented by these [`Limits`].
  pub fn max_size(self) -> Size {
    Size {
      width: self.max_width,
      height: self.max_height,
    }
  }

  pub fn min_size(self) -> Size {
    Size {
      width: self.min_width,
      height: self.min_height,
    }
  }

  pub fn is_size_within_limits(self, size: Size) -> bool {
    let max_size = self.max_size();
    let min_size = self.min_size();

    max_size >= size && min_size <= size
  }

  pub fn strict(width: u16, height: u16) -> Self {
    Self {
      min_width: width,
      max_width: width,
      min_height: height,
      max_height: height,
    }
  }
}

impl std::cmp::PartialOrd for Size {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.linear_length().cmp(&other.linear_length()))
  }
}

impl std::cmp::Ord for Size {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self.cmp(other)
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Size {
  pub width: u16,
  pub height: u16,
}

impl From<(u16, u16)> for Size {
  fn from((width, height): (u16, u16)) -> Self {
    Self { width, height }
  }
}

impl Size {
  /// Gets the length of the area of this [`Size`]. Equals `width * height`.
  pub fn linear_length(self) -> usize {
    self.width as usize * self.height as usize
  }

  pub fn into_strict_limits(self) -> Limits {
    Limits::strict(self.width, self.height)
  }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, derive_more::Add)]
pub struct XY {
  pub x: u16,
  pub y: u16,
}

impl From<(u16, u16)> for XY {
  fn from((x, y): (u16, u16)) -> Self {
    XY { x, y }
  }
}

#[track_caller]
pub fn run<V: IntoView>(f: impl FnOnce(Scope) -> V + 'static) {
  // Update the panic hook to make sure we leave the terminal in a
  // usable state on panic
  std::panic::update_hook(|prev, info| {
    use std::io::Write;

    cleanup_screen();

    prev(info);

    let mut file = std::fs::File::create("panic.txt").unwrap();

    write!(file, "{}", info);
  });

  let runtime = leptos_reactive::create_runtime();

  setup_screen();

  let disposer = leptos_reactive::create_scope(
    runtime,
    #[track_caller]
    move |cx| {
      let mut view = f(cx).into_view(cx);

      let mut surface = BufDrawSurface::default();

      render_view(&mut surface, &mut view);

      loop {
        match crossterm::event::read().unwrap() {
          Event::Key(KeyEvent {
            code,
            modifiers,
            state,
            ..
          }) => match code {
            KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
              break;
            }
            _ => {}
          },
          Event::Resize(width, height) => {
            surface.resize(Size { width, height });

            render_view(&mut surface, &mut view);
          }
          _ => {}
        }
      }
    },
  );

  cleanup_screen();
}

fn setup_screen() {
  let mut stdout = std::io::stdout();

  crossterm::terminal::enable_raw_mode();

  stdout
    .queue(EnterAlternateScreen)
    .unwrap()
    .queue(EnableMouseCapture)
    .unwrap()
    .flush();
}

fn cleanup_screen() {
  let mut stdout = std::io::stdout();

  crossterm::terminal::disable_raw_mode();

  stdout
    .queue(EndSynchronizedUpdate)
    .unwrap()
    .queue(Clear(ClearType::All))
    .unwrap()
    .queue(LeaveAlternateScreen)
    .unwrap()
    .queue(DisableMouseCapture)
    .unwrap()
    .queue(Show)
    .unwrap()
    .flush();
}

#[track_caller]
fn render_view(surface: &mut StdoutDrawSurface, view: &mut View) {
  let limits = surface.size().into_strict_limits();

  surface.size = limits.max_size();
  surface.top_left = XY::default();

  let child_size = view.layout(limits);

  debug_assert_size_within_limits(
    limits,
    child_size,
    std::panic::Location::caller(),
  );

  surface
    .buf
    .queue(BeginSynchronizedUpdate)
    .unwrap()
    .queue(Clear(ClearType::All))
    .unwrap()
    .queue(Hide)
    .unwrap();

  view.draw(surface);

  surface.buf.flush().unwrap();
}
