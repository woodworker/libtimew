use chrono::prelude::*;
use std::str::FromStr;

#[derive(Debug)]
pub struct TimeWarriorLine {
    tw_type: String,
    from: DateTime<Utc>,
    until: DateTime<Utc>,
    tags: Vec<String>,
    active: bool,
}

impl TimeWarriorLine {
    pub fn duration(&self) -> chrono::Duration {
        self.until - self.from
    }

    pub fn full_tag(&self) -> String {
        self.tags.join(" ")
    }

    pub fn get_day(&self) -> Date<Utc> {
        self.from.date()
    }
}

#[derive(Debug)]
pub enum TimeWarriorLineError {
    Generic(String),
    NoDate(),
}

impl FromStr for TimeWarriorLine {
    type Err = TimeWarriorLineError;

    // Parses a timewarrior line
    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut parts = line.split_whitespace();

        let tw_type = match parts.next() {
            Some(a) => a.to_owned(),
            _ => {
                return Err(TimeWarriorLineError::Generic("Type parsing".to_owned()));
            }
        };

        let from = match parts.next() {
            Some(a) => {
                let f = match parse_date(a.to_owned()) {
                    Some(b) => b,
                    None => {
                        return Err(TimeWarriorLineError::NoDate());
                    }
                };
                f
            }
            _ => {
                return Err(TimeWarriorLineError::NoDate());
            }
        };

        let mut active = false;
        let until: DateTime<Utc> = match parts.next() {
            // no end date but tags
            Some("#") => {
                active = true;
                Utc::now()
            }
            // end date set
            Some("-") => {
                let utc = match parts.next() {
                    Some(u) => {
                        let stuff = parts.next();
                        match stuff {
                            Some("#") => (),
                            None => (),
                            _ => {
                                return Err(TimeWarriorLineError::Generic(
                                    format!("Unexpected {:?}", stuff).to_owned(),
                                ));
                            }
                        }
                        let f = match parse_date(u.to_owned()) {
                            Some(a) => a,
                            None => {
                                return Err(TimeWarriorLineError::Generic(
                                    format!("Unexpected {:?}", u).to_owned(),
                                ));
                            }
                        };
                        f
                    }
                    None => {
                        return Err(TimeWarriorLineError::Generic("nope".to_owned()));
                    }
                };
                utc
            }
            // no enddate and no tags
            None => {
                active = true;
                Utc::now()
            }
            // everything else is an error
            e => {
                return Err(TimeWarriorLineError::Generic(
                    format!("Unexpected {:?}", e).to_owned(),
                ));
            }
        };

        let str_nums: Vec<String> = parts.map(|n| n.to_string()).collect();

        let tagline = str_nums.join(" ");

        let mut multitag = false;
        let mut tag_string = "".to_owned();
        let mut tags = Vec::<String>::new();
        for one_char in tagline.chars() {
            match one_char {
                '"' => {
                    multitag = !multitag;
                }
                ' ' => {
                    if multitag {
                        tag_string.push(' ');
                    } else {
                        tags.push(tag_string);
                        tag_string = "".to_owned();
                    }
                }
                c => {
                    tag_string.push(c);
                }
            }
        }
        if tag_string != "" {
            tags.push(tag_string);
        }

        Ok(TimeWarriorLine {
            tw_type: tw_type,
            from: from,
            until: until,
            tags: tags,
            active: active,
        })
    }
}

fn parse_date(date_string: String) -> Option<DateTime<Utc>> {
    let from_part = format!("{} +0000", date_string);

    let date = match DateTime::parse_from_str(&from_part, "%Y%m%dT%H%M%SZ %z") {
        Ok(a) => Utc.from_local_datetime(&a.naive_local()).single(),
        Err(_) => None,
    };
    date
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn garbage_in_err_out() {
        let result = TimeWarriorLine::from_str("afdf dafdf dsfads fdsaf");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_z_is_valid_timezone_definition() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055CEST");
        assert_eq!(
            result.is_err(),
            true,
            "CEST line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_broken_lines_1() {
        let result = TimeWarriorLine::from_str("inc");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_broken_lines_2() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - sdsadsad");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_broken_lines_3() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z sadasds");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_broken_lines_4() {
        let result =
            TimeWarriorLine::from_str("inc 20001011T133055Z - 20001011T183055Z dsafsadsads");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_broken_lines_5() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - ");
        assert_eq!(
            result.is_err(),
            true,
            "line should not be parsed as ok result"
        );
    }

    #[test]
    fn only_started_no_tags() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z");
        assert_eq!(result.is_ok(), true, "parsed line is not a ok result");

        let line = result.unwrap();

        assert_eq!(line.tw_type, "inc");
        assert_eq!(line.active, true);
        assert_eq!(line.tags, Vec::<String>::new());

        assert_eq!(line.full_tag(), "".to_owned());

        assert_eq!(line.from.format("%Y-%m-%d").to_string(), "2000-10-11");
        assert_eq!(line.from.format("%H:%M:%S").to_string(), "13:30:55");
    }

    #[test]
    fn only_started_one_tag() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z # Walala");
        assert_eq!(result.is_ok(), true, "parsed line is not a ok result");

        let line = result.unwrap();

        assert_eq!(line.tw_type, "inc");
        assert_eq!(line.active, true);
        assert_eq!(line.tags, vec!["Walala"]);

        assert_eq!(line.full_tag(), "Walala".to_owned());

        assert_eq!(line.from.format("%Y-%m-%d").to_string(), "2000-10-11");
        assert_eq!(line.from.format("%H:%M:%S").to_string(), "13:30:55");
    }

    #[test]
    fn only_start_and_enddate_no_tags() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - 20001112T144054Z");
        assert_eq!(
            result.is_ok(),
            true,
            "parsed line is not a ok result {:?}",
            result
        );

        let line = result.unwrap();

        assert_eq!(line.tw_type, "inc");
        assert_eq!(line.active, false);
        assert_eq!(line.tags, Vec::<String>::new());

        assert_eq!(line.full_tag(), "".to_owned());

        assert_eq!(line.from.format("%Y-%m-%d").to_string(), "2000-10-11");
        assert_eq!(line.from.format("%H:%M:%S").to_string(), "13:30:55");

        assert_eq!(line.until.format("%Y-%m-%d").to_string(), "2000-11-12");
        assert_eq!(line.until.format("%H:%M:%S").to_string(), "14:40:54");
    }

    #[test]
    fn duration_is_correct() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - 20001011T134055Z");
        assert_eq!(
            result.is_ok(),
            true,
            "parsed line is not a ok result {:?}",
            result
        );

        let line = result.unwrap();

        assert_eq!(line.duration(), chrono::Duration::minutes(10));
    }

    #[test]
    fn date_is_correct() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - 20001011T134055Z");
        assert_eq!(
            result.is_ok(),
            true,
            "parsed line is not a ok result {:?}",
            result
        );

        let line = result.unwrap();

        assert_eq!(line.get_day(), chrono::Utc.ymd(2000, 10, 11));
    }

    #[test]
    fn only_start_and_enddate_one_taga() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - 20001112T144054Z # Buvere");
        assert_eq!(
            result.is_ok(),
            true,
            "parsed line is not a ok result {:?}",
            result
        );

        let line = result.unwrap();

        assert_eq!(line.tw_type, "inc");
        assert_eq!(line.active, false);
        assert_eq!(line.tags, vec!["Buvere"]);

        assert_eq!(line.full_tag(), "Buvere".to_owned());

        assert_eq!(line.from.format("%Y-%m-%d").to_string(), "2000-10-11");
        assert_eq!(line.from.format("%H:%M:%S").to_string(), "13:30:55");

        assert_eq!(line.until.format("%Y-%m-%d").to_string(), "2000-11-12");
        assert_eq!(line.until.format("%H:%M:%S").to_string(), "14:40:54");
    }

    #[test]
    fn tags_with_spaces_are_recognized() {
        let result = TimeWarriorLine::from_str(
            "inc 20001011T133055Z - 20001112T144054Z # \"ABC CDE\" EFG HIJ",
        );
        assert_eq!(
            result.is_ok(),
            true,
            "parsed line is not a ok result {:?}",
            result
        );

        let line = result.unwrap();

        assert_eq!(line.tw_type, "inc");
        assert_eq!(line.active, false);
        assert_eq!(line.tags, vec!["ABC CDE", "EFG", "HIJ"]);

        // assert_eq!(line.full_tag(), "\"ABC CDE\" EFG HIJ".to_owned());

        assert_eq!(line.tags.len(), 3);
    }
}
