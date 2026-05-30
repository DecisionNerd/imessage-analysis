#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Sent,
    Received,
}

impl Direction {
    fn as_sql(self) -> &'static str {
        match self {
            Direction::Sent => "is_from_me = 1",
            Direction::Received => "is_from_me = 0",
        }
    }
}

/// Named analysis queries. Each returns a SQL string ready for execution.
pub fn top_contacts(
    limit: usize,
    year: Option<i32>,
    direct_only: bool,
    direction: Option<Direction>,
) -> String {
    let mut filters = vec!["name IS NOT NULL".to_string()];
    if direct_only {
        filters.push("chat_size = 1".to_string());
    }
    if let Some(y) = year {
        filters.push(format!("year = {y}"));
    }
    if let Some(d) = direction {
        filters.push(d.as_sql().to_string());
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
    direction: Option<Direction>,
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
    if let Some(d) = direction {
        filters.push(d.as_sql().to_string());
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

pub fn reactions(contact: Option<&str>, year: Option<i32>, direction: Option<Direction>) -> String {
    let mut filters = vec!["reaction != 'no-reaction'".to_string()];
    if let Some(c) = contact {
        let escaped = c.replace('\'', "''");
        filters.push(format!("name = '{escaped}'"));
    }
    if let Some(y) = year {
        filters.push(format!("year = {y}"));
    }
    if let Some(d) = direction {
        filters.push(d.as_sql().to_string());
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

pub fn seasonality_dow(direction: Option<Direction>) -> String {
    let extra = direction
        .map(|d| format!(" AND {}", d.as_sql()))
        .unwrap_or_default();
    format!(
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
             COUNT(*) AS messages
         FROM messages
         WHERE timestamp IS NOT NULL{extra}
         GROUP BY dow_number, day_of_week
         ORDER BY dow_number"
    )
}

pub fn seasonality_month(direction: Option<Direction>) -> String {
    let extra = direction
        .map(|d| format!(" AND {}", d.as_sql()))
        .unwrap_or_default();
    format!(
        "SELECT
             month,
             COUNT(*) AS messages
         FROM messages
         WHERE month IS NOT NULL{extra}
         GROUP BY month
         ORDER BY month"
    )
}

/// Substring search across name and contact_info columns.
pub fn search_contacts(query: &str, limit: usize) -> String {
    let escaped = query
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
        .replace('\'', "''")
        .to_lowercase();
    format!(
        "SELECT
             name,
             contact_info,
             COUNT(*) AS message_count
         FROM messages
         WHERE (lower(coalesce(name, '')) LIKE '%{escaped}%' ESCAPE '\\'
             OR lower(coalesce(contact_info, '')) LIKE '%{escaped}%' ESCAPE '\\')
           AND (name IS NOT NULL OR contact_info IS NOT NULL)
         GROUP BY name, contact_info
         ORDER BY message_count DESC
         LIMIT {limit}"
    )
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
             COUNT(DISTINCT CAST(date AS VARCHAR)) AS active_days,
             ROUND(CAST(COUNT(*) AS DOUBLE) /
                 NULLIF(CAST(
                     DATEDIFF('day', CAST(MIN(CAST(date AS VARCHAR)) AS DATE), CAST(MAX(CAST(date AS VARCHAR)) AS DATE)) + 1
                 AS DOUBLE), 0), 2) AS avg_per_day
         FROM messages
         {where_clause}
         GROUP BY name
         ORDER BY total_messages DESC"
    )
}
