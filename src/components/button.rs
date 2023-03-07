use crate::{
  IntoView,
  Size,
  View,
  Widget,
  XY,
};
use crossterm::style::{
  Attribute,
  Color,
};
use std::{
  borrow::Cow,
  sync::{
    Arc,
    Mutex,
  },
};

#[derive(Debug)]
pub struct Button {
  text: Cow<'static, str>,
  formatted_text: String,
  focused: bool,
}

impl Widget for Button {
  fn name(&self) -> Cow<'static, str> {
    "Button".into()
  }

  fn layout(&mut self, limits: crate::Limits) -> crate::Size {
    let wrapped =
      textwrap::wrap(&self.text, limits.max_width as usize).remove(0);

    let height = if limits.max_height == 0 {
      0
    } else if limits.min_height > 1 {
      limits.min_height
    } else {
      1
    };

    match limits.max_width {
      0 => self.formatted_text = "".into(),
      1 => self.formatted_text = "<".into(),
      2 => self.formatted_text = "<>".into(),
      width => {
        let width = width - 2;

        self.formatted_text =
          format!("<{}>", &wrapped[0..wrapped.len().min(width as usize)]);
      }
    }

    let width = if limits.min_width > self.formatted_text.len() as u16 {
      limits.min_width
    } else {
      self.formatted_text.len() as u16
    };

    Size { width, height }
  }

  fn draw(&self, surface: &mut dyn crate::DrawSurface) {
    let focused = self.focused;

    surface.write_styled(
      XY::default(),
      &self.formatted_text,
      None,
      if focused { Some(Color::Red) } else { None },
      if focused { Some(Attribute::Bold) } else { None },
    );
  }
}

impl IntoView for Button {
  fn into_view(self, _: leptos_reactive::Scope) -> crate::View {
    View::Widget(Arc::new(Mutex::new(self)))
  }
}

impl Button {
  pub fn new(text: impl Into<Cow<'static, str>>) -> Self {
    Self {
      text: text.into(),
      formatted_text: String::new(),
      focused: false,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::components::get_widget_output;
  use crossterm::{
    cursor::MoveTo,
    style::{
      Print,
      ResetColor,
      SetAttribute,
      SetBackgroundColor,
    },
  };

  #[test]
  fn renders() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (7, 1), (0, 0));

    assert_eq!(size, (7, 1).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<hello>")]);
  }

  #[test]
  fn focused_renders_background_color() {
    let (size, output) = get_widget_output(
      {
        let mut btn = Button::new("hello");

        btn.focused = true;

        btn
      },
      (7, 1),
      (0, 0),
    );

    assert_eq!(size, (7, 1).into());

    assert_eq!(
      output,
      commands![
        MoveTo(0, 0),
        SetAttribute(Attribute::Bold),
        SetBackgroundColor(Color::Red),
        Print("<hello>"),
        SetAttribute(Attribute::Reset),
        ResetColor,
      ]
    );
  }

  #[test]
  fn renders_no_characters() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (0, 1), (0, 0));

    assert_eq!(size, (0, 1).into());

    assert_eq!(output, "");
  }

  #[test]
  fn renders_single_character() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (1, 1), (0, 0));

    assert_eq!(size, (1, 1).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<")]);
  }

  #[test]
  fn renders_two_character() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (2, 1), (0, 0));

    assert_eq!(size, (2, 1).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<>")]);
  }

  #[test]
  fn renders_three_character() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (3, 1), (0, 0));

    assert_eq!(size, (3, 1).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<h>")]);
  }

  #[test]
  fn renders_all_character() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (7, 1), (0, 0));

    assert_eq!(size, (7, 1).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<hello>")]);
  }

  #[test]
  fn resizes_to_min_size() {
    let (size, output) =
      get_widget_output(Button::new("hello"), (10, 10), (9, 9));

    assert_eq!(size, (9, 9).into());

    assert_eq!(output, commands![MoveTo(0, 0), Print("<hello>")]);
  }
}
