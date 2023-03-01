#![feature(panic_update_hook, closure_track_caller)]
#![allow(warnings)]

pub use components::*;
use crossterm::{
  cursor::{
    MoveTo,
    MoveToNextLine,
  },
  event::{
    DisableMouseCapture,
    EnableMouseCapture,
    Event,
    KeyCode,
    KeyEvent,
    KeyModifiers,
  },
  style::Print,
  terminal::{
    BeginSynchronizedUpdate,
    Clear,
    ClearType,
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
mod components;

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

pub trait DrawSurface {
  fn size(&self) -> Size;

  /// Writes the given data starting at the given coordinates.
  /// Returns [`Err`] with the data that was written out of bounds,
  /// if any.
  ///
  /// Note:
  /// The coordinates are relative to the provided [`Limits`]
  /// and are not absolute.
  fn write<'a>(&mut self, at: XY, data: &'a str) -> Result<(), &'a str>;

  fn shrink(
    &mut self,
    top_left: XY,
    size: Size,
    f: Box<dyn FnOnce(&mut dyn DrawSurface) + '_>,
  );

  fn shrink_centered(
    &mut self,
    size: Size,
    f: Box<dyn FnOnce(&mut dyn DrawSurface) + '_>,
  ) {
    let surface_size = self.size();

    debug_assert!(
      surface_size >= size,
      "size to shrink by is larger than available area",
    );

    let x_offset = (surface_size.width - size.width) / 2;
    let y_offset = (surface_size.height - size.height) / 2;

    self.shrink(
      XY {
        x: x_offset,
        y: y_offset,
      },
      size,
      f,
    )
  }
}

pub trait Widget: fmt::Debug + Send + Sync {
  fn name(&self) -> Cow<'static, str>;

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

  fn from_stdout() -> Self {
    let (width, height) = crossterm::terminal::size().unwrap();

    Limits::strict(width, height)
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

impl Size {
  /// Gets the length of the area of this [`Size`]. Equals `width * height`.
  pub fn linear_length(self) -> usize {
    self.width as usize * self.height as usize
  }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, derive_more::Add)]
pub struct XY {
  pub x: u16,
  pub y: u16,
}

#[derive(Debug)]
struct StdOutDrawSurface {
  stdout: BufWriter<std::io::Stdout>,
  top_left: XY,
  size: Size,
}

impl Default for StdOutDrawSurface {
  fn default() -> Self {
    Self {
      stdout: BufWriter::new(std::io::stdout()),
      top_left: XY::default(),
      size: Size::default(),
    }
  }
}

impl DrawSurface for StdOutDrawSurface {
  fn size(&self) -> Size {
    self.size
  }

  fn write<'a>(&mut self, at: XY, data: &'a str) -> Result<(), &'a str> {
    let lines = data.lines().collect::<Vec<_>>();

    if at.y + lines.len() as u16 > self.size.height {
      return Err(lines[self.size.height as usize]);
    }

    if let Some(pos) = lines
      .iter()
      .position(|line| line.len() as u16 > at.x + self.size.width)
    {
      return Err(lines[pos]);
    }

    if at.x + data.len() as u16 > self.size.width {
      return Err(&data[self.size.width as usize..]);
    }

    let start_at = self.top_left + at;

    self.stdout.queue(MoveTo(start_at.x, start_at.y)).unwrap();

    for line in lines {
      self
        .stdout
        .queue(Print(line))
        .unwrap()
        .queue(MoveToNextLine(1))
        .unwrap();
    }

    Ok(())
  }

  fn shrink(
    &mut self,
    top_left: XY,
    size: Size,
    f: Box<dyn FnOnce(&mut dyn DrawSurface) + '_>,
  ) {
    let original_top_left = self.top_left;
    let original_size = self.size;

    debug_assert!(
      top_left.x < original_size.width && top_left.y < original_size.height,
      "attempted to shrink with `top_left` position being out of \
       bounds\navailable size: {original_size:?}\ndesired top left: \
       {top_left:?}",
    );

    debug_assert!(
      original_top_left.x + top_left.x + size.width
        <= original_top_left.x + original_size.width
        && original_top_left.y + top_left.y + size.height
          <= original_top_left.y + original_size.height,
      "attempted to shrink with a `size` larger than the available \
       space\navailable size: {original_size:?}\ndesired top left: \
       {top_left:?}\ndesired size: {size:?}\n",
    );

    self.top_left = self.top_left + top_left;
    self.size = size;

    f(self);

    self.top_left = original_top_left;
    self.size = original_size;
  }
}

#[track_caller]
pub fn run<V: IntoView>(f: impl FnOnce(Scope) -> V + 'static) {
  // Update the panic hook to make sure we leave the terminal in a
  // usable state on panic
  std::panic::update_hook(|prev, info| {
    cleanup_screen();

    prev(info)
  });

  let runtime = leptos_reactive::create_runtime();

  setup_screen();

  let disposer = leptos_reactive::create_scope(
    runtime,
    #[track_caller]
    move |cx| {
      let mut view = f(cx).into_view(cx);

      let mut surface = StdOutDrawSurface::default();

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
    .queue(LeaveAlternateScreen)
    .unwrap()
    .queue(DisableMouseCapture)
    .unwrap()
    .flush();
}

#[track_caller]
fn render_view(surface: &mut StdOutDrawSurface, view: &mut View) {
  let limits = Limits::from_stdout();

  surface.size = limits.max_size();
  surface.top_left = XY::default();

  let child_size = view.layout(limits);

  debug_assert_size_within_limits(
    limits,
    child_size,
    std::panic::Location::caller(),
  );

  surface
    .stdout
    .queue(BeginSynchronizedUpdate)
    .unwrap()
    .queue(Clear(ClearType::All))
    .unwrap();

  view.draw(surface);

  surface.stdout.flush().unwrap();
}
