use std::str::FromStr;
use chrono::prelude::*;

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
            },
        };

        let from = match parts.next() {
            Some(a) => {
                let f = match parse_date(a.to_owned()) {
                    Some(b) => b,
                    None => {
                        return Err(TimeWarriorLineError::NoDate());
                    },
                };
                (f)
            },
            _ => {
                return Err(TimeWarriorLineError::NoDate());
            },
        };

        let mut active = false;
        let until: DateTime<Utc> = match parts.next() {
            // no end date but tags
            Some("#") => {
                active = true;
                Utc::now()
            },
            // end date set
            Some("-") => {
                let utc = match parts.next() {
                    Some(u) => {
                        let stuff = parts.next();
                        match stuff {
                            Some("#") => (),
                            None => (),
                            _ => {
                                return Err(TimeWarriorLineError::Generic(format!("Unexpected {:?}", stuff).to_owned()));
                            }
                        }
                        let f = match parse_date(u.to_owned()) {
                            Some(a) => a,
                            None => {
                                return Err(TimeWarriorLineError::Generic(format!("Unexpected {:?}", u).to_owned()));
                            },
                        };
                        (f)
                    },
                    None => {
                        return Err(TimeWarriorLineError::Generic("nope".to_owned()));
                    }
                };
                (utc)
            },
            // no enddate and no tags
            _ => {
                active = true;
                Utc::now()
            },
        };

        let mut tags = Vec::<String>::new();
        for tag in parts {
            tags.push(tag.to_owned());
        }

        Ok(TimeWarriorLine{
            tw_type: tw_type,
            from: from,
            until: until,
            tags: tags,
            active: active
        })
    }
}

fn parse_date(date_string: String) -> Option<DateTime<Utc>> {
    let from_part = format!("{} +0000", date_string);
    
    let date = match DateTime::parse_from_str(&from_part, "%Y%m%dT%H%M%SZ %z") {
        Ok(a) => Utc.from_local_datetime(&a.naive_local()).single(),
        Err(_) => None
    };
    (date)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_only_started_no_tags() {
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
    fn test_only_start_and_enddate_no_tags() {
        let result = TimeWarriorLine::from_str("inc 20001011T133055Z - 20001112T144054Z");
        assert_eq!(result.is_ok(), true, "parsed line is not a ok result {:?}", result);
        
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

}