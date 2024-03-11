pub fn format_duration(date: chrono::Duration) -> String {
    let mut string = String::with_capacity(64);

    if date.num_days() > 0 {
        string.push_str(&format!("{} dias, ", date.num_days()));
    }

    if date.num_hours() > 0 {
        string.push_str(&format!("{} horas, ", date.num_hours()));
    }

    string.push_str(&format!(
        "{} minutos e {} segundos",
        date.num_minutes() % 60,
        date.num_seconds() % 60
    ));

    string
}
