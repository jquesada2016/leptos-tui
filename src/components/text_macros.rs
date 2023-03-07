macro_rules! commands {
    (
      $($expr:expr),* $(,)?
    ) => { {
      let mut buf = vec![];

      crossterm::execute!(
        buf,
        $($expr),*
      ).unwrap();

      String::from_utf8(buf).unwrap()
    }
    };
}
