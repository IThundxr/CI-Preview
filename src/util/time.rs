pub fn format_duration(total_seconds: i64) -> String {
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut output = String::new();

    if hours > 0 {
        output += &format!("{} hour{}", hours, if hours == 1 { "" } else { "s" });
    }

    if minutes > 0 {
        if !output.is_empty() {
            output += ", ";
        }

        output += &format!("{} minute{}", minutes, if minutes == 1 { "" } else { "s" });
    }

    if seconds > 0 {
        if !output.is_empty() {
            output += " and ";
        }

        output += &format!("{} second{}", seconds, if seconds == 1 { "" } else { "s" });
    }

    output
}
