use crate::{
  Size,
  XY,
};
use crossterm::{
  cursor::{
    MoveTo,
    MoveToNextLine,
  },
  style::{
    Attribute,
    Color,
    Print,
    ResetColor,
    SetBackgroundColor,
    SetForegroundColor,
    Stylize,
  },
  QueueableCommand,
};
use std::{
  borrow::Cow,
  io::{
    BufWriter,
    Stdout,
    Write,
  },
};

pub(crate) type StdoutDrawSurface = BufDrawSurface<Stdout>;

pub trait DrawSurface {
  fn size(&self) -> Size;

  /// Writes the given data starting at the given coordinates.
  /// Returns [`Err`] with the data that was written out of bounds,
  /// if any.
  ///
  /// Note:
  /// The coordinates are relative to the provided [`Limits`]
  /// and are not absolute.
  fn write(&mut self, at: XY, data: &str) {
    self.write_styled(at, data, None, None, None)
  }

  fn write_styled(
    &mut self,
    at: XY,
    data: &str,
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    attribute: Option<Attribute>,
  );

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

#[derive(Debug)]
pub(crate) struct BufDrawSurface<W: Write> {
  pub(crate) buf: BufWriter<W>,
  pub(crate) top_left: XY,
  pub(crate) size: Size,
}

impl Default for BufDrawSurface<Stdout> {
  fn default() -> Self {
    Self {
      buf: BufWriter::new(std::io::stdout()),
      top_left: XY::default(),
      size: Size::default(),
    }
  }
}

impl<W: Write> DrawSurface for BufDrawSurface<W> {
  fn size(&self) -> Size {
    self.size
  }

  fn write_styled(
    &mut self,
    at: XY,
    mut data: &str,
    foreground_color: Option<Color>,
    background_color: Option<Color>,
    attribute: Option<Attribute>,
  ) {
    if data.is_empty() {
      return;
    }

    if at.y > self.size.height {
      return;
    }

    if at.x > self.size.width {
      return;
    }

    let mut data = Cow::Borrowed(
      &data.lines().next().unwrap()
        [0..((self.size.width - at.x) as usize).min(data.len())],
    );

    if let Some(color) = foreground_color {
      data = Cow::Owned(format!("{}{data}", SetForegroundColor(color)));
    }

    if let Some(color) = background_color {
      data = Cow::Owned(format!("{}{data}", SetBackgroundColor(color)));
    }

    if let Some(attr) = attribute {
      data = Cow::Owned(data.attribute(attr).to_string());
    }

    if foreground_color.is_some() || background_color.is_some() {
      data = Cow::Owned(format!("{data}{ResetColor}"));
    }

    self
      .buf
      .queue(MoveTo(at.x, at.y))
      .unwrap()
      .queue(Print(data))
      .unwrap();
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

impl<W: Write> BufDrawSurface<W> {
  pub fn new(writer: W, size: impl Into<Size>) -> Self {
    Self {
      buf: BufWriter::new(writer),
      top_left: XY::default(),
      size: size.into(),
    }
  }

  pub fn resize(&mut self, new_size: Size) {
    self.size = new_size;
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn can_write() {
    let mut buf = vec![];
    let data = "hello";
    let expected_data = format!("{}hello", MoveTo(0, 0));

    let mut surface = BufDrawSurface::new(&mut buf, (5, 1));

    surface.write((0, 0).into(), &data);

    drop(surface);

    let buf = std::str::from_utf8(&buf).unwrap();

    assert_eq!(buf, expected_data);
  }

  #[test]
  fn write_out_of_bounds_clips() {
    let mut buf = vec![];
    let data = "hello";
    let expected_data = format!("{}he", MoveTo(0, 0));

    let mut surface = BufDrawSurface::new(&mut buf, (2, 1));

    surface.write((0, 0).into(), &data);

    drop(surface);

    let buf = std::str::from_utf8(&buf).unwrap();

    assert_eq!(buf, expected_data);
  }

  #[test]
  fn only_first_line_writes() {
    let mut buf = vec![];
    let data = "hello\nthere";
    let expected_data = format!("{}hello", MoveTo(0, 0));

    let mut surface = BufDrawSurface::new(&mut buf, (5, 2));

    surface.write((0, 0).into(), &data);

    drop(surface);

    let buf = std::str::from_utf8(&buf).unwrap();

    assert_eq!(buf, expected_data);
  }
}
