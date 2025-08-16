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
    pub title: Option<String>,
    pub theme: String,
}

impl Args {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    #[allow(clippy::needless_pass_by_value)]
    pub fn parse(matches: ArgMatches) -> Self {
        let start = matches
            .get_one::<DateTime<Local>>("start")
            .copied()
            .unwrap();
        let end = matches
            .get_one::<DateTime<Local>>("end")
            .copied()
            .unwrap_or_else(|| start + matches.get_one::<Duration>("duration").copied().unwrap());

        if end < start {
            println!(
                "End time {end} must be after start time {start}.",
                start = start.format("%Y-%m-%d %H:%M:%S"),
                end = end.format("%Y-%m-%d %H:%M:%S")
            );
            exit(1);
        }

        Args {
            start,
            end,
            interval: matches.get_one("interval").copied().unwrap(),
            verbose: matches.get_one("verbose").copied().unwrap(),
            title: matches.get_one::<String>("title").cloned(),
            theme: matches
                .get_one::<String>("theme")
                .cloned()
                .unwrap_or_else(|| "default".to_string()),
        }
    }
}

pub fn build_command() -> Command {
    Command::new("doit")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Just Do It! - Progress Bar Tool for Motivation")
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
        .arg(
            clap::Arg::new("title")
                .short('t')
                .long("title")
                .value_parser(clap::value_parser!(String))
                .help("Custom title for the progress session"),
        )
        .arg(
            clap::Arg::new("theme")
                .long("theme")
                .value_parser(clap::value_parser!(String))
                .default_value("default")
                .help("Theme for the progress display [default|retro|cyberpunk]"),
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
        return Ok(convert_from_utc(&datetime));
    }
    Err(format!("Invalid start time format: {s}"))
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
        return Ok(convert_from_utc(&datetime));
    }
    Err(format!("Invalid end time format: {s}"))
}

fn parse_datetime_as_ymd_hmsz(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M:%S%z", "%Y-%m-%d %H:%M:%S%z"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(convert_from_utc(&datetime));
        }
    }
    Err(format!("Invalid datetime format: {s}"))
}

fn parse_datetime_as_ymd_hms(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M:%S", "%Y-%m-%d %H:%M:%S", "%Y%m%d%H%M%S"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(convert_from_utc(&datetime));
        }
    }
    Err(format!("Invalid datetime format: {s}"))
}

fn parse_datetime_as_ymd_hm(s: &str) -> Result<DateTime<Local>, String> {
    let formats = ["%Y-%m-%dT%H:%M", "%Y-%m-%d %H:%M", "%Y%m%d%H%M"];
    for format in &formats {
        if let Ok(datetime) = NaiveDateTime::parse_from_str(s, format) {
            return Ok(convert_from_utc(&datetime));
        }
    }
    Err(format!("Invalid datetime format: {s}"))
}

fn convert_from_utc(datetime: &NaiveDateTime) -> DateTime<Local> {
    TimeZone::from_utc_datetime(&Local, datetime)
}

fn parse_date(s: &str) -> Result<NaiveDate, String> {
    let date_formats = ["%Y-%m-%d", "%Y%m%d"];
    for format in &date_formats {
        if let Ok(date) = chrono::NaiveDate::parse_from_str(s, format) {
            return Ok(date);
        }
    }
    Err(format!("Invalid date format: {s}"))
}

fn parse_duration(s: &str) -> Result<Duration, String> {
    let re = Regex::new(r"^(\d+)([smhd])$").unwrap();
    if let Some(caps) = re.captures(s) {
        let value = caps[1]
            .parse::<i64>()
            .map_err(|_| format!("Invalid duration value: {s}"))?;
        let unit = &caps[2];
        match unit {
            "s" => return Ok(Duration::seconds(value)),
            "m" => return Ok(Duration::minutes(value)),
            "h" => return Ok(Duration::hours(value)),
            "d" => return Ok(Duration::days(value)),
            _ => {}
        }
    }
    Err(format!("Invalid duration format: {s}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_with_start() {
        let args = vec![
            "doit",
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
        let args = vec!["doit", "--end", &end];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.start, now);
    }

    #[test]
    fn test_parse_with_end() {
        let args = vec![
            "doit",
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
        let args = vec!["doit", "--start", "2025-01-01 10:20:30", "--duration", "1s"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 10:20:31"
        );
    }

    #[test]
    fn test_parse_with_duration_minutes() {
        let args = vec!["doit", "--start", "2025-01-01 10:20:30", "--duration", "1m"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 10:21:30"
        );
    }

    #[test]
    fn test_parse_with_duration_hours() {
        let args = vec!["doit", "--start", "2025-01-01 10:20:30", "--duration", "1h"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-01 11:20:30"
        );
    }

    #[test]
    fn test_parse_with_duration_days() {
        let args = vec!["doit", "--start", "2025-01-01 10:20:30", "--duration", "1d"];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(
            args.end.format("%Y-%m-%d %H:%M:%S").to_string(),
            "2025-01-02 10:20:30"
        );
    }

    #[test]
    fn test_parse_with_valid_interval() {
        let args = vec![
            "doit",
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
    fn test_parse_with_invalid_interval() {
        let test_cases = vec!["0", "61", "x"];
        for interval in test_cases {
            let args = vec![
                "doit",
                "--start",
                "2025-01-01 10:20:30",
                "--end",
                "2025-01-31 23:59:59",
                "--interval",
                interval,
            ];
            let command = build_command();
            let result = command.try_get_matches_from(args);
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_parse_without_interval() {
        let args = vec![
            "doit",
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
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--verbose",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert!(args.verbose);
    }

    #[test]
    fn test_parse_without_verbose() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert!(!args.verbose);
    }

    #[test]
    fn test_parse_start_time_with_success() {
        let test_cases = vec![
            ("2025-10-01 01:02", "2025-10-01 01:02:00"),
            ("2025-10-01 01:02:03", "2025-10-01 01:02:03"),
            ("2025-10-01T01:02:03+00:00", "2025-10-01 01:02:03"),
            ("2025-10-01T01:02:03+09:00", "2025-10-01 01:02:03"),
            ("20251001010203", "2025-10-01 01:02:03"),
            ("202510010102", "2025-10-01 01:02:00"),
            ("20251001", "2025-10-01 00:00:00"),
            ("2025-10-01", "2025-10-01 00:00:00"),
        ];
        for (input, expected) in test_cases {
            let result = parse_start_time(input)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_start_time_with_failure() {
        let test_cases = vec![
            // %Y-%m-%d %H:%M:%S
            "2025-00-01 01:02:00",
            "2025-13-01 01:02:03",
            "2025-10-00 01:02:00",
            "2025-10-32 01:02:03",
            "2025-10-01 25:02:03",
            "2025-10-01 01:60:03",
            // %Y-%m-%d %H:%M
            "2025-00-01 01:02",
            "2025-13-01 01:02",
            "2025-10-00 01:02",
            "2025-10-32 01:02",
            "2025-10-01 25:02",
            "2025-10-01 01:60",
            // %Y-%m-%dT%H:%M:%S
            "2025-00-01T01:02:00",
            "2025-00-01T01:02:00",
            "2025-13-01T01:02:03",
            "2025-10-00T01:02:00",
            "2025-10-32T01:02:03",
            "2025-10-01T25:02:03",
            "2025-10-01T01:60:03",
            // %Y-%m-%dT%H:%M
            "2025-00-01T01:02",
            "2025-00-01T01:02",
            "2025-13-01T01:02",
            "2025-10-00T01:02",
            "2025-10-32T01:02",
            "2025-10-01T25:02",
            // %Y%m%d%H%M%S
            "20250001010200",
            "20251301010203",
            "20251000010200",
            "20251032010203",
            "20251001250203",
            "20251001016003",
            // %Y%m%d%H%M
            "202500010102",
            "202513010102",
            "202510000102",
            "202510320102",
            "202510012502",
            "202510010160",
            // %Y-%m-%d
            "2025-00-01",
            "2025-13-01",
            "2025-10-00",
            "2025-10-32",
            // %Y%m%d
            "20250001",
            "20251301",
            "20251000",
            "20251032",
        ];
        for input in test_cases {
            let result = parse_start_time(input);
            assert!(result.is_err(), "Failed to parse start time: {input}");
        }
    }

    #[test]
    fn test_parse_end_time_with_success() {
        let test_cases = vec![
            ("2025-10-01 01:02", "2025-10-01 01:02:59"),
            ("2025-10-01 01:02:03", "2025-10-01 01:02:03"),
            ("2025-10-01T01:02:03+00:00", "2025-10-01 01:02:03"),
            ("2025-10-01T01:02:03+09:00", "2025-10-01 01:02:03"),
            ("20251001010203", "2025-10-01 01:02:03"),
            ("202510010102", "2025-10-01 01:02:59"),
            ("20251001", "2025-10-01 23:59:59"),
            ("2025-10-01", "2025-10-01 23:59:59"),
        ];
        for (input, expected) in test_cases {
            let result = parse_end_time(input)
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_parse_end_time_with_failure() {
        let test_cases = vec![
            // %Y-%m-%d %H:%M:%S
            "2025-00-01 01:02:00",
            "2025-13-01 01:02:03",
            "2025-10-00 01:02:00",
            "2025-10-32 01:02:03",
            "2025-10-01 25:02:03",
            "2025-10-01 01:60:03",
            // %Y-%m-%d %H:%M
            "2025-00-01 01:02",
            "2025-13-01 01:02",
            "2025-10-00 01:02",
            "2025-10-32 01:02",
            "2025-10-01 25:02",
            "2025-10-01 01:60",
            // %Y-%m-%dT%H:%M:%S
            "2025-00-01T01:02:00",
            "2025-00-01T01:02:00",
            "2025-13-01T01:02:03",
            "2025-10-00T01:02:00",
            "2025-10-32T01:02:03",
            "2025-10-01T25:02:03",
            "2025-10-01T01:60:03",
            // %Y-%m-%dT%H:%M
            "2025-00-01T01:02",
            "2025-00-01T01:02",
            "2025-13-01T01:02",
            "2025-10-00T01:02",
            "2025-10-32T01:02",
            "2025-10-01T25:02",
            // %Y%m%d%H%M%S
            "20250001010200",
            "20251301010203",
            "20251000010200",
            "20251032010203",
            "20251001250203",
            "20251001016003",
            // %Y%m%d%H%M
            "202500010102",
            "202513010102",
            "202510000102",
            "202510320102",
            "202510012502",
            "202510010160",
            // %Y-%m-%d
            "2025-00-01",
            "2025-13-01",
            "2025-10-00",
            "2025-10-32",
            // %Y%m%d
            "20250001",
            "20251301",
            "20251000",
            "20251032",
        ];
        for input in test_cases {
            let result = parse_end_time(input);
            assert!(result.is_err(), "Failed to parse start time: {input}");
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

    #[test]
    fn test_convert_from_utc() {
        let test_cases = vec![
            ("2025-10-01 01:02:03+09:00", "2025-10-01 01:02:03"),
            ("2025-10-01 01:02:03+00:00", "2025-10-01 01:02:03"),
        ];
        for (input, expected) in test_cases {
            let datetime_with_tz = DateTime::parse_from_str(input, "%Y-%m-%d %H:%M:%S%z").unwrap();
            let naive_datetime = datetime_with_tz.naive_local();
            let local_datetime = convert_from_utc(&naive_datetime);
            assert_eq!(
                local_datetime.format("%Y-%m-%d %H:%M:%S").to_string(),
                expected,
                "Failed for input: {input}",
            );
        }
    }

    #[test]
    fn test_parse_with_title() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--title",
            "My Custom Title",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.title, Some("My Custom Title".to_string()));
    }

    #[test]
    fn test_parse_with_title_short() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "-t",
            "Short Title",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.title, Some("Short Title".to_string()));
    }

    #[test]
    fn test_parse_without_title() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.title, None);
        assert_eq!(args.theme, "default");
    }

    #[test]
    fn test_parse_with_retro_theme() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--theme",
            "retro",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.theme, "retro");
    }

    #[test]
    fn test_parse_with_default_theme() {
        let args = vec![
            "doit",
            "--start",
            "2025-01-01 10:20:30",
            "--end",
            "2025-01-31 23:59:59",
            "--theme",
            "default",
        ];
        let command = build_command();
        let args = Args::parse(command.get_matches_from(args));
        assert_eq!(args.theme, "default");
    }
}
