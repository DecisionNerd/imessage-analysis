/// Named analysis queries. Each returns a SQL string ready for execution.

pub fn top_contacts(limit: usize, year: Option<i32>, direct_only: bool) -> String {
    let mut filters = vec!["name IS NOT NULL".to_string()];
    if direct_only {
        filters.push("chat_size = 1".to_string());
    }
    if let Some(y) = year {
        filters.push(format!("year = {y}"));
    }
    let where_clause = format!("WHERE {}", filters.join(" AND "));
    format!(
        "SELECT name, COUNT(*) AS message_count
         FROM messages
         {where_clause}
         GROUP BY name
         ORDER BY message_count DESC
         LIMIT {limit}"
    )
}

pub fn time_series(
    contact: Option<&str>,
    window: usize,
    start: Option<&str>,
    end: Option<&str>,
) -> String {
    let mut filters: Vec<String> = vec!["date IS NOT NULL".to_string()];
    if let Some(c) = contact {
        let escaped = c.replace('\'', "''");
        filters.push(format!("name = '{escaped}'"));
    }
    if let Some(s) = start {
        let escaped = s.replace('\'', "''");
        filters.push(format!("date >= '{escaped}'"));
    }
    if let Some(e) = end {
        let escaped = e.replace('\'', "''");
        filters.push(format!("date <= '{escaped}'"));
    }
    let where_clause = format!("WHERE {}", filters.join(" AND "));
    format!(
        "SELECT
             CAST(date AS VARCHAR) AS date,
             COUNT(*) AS messages,
             AVG(COUNT(*)) OVER (
                 ORDER BY date
                 ROWS BETWEEN {preceding} PRECEDING AND CURRENT ROW
             ) AS rolling_avg
         FROM messages
         {where_clause}
         GROUP BY date
         ORDER BY date",
        preceding = window.saturating_sub(1)
    )
}

pub fn reactions(contact: Option<&str>, year: Option<i32>) -> String {
    let mut filters = vec!["reaction != 'no-reaction'".to_string()];
    if let Some(c) = contact {
        let escaped = c.replace('\'', "''");
        filters.push(format!("name = '{escaped}'"));
    }
    if let Some(y) = year {
        filters.push(format!("year = {y}"));
    }
    let where_clause = format!("WHERE {}", filters.join(" AND "));
    format!(
        "SELECT reaction, COUNT(*) AS count
         FROM messages
         {where_clause}
         GROUP BY reaction
         ORDER BY count DESC"
    )
}

pub fn effects(year: Option<i32>) -> String {
    let mut filters = vec!["message_effect != 'no-effect'".to_string()];
    if let Some(y) = year {
        filters.push(format!("year = {y}"));
    }
    let where_clause = format!("WHERE {}", filters.join(" AND "));
    format!(
        "SELECT message_effect, COUNT(*) AS count
         FROM messages
         {where_clause}
         GROUP BY message_effect
         ORDER BY count DESC"
    )
}

pub fn links(limit: usize) -> String {
    format!(
        "SELECT link_domain, COUNT(*) AS count
         FROM messages
         WHERE link_domain IS NOT NULL
         GROUP BY link_domain
         ORDER BY count DESC
         LIMIT {limit}"
    )
}

pub fn seasonality_dow() -> &'static str {
    "SELECT
         EXTRACT(DOW FROM CAST(timestamp AS TIMESTAMP)) AS dow_number,
         CASE EXTRACT(DOW FROM CAST(timestamp AS TIMESTAMP))
             WHEN 0 THEN 'Sunday'
             WHEN 1 THEN 'Monday'
             WHEN 2 THEN 'Tuesday'
             WHEN 3 THEN 'Wednesday'
             WHEN 4 THEN 'Thursday'
             WHEN 5 THEN 'Friday'
             WHEN 6 THEN 'Saturday'
         END AS day_of_week,
         SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
         SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
     FROM messages
     WHERE timestamp IS NOT NULL
     GROUP BY dow_number, day_of_week
     ORDER BY dow_number"
}

pub fn seasonality_month() -> &'static str {
    "SELECT
         month,
         SUM(CASE WHEN is_from_me = 1 THEN 1 ELSE 0 END) AS sent,
         SUM(CASE WHEN is_from_me = 0 THEN 1 ELSE 0 END) AS received
     FROM messages
     WHERE month IS NOT NULL
     GROUP BY month
     ORDER BY month"
}

pub fn contact_stats(contact: Option<&str>) -> String {
    let where_clause = match contact {
        Some(c) => {
            let escaped = c.replace('\'', "''");
            format!("WHERE name = '{escaped}'")
        }
        None => "WHERE name IS NOT NULL".to_string(),
    };
    format!(
        "SELECT
             name,
             COUNT(*) AS total_messages,
             MIN(CAST(date AS VARCHAR)) AS first_date,
             MAX(CAST(date AS VARCHAR)) AS last_date,
             COUNT(DISTINCT CAST(date AS VARCHAR)) AS active_days
         FROM messages
         {where_clause}
         GROUP BY name
         ORDER BY total_messages DESC"
    )
}
