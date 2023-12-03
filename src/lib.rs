use crate::LineType::Unsupported;
use file_mode::Mode;

#[derive(Debug, PartialEq)]
pub enum LineType {
    DirectoryCreateAndClean,
    DirectoryCreateAndRemove,
    Unsupported(char),
}

#[derive(Debug, PartialEq)]
pub enum Modifier {
    OnlySafeDuringBoot,
    IgnoreCreateError,
    RemoveExisting,
    Base64EncodedArgument,
    Unsupported(char),
}

#[derive(Debug)]
pub struct Entry {
    line_type: LineType,
    modifiers: Vec<Modifier>,
    path: String,
    mode: Option<Mode>,
    user: Option<String>,
    group: Option<String>,
    age: Option<String>,
    argument: Option<String>,
}

#[derive(Debug, PartialEq)]
enum State {
    LineType,
    Modifiers,
    Path,
    Mode,
    User,
    Group,
    Age,
    Argument,
}

impl Entry {
    pub fn from_str(string: &str) -> Self {
        parse_line(string)
    }
}

fn parse_line(line: &str) -> Entry {
    let mut state = State::LineType;

    let mut entry = Entry {
        line_type: Unsupported('-'),
        modifiers: Vec::new(),
        path: String::new(),
        mode: None,
        user: None,
        group: None,
        age: None,
        argument: None,
    };

    let mut start_index = 0;
    let mut inside_quotes = false;

    for (i, c) in line.chars().enumerate() {
        match state {
            State::LineType => {
                entry.line_type = match c {
                    'd' => LineType::DirectoryCreateAndClean,
                    'D' => LineType::DirectoryCreateAndRemove,
                    _ => Unsupported(c),
                };
                state = State::Modifiers;
            }
            State::Modifiers => {
                if c == ' ' {
                    state = State::Path;
                } else {
                    entry.modifiers.push(match c {
                        '!' => Modifier::OnlySafeDuringBoot,
                        '-' => Modifier::IgnoreCreateError,
                        '=' => Modifier::RemoveExisting,
                        '^' => Modifier::Base64EncodedArgument,
                        _ => Modifier::Unsupported(c),
                    });
                }
            }
            State::Path => {
                if c == ' ' || c == '\'' {
                    if (c == '\'' && inside_quotes) || (start_index > 0 && !inside_quotes) {
                        state = State::Mode;
                        entry.path.push_str(&line[start_index..i]);
                        start_index = 0;
                        inside_quotes = false;
                    } else if c == '\'' && !inside_quotes {
                        start_index = i + 1;
                        inside_quotes = true;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::Mode => {
                if c == ' ' {
                    if start_index > 0 {
                        state = State::User;
                        let octal_string = &line[start_index..i];
                        if let Ok(octal) = u32::from_str_radix(octal_string, 8) {
                            entry.mode = Some(Mode::from(octal))
                        }
                        start_index = 0;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::User => {
                if c == ' ' {
                    if start_index > 0 {
                        state = State::Group;
                        entry.user = Some(line[start_index..i].to_string());
                        start_index = 0;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::Group => {
                if c == ' ' {
                    if start_index > 0 {
                        state = State::Age;
                        entry.group = Some(line[start_index..i].to_string());
                        start_index = 0;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::Age => {
                if c == ' ' {
                    if start_index > 0 {
                        state = State::Argument;
                        entry.age = Some(line[start_index..i].to_string());
                        start_index = i;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::Argument => {
                if c == '-' {
                    break;
                } else if c != ' ' {
                    entry.argument = Some(line[i..].to_string());
                    break;
                }
            }
        }
    }

    entry
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = "d         /run/screens        1777      root      screen    10d       -";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::DirectoryCreateAndClean);
        assert_eq!(parsed.path, "/run/screens");
        assert_eq!(parsed.mode, Some(Mode::from(0o1777)));
        assert_eq!(parsed.user, Some("root".to_owned()));
        assert_eq!(parsed.group, Some("screen".to_owned()));
        assert_eq!(parsed.age, Some("10d".to_owned()));
        assert_eq!(parsed.argument, None);

        let data = "d         '/run/uscreens'     0755      root      screen    10d12h    -";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::DirectoryCreateAndClean);
        assert_eq!(parsed.path, "/run/uscreens");
        assert_eq!(parsed.mode, Some(Mode::from(0o0755)));
        assert_eq!(parsed.user, Some("root".to_owned()));
        assert_eq!(parsed.group, Some("screen".to_owned()));
        assert_eq!(parsed.age, Some("10d12h".to_owned()));
        assert_eq!(parsed.argument, None);

        // let data = "d         '/a z/some test'    0755      root      screen    10d12h    -";
        // let parsed = parse_line(data);
        // assert_eq!(parsed.line_type, "d");
        // assert_eq!(parsed.path, "/a z/some test");
        // assert_eq!(parsed.mode, Mode::from(0o0755));
        // assert_eq!(parsed.user, "root");
        // assert_eq!(parsed.group, "screen");
        // assert_eq!(parsed.age, "10d12h");
        // assert_eq!(parsed.argument, "-");
        //
        // let data =
        //     "t /run/cups - - - - security.SMACK64=printing user.attr-with-spaces=\"foo bar\"";
        // let parsed = parse_line(data);
        // assert_eq!(parsed.line_type, "t");
        // assert_eq!(parsed.path, "/run/cups");
        // assert_eq!(parsed.mode, None);
        // assert_eq!(parsed.user, "-");
        // assert_eq!(parsed.group, "-");
        // assert_eq!(parsed.age, "-");
        // assert_eq!(
        //     parsed.argument.unwrap(),
        //     "security.SMACK64=printing user.attr-with-spaces=\"foo bar\""
        // );
    }
}
