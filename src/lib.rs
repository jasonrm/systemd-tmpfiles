use file_mode::Mode;
use std::path::PathBuf;

#[derive(Debug)]
struct Entry<'a> {
    temp_type: &'a str,
    path: &'a str,
    mode: &'a str,
    user: &'a str,
    group: &'a str,
    age: &'a str,
    argument: &'a str,
}

enum State {
    TempType,
    Path,
    Mode,
    User,
    Group,
    Age,
    Argument,
}

fn parse_line<'a>(line: &'a str) -> Entry<'a> {
    let mut state = State::TempType;
    let mut temp_type = "";
    let mut path = "";
    let mut mode = "";
    let mut user = "";
    let mut group = "";
    let mut age = "";
    let mut argument = "";

    let mut start_index = 0;
    let mut inside_quotes = false;

    for (i, c) in line.chars().enumerate() {
        match state {
            State::TempType => {
                if c == ' ' {
                    state = State::Path;
                } else {
                    temp_type = &line[..i + 1];
                }
            }
            State::Path => {
                if c == ' ' || c == '\'' {
                    if (c == '\'' && inside_quotes) || (start_index > 0 && !inside_quotes) {
                        state = State::Mode;
                        path = &line[start_index..i];
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
                        mode = &line[start_index..i];
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
                        user = &line[start_index..i];
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
                        group = &line[start_index..i];
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
                        age = &line[start_index..i];
                        start_index = i;
                    }
                } else {
                    if start_index == 0 {
                        start_index = i;
                    }
                }
            }
            State::Argument => {
                if c != ' ' {
                    argument = &line[i..];
                    break;
                }
            }
        }
    }

    Entry {
        temp_type,
        path,
        mode,
        user,
        group,
        age,
        argument,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = "d         /run/screens        1777      root      screen    10d       -";
        let parsed = parse_line(data);
        assert_eq!(parsed.temp_type, "d");
        assert_eq!(parsed.path, "/run/screens");
        assert_eq!(parsed.mode, "1777");
        assert_eq!(parsed.user, "root");
        assert_eq!(parsed.group, "screen");
        assert_eq!(parsed.age, "10d");
        assert_eq!(parsed.argument, "-");

        let data = "d         '/run/uscreens'     0755      root      screen    10d12h    -";
        let parsed = parse_line(data);
        assert_eq!(parsed.temp_type, "d");
        assert_eq!(parsed.path, "/run/uscreens");
        assert_eq!(parsed.mode, "0755");
        assert_eq!(parsed.user, "root");
        assert_eq!(parsed.group, "screen");
        assert_eq!(parsed.age, "10d12h");
        assert_eq!(parsed.argument, "-");

        let data = "d         '/a z/some test'    0755      root      screen    10d12h    -";
        let parsed = parse_line(data);
        assert_eq!(parsed.temp_type, "d");
        assert_eq!(parsed.path, "/a z/some test");
        assert_eq!(parsed.mode, "0755");
        assert_eq!(parsed.user, "root");
        assert_eq!(parsed.group, "screen");
        assert_eq!(parsed.age, "10d12h");
        assert_eq!(parsed.argument, "-");

        let data =
            "t /run/cups - - - - security.SMACK64=printing user.attr-with-spaces=\"foo bar\"";
        let parsed = parse_line(data);
        assert_eq!(parsed.temp_type, "t");
        assert_eq!(parsed.path, "/run/cups");
        assert_eq!(parsed.mode, "-");
        assert_eq!(parsed.user, "-");
        assert_eq!(parsed.group, "-");
        assert_eq!(parsed.age, "-");
        assert_eq!(
            parsed.argument,
            "security.SMACK64=printing user.attr-with-spaces=\"foo bar\""
        );
    }
}
