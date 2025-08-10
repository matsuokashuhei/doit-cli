use chrono::{DateTime, Duration, Local, NaiveDate, NaiveDateTime, TimeZone, Timelike};
use clap::{ArgAction, ArgMatches, Command};
use regex::Regex;
use std::process::exit;

#[derive(Debug)]
pub struct Args {
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub interval: u64,
    pub verbose: bool,
}

impl Args {
    pub fn parse(matches: ArgMatches) -> Self {
        let start = matches
            .get_one::<DateTime<Local>>("start")
            .cloned()
            .unwrap();
        let end = matches
            .get_one::<DateTime<Local>>("end")
            .cloned()
            .unwrap_or_else(|| start + matches.get_one::<Duration>("duration").cloned().unwrap());

        if let Err(e) = end_after_start(&end, &start) {
            println!("{}", e);
            exit(1);
        }

        Args {
            start,
            end,
            interval: matches.get_one("interval").cloned().unwrap(),
            verbose: matches.get_one("verbose").cloned().unwrap(),
        }
    }
}

pub fn build_command() -> Command {
    Command::new("pmon")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Progress Bar Tool")
        .arg(
            clap::Arg::new("start")
                .short('s')
                .long("start")
                .value_parser(parse_start_time)
                .default_value(Local::now().format("%Y-%m-%d %H:%M:%S").to_string())
                .help("Start time"),
        )
        .arg(
            clap::Arg::new("end")
                .required(true)
                .short('e')
                .long("end")
                .value_parser(parse_end_time)
                .conflicts_with("duration")
                .help("End time"),
        )
        .arg(
            clap::Arg::new("duration")
                .required(true)
                .short('d')
                .long("duration")
                .value_parser(parse_duration)
                .conflicts_with("end")
                .help("Duration"),
        )
        .arg(
            clap::Arg::new("interval")
                .short('i')
                .long("interval")
                .value_parser(clap::value_parser!(u64).range(1..60))
                .default_value("5")
                .help("Update interval in seconds"),
        )
        .arg(
            clap::Arg::new("verbose")
                .short('v')
                .long("verbose")
                .action(ArgAction::SetTrue)
                .help("Display verbose output"),
        )
}

fn parse_start_time(s: &str) -> Result<DateTime<Local>, String> {
    if let Ok(datetime) = parse_datetime_as_ymd_hmsz(s) {
        return Ok(datetime);
    }
    if let Ok(datetime) = parse_datetime_as_ymd_hms(s) {
        return Ok(datetime);
    }
    if let Ok(datetime) = parse_datetime_as_ymd_hm(s) {
        return Ok(datetime);
    }
    if let Ok(date) = parse_date(s) {
        let datetime = date.and_hms_opt(0, 0, 0).unwrap();
        return Ok(TimeZone::from_utc_datetime(&Local, &datetime));
    }
    Err(format!("Invalid start time format: {}", s))
}

fn parse_end_time(s: &str) -> Result<DateTime<Local>, String> {
    if let Ok(datetime) = parse_datetime_as_ymd_hmsz(s) {
        return Ok(datetime);
    }
    if let Ok(datetime) = parse_datetime_as_ymd_hms(s) {
        return Ok(datetime);
    }
    if let Ok(datetime) = parse_datetime_as_ymd_hm(s) {
        return Ok(datetime.with_second(59).unwrap());
    }
    if let Ok(date) = parse_date(s) {
        let datetime = date.and_hms_opt(23, 59, 59).unwrap();
        return Ok(TimeZone::from_utc_datetime(&Local, &datetime));
    }
    Err(format!("Invalid end time format: {}", s))
}

fn end_after_start(end: &DateTime<Local>, start: &DateTime<Local>) -> Result<(), String> {
    if end < start {
        return Err(format!(
            "End time {} must be after start time {}",
            end.format("%Y-%m-%d %H:%M:%S"),
            start.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    Ok(())
}

#[warn(non_snake_case)]
fn parse_datetime_as_ymd_hmsz(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%d %H:%M:%S%z"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(TimeZone::from_utc_datetime(&Local, &datetime));
        }
    }
    Err(format!("Invalid datetime format: {}", s))
}

fn parse_datetime_as_ymd_hms(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y%m%d%H%M%S"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(TimeZone::from_utc_datetime(&Local, &datetime));
        }
    }
    Err(format!("Invalid datetime format: {}", s))
}

fn parse_datetime_as_ymd_hm(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M", "%Y%m%d%H%M"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(TimeZone::from_utc_datetime(&Local, &datetime));
        }
    }
    Err(format!("Invalid datetime format: {}", s))
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    let date_formats = ["%Y-%m-%d", "%Y%m%d"];
    for format in &date_formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(s, format) {
            return Ok(date);
        }
    }
    Err(format!("Invalid date format: {}", s))
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let re = Regex::new(r"^(\d+)([smhd])$").unwrap();
    if let Some(caps) = re.captures(s) {
        let value = caps[1]
            .parse::<i64>()
            .map_err(|_| format!("Invalid duration value: {}", s))?;
        let unit = &caps[2];
        match unit {
            "s" => return Ok(Duration::seconds(value)),
            "m" => return Ok(Duration::minutes(value)),
            "h" => return Ok(Duration::hours(value)),
            "d" => return Ok(Duration::days(value)),
            _ => {}
        }
    }
    Err(format!("Invalid duration format: {}", s))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_start() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.start.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 10:20:30"
        );
    }

    #[test]
    fn test_parse_without_start() {
        let now = Local::now().with_nanosecond(0).unwrap();
        let end = (now + Duration::days(30))
            .format("%Y-%m-%d %H:%M:%S")
            .to_string();
        let args = vec!["pmon", "--end", &end];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.start, now);
    }

    #[test]
    fn test_parse_with_end() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-31 23:59:59"
        );
    }

    #[test]
    fn test_parse_with_duration_seconds() {
        let args = vec!["pmon", "--start", "2025-01-01 10:20:30", "--duration", "1s"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 10:20:31"
        );
    }

    #[test]
    fn test_parse_with_duration_minutes() {
        let args = vec!["pmon", "--start", "2025-01-01 10:20:30", "--duration", "1m"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 10:21:30"
        );
    }

    #[test]
    fn test_parse_with_duration_hours() {
        let args = vec!["pmon", "--start", "2025-01-01 10:20:30", "--duration", "1h"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 11:20:30"
        );
    }

    #[test]
    fn test_parse_with_duration_days() {
        let args = vec!["pmon", "--start", "2025-01-01 10:20:30", "--duration", "1d"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-02 10:20:30"
        );
    }

    #[test]
    fn test_parse_with_interval() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--interval",
            "10",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.interval, 10);
    }

    #[test]
    fn test_parse_without_interval() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.interval, 5);
    }

    #[test]
    fn test_parse_with_verbose() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--verbose",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.verbose, true);
    }

    #[test]
    fn test_parse_without_verbose() {
        let args = vec![
            "pmon",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.verbose, false);
    }

    #[test]
    fn test_parse_start_time() {
        let test_cases = vec![
            (
                "2025-10-01 01:02",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:00",
            ),
            (
                "2025-10-01 01:02:03",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            (
                "2025-10-01T01:02:03+00:00",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            (
                "2025-10-01T01:02:03+09:00",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            ("20251001010203", "%Y-%m-%d %H:%M:%S", "2025-10-01 01:02:03"),
            ("202510010102", "%Y-%m-%d %H:%M:%S", "2025-10-01 01:02:00"),
            ("20251001", "%Y-%m-%d %H:%M:%S", "2025-10-01 00:00:00"),
            ("2025-10-01", "%Y-%m-%d %H:%M:%S", "2025-10-01 00:00:00"),
        ];
        for (input, format, expected) in test_cases {
            let result = parse_start_time(input).unwrap().format(format).to_string();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_end_time() {
        let test_cases = vec![
            (
                "2025-10-01 01:02",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:59",
            ),
            (
                "2025-10-01 01:02:03",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            (
                "2025-10-01T01:02:03+00:00",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            (
                "2025-10-01T01:02:03+09:00",
                "%Y-%m-%d %H:%M:%S",
                "2025-10-01 01:02:03",
            ),
            ("20251001010203", "%Y-%m-%d %H:%M:%S", "2025-10-01 01:02:03"),
            ("202510010102", "%Y-%m-%d %H:%M:%S", "2025-10-01 01:02:59"),
            ("20251001", "%Y-%m-%d %H:%M:%S", "2025-10-01 23:59:59"),
            ("2025-10-01", "%Y-%m-%d %H:%M:%S", "2025-10-01 23:59:59"),
        ];
        for (input, format, expected) in test_cases {
            let result = parse_end_time(input).unwrap().format(format).to_string();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_end_after_start() {
        let test_cases = vec![
            ("2025-10-01 01:02:02", "2025-10-01 01:02:03", true),
            ("2025-10-01 01:02:02", "2025-10-01 01:02:01", false),
            ("2025-10-01 01:02:03", "2025-10-01 01:02:03", true),
        ];
        for (start_input, end_input, expected) in test_cases {
            let start = TimeZone::from_utc_datetime(
                &Local,
                &NaiveDateTime::parse_from_str(start_input, "%Y-%m-%d %H:%M:%S").unwrap(),
            );
            let end = TimeZone::from_utc_datetime(
                &Local,
                &NaiveDateTime::parse_from_str(end_input, "%Y-%m-%d %H:%M:%S").unwrap(),
            );
            assert_eq!(end_after_start(&end, &start).is_ok(), expected);
        }
    }

    #[test]
    fn test_parse_duration() {
        assert_eq!(parse_duration("1s"), Ok(Duration::seconds(1)));
        assert_eq!(parse_duration("2m"), Ok(Duration::minutes(2)));
        assert_eq!(parse_duration("3h"), Ok(Duration::hours(3)));
        assert_eq!(parse_duration("4d"), Ok(Duration::days(4)));
        assert!(parse_duration("5x").is_err());
    }
}
