use file_mode::Mode;
use std::path::PathBuf;

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
    path: PathBuf,
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
    pub fn line_type(&self) -> &LineType {
        &self.line_type
    }

    pub fn modifiers(&self) -> &Vec<Modifier> {
        &self.modifiers
    }
    pub fn path(&self) -> &PathBuf {
        &self.path
    }
    pub fn mode(&self) -> &Option<Mode> {
        &self.mode
    }
    pub fn user(&self) -> &Option<String> {
        &self.user
    }
    pub fn group(&self) -> &Option<String> {
        &self.group
    }
    pub fn age(&self) -> &Option<String> {
        &self.age
    }
    pub fn argument(&self) -> &Option<String> {
        &self.argument
    }
}

fn parse_line(line: &str) -> Entry {
    let mut state = State::LineType;

    let mut entry = Entry {
        line_type: LineType::Unsupported('-'),
        modifiers: Vec::new(),
        path: PathBuf::new(),
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
                    _ => LineType::Unsupported(c),
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
                        entry.path.push(&line[start_index..i]);
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
                        if c == '-' {
                            state = State::User;
                        } else {
                            start_index = i;
                        }
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
                        if c == '-' {
                            state = State::Group;
                        } else {
                            start_index = i;
                        }
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
                        if c == '-' {
                            state = State::Age;
                        } else {
                            start_index = i;
                        }
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
                        if c == '-' {
                            state = State::Argument;
                        } else {
                            start_index = i;
                        }
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

    // Hack to handle the only a line type and path
    // TODO: rewrite parsing
    if state == State::Path {
        entry.path.push(&line[start_index..]);
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
        assert_eq!(parsed.path, PathBuf::from("/run/screens"));
        assert_eq!(parsed.mode, Some(Mode::from(0o1777)));
        assert_eq!(parsed.user, Some("root".to_owned()));
        assert_eq!(parsed.group, Some("screen".to_owned()));
        assert_eq!(parsed.age, Some("10d".to_owned()));
        assert_eq!(parsed.argument, None);

        let data = "d         '/run/uscreens'     0755      root      screen    10d12h    -";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::DirectoryCreateAndClean);
        assert_eq!(parsed.path, PathBuf::from("/run/uscreens"));
        assert_eq!(parsed.mode, Some(Mode::from(0o0755)));
        assert_eq!(parsed.user, Some("root".to_owned()));
        assert_eq!(parsed.group, Some("screen".to_owned()));
        assert_eq!(parsed.age, Some("10d12h".to_owned()));
        assert_eq!(parsed.argument, None);

        let data = "d         '/a z/some test'    0755      root      screen    10d12h    -";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::DirectoryCreateAndClean);
        assert_eq!(parsed.path, PathBuf::from("/a z/some test"));
        assert_eq!(parsed.mode, Some(Mode::from(0o0755)));
        assert_eq!(parsed.user, Some("root".to_owned()));
        assert_eq!(parsed.group, Some("screen".to_owned()));
        assert_eq!(parsed.age, Some("10d12h".to_owned()));
        assert_eq!(parsed.argument, None);

        let data =
            "t /run/cups - - - - security.SMACK64=printing user.attr-with-spaces=\"foo bar\"";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::Unsupported('t'));
        assert_eq!(parsed.path, PathBuf::from("/run/cups"));
        assert_eq!(parsed.mode, None);
        assert_eq!(parsed.user, None);
        assert_eq!(parsed.group, None);
        assert_eq!(parsed.age, None);
        assert_eq!(
            parsed.argument.unwrap(),
            "security.SMACK64=printing user.attr-with-spaces=\"foo bar\""
        );
        let data = "r! /var/cache/dnf/*/*/download_lock.pid";
        let parsed = parse_line(data);
        assert_eq!(parsed.line_type, LineType::Unsupported('r'));
        assert_eq!(parsed.modifiers, vec![Modifier::OnlySafeDuringBoot]);
        assert_eq!(
            parsed.path,
            PathBuf::from("/var/cache/dnf/*/*/download_lock.pid")
        );
        assert_eq!(parsed.mode, None);
        assert_eq!(parsed.user, None);
        assert_eq!(parsed.group, None);
        assert_eq!(parsed.age, None);
        assert_eq!(parsed.argument, None);
    }
}
