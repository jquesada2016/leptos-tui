use super::CoreComponent;
use crate::{
  DrawSurface,
  IntoView,
  Limits,
  Size,
  Widget,
  XY,
};
use std::borrow::Cow;

#[derive(Debug, derive_more::Display)]
#[display(fmt = "{}", text)]
pub struct Text {
  text: Cow<'static, str>,
  wrapped_text: Vec<String>,
  size: Size,
}

impl IntoView for Text {
  fn into_view(self, cx: leptos_reactive::Scope) -> crate::View {
    CoreComponent::Text(self).into_view(cx)
  }
}

impl Widget for Text {
  fn name(&self) -> Cow<'static, str> {
    "Text".into()
  }

  fn layout(&mut self, limits: Limits) -> Size {
    let wrapped_text = textwrap::wrap(&self.text, limits.max_width as usize)
      .into_iter()
      .map(|line| line.into_owned())
      .collect();

    self.wrapped_text = wrapped_text;

    let width = self.wrapped_text.iter().fold(0, |acc, cur| {
      if cur.len() as u16 > acc {
        cur.len() as u16
      } else {
        acc
      }
    });

    let height = if self.wrapped_text.len() as u16 <= limits.max_height {
      if self.wrapped_text.len() as u16 >= limits.min_height {
        self.wrapped_text.len() as u16
      } else {
        limits.min_height
      }
    } else {
      limits.max_height
    };

    let size = Size { width, height };

    self.size = size;

    size
  }

  fn draw(&self, surface: &mut dyn DrawSurface) {
    let size = self.size;

    for (i, line) in self
      .wrapped_text
      .iter()
      .enumerate()
      .take(size.height as usize)
    {
      surface.write(XY { x: 0, y: i as u16 }, line);
    }
  }
}

impl Text {
  pub fn new<T: Into<Cow<'static, str>>>(text: T) -> Self {
    Self {
      text: text.into(),
      wrapped_text: vec![],
      size: Size::default(),
    }
  }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Alignment {
  #[default]
  Start,
  Center,
  End,
}

fn get_text_size(limits: Limits, text: &str) -> Size {
  let wrapped_text = textwrap::wrap(text, limits.max_width as usize);

  let height = wrapped_text.len();

  if height as u16 <= limits.max_height {
    Size {
      width: limits.max_width,
      height: height as u16,
    }
  } else {
    limits.max_size()
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  mod layout {
    use super::*;

    #[test]
    fn text_fits_on_one_line() {
      let size = get_text_size(
        Limits {
          min_width: 0,
          min_height: 0,
          max_width: 10,
          max_height: 10,
        },
        "Hello",
      );

      assert_eq!(
        size,
        Size {
          width: 10,
          height: 1
        }
      );
    }

    #[test]
    fn text_fits_on_two_lines() {
      let size = get_text_size(
        Limits {
          min_width: 0,
          min_height: 0,
          max_width: 10,
          max_height: 10,
        },
        "Hello there",
      );

      assert_eq!(
        size,
        Size {
          width: 10,
          height: 2
        }
      );
    }

    #[test]
    fn too_much_text_fits_on_one_line_within_limits() {
      let size = get_text_size(
        Limits {
          min_width: 0,
          min_height: 0,
          max_width: 10,
          max_height: 1,
        },
        "Hello there",
      );

      assert_eq!(
        size,
        Size {
          width: 10,
          height: 1,
        }
      );
    }

    #[test]
    fn text_with_new_lines_gets_right_size() {
      let size = get_text_size(
        Limits {
          min_width: 0,
          min_height: 0,
          max_width: 100,
          max_height: 10,
        },
        "Hello there!\nHow are you?",
      );

      assert_eq!(
        size,
        Size {
          width: 100,
          height: 2,
        }
      );
    }

    #[test]
    fn text_with_new_lines_gets_right_size_and_splits_on_whitespace() {
      let size = get_text_size(
        Limits {
          min_width: 0,
          min_height: 0,
          max_width: 10,
          max_height: 10,
        },
        "Hello there!\nHow are you?",
      );

      assert_eq!(
        size,
        Size {
          width: 10,
          height: 4,
        }
      );
    }
  }

  mod widget {
    use super::*;
    use crate::{
      components::get_widget_output,
      BufDrawSurface,
    };
    use crossterm::{
      cursor::MoveTo,
      style::Print,
    };

    #[test]
    fn renders() {
      let (size, output) =
        get_widget_output(Text::new("hello"), (5, 1), (0, 0));

      assert_eq!(size, (5, 1).into());

      assert_eq!(output, commands![MoveTo(0, 0), Print("hello")]);
    }

    #[test]
    fn aligns_left() {
      let (size, output) =
        get_widget_output(Text::new("hello   "), (8, 1), (0, 0));

      assert_eq!(size, (5, 1).into());

      assert_eq!(output, commands![MoveTo(0, 0), Print("hello")]);
    }

    #[test]
    fn clipps() {
      let (size, output) =
        get_widget_output(Text::new("hello   "), (1, 1), (0, 0));

      assert_eq!(size, (1, 1).into());

      assert_eq!(output, commands![MoveTo(0, 0), Print("h")]);
    }

    #[test]
    fn renders_multiple_lines() {
      let (size, output) =
        get_widget_output(Text::new("hello\nthere"), (5, 2), (0, 0));

      assert_eq!(size, (5, 2).into());

      assert_eq!(
        output,
        commands!(MoveTo(0, 0), Print("hello"), MoveTo(0, 1), Print("there"))
      );
    }
  }
}
