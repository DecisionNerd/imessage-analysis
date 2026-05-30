use comfy_table::{presets::UTF8_FULL_CONDENSED, Cell, CellAlignment, Table};
use datafusion::arrow::datatypes::DataType;
use datafusion::arrow::error::ArrowError;
use datafusion::arrow::record_batch::RecordBatch;
use datafusion::arrow::util::display::{ArrayFormatter, FormatOptions};

pub enum Format {
    Table,
    Json,
    Csv,
}

impl Format {
    pub fn from_str(s: &str) -> Self {
        match s {
            "json" => Self::Json,
            "csv" => Self::Csv,
            _ => Self::Table,
        }
    }
}

pub fn print_batches(batches: &[RecordBatch], format: &Format, limit: usize) {
    if batches.is_empty() || batches.iter().all(|b| b.num_rows() == 0) {
        println!("(no results)");
        return;
    }

    match format {
        Format::Table => print_table(batches, limit),
        Format::Json => print_json(batches, limit),
        Format::Csv => print_csv(batches, limit),
    }
}

fn is_numeric(dt: &DataType) -> bool {
    matches!(
        dt,
        DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64
            | DataType::Float32
            | DataType::Float64
    )
}

fn make_formatters<'a>(
    batch: &'a RecordBatch,
    opts: &'a FormatOptions,
) -> Result<Vec<ArrayFormatter<'a>>, ArrowError> {
    (0..batch.num_columns())
        .map(|i| ArrayFormatter::try_new(batch.column(i).as_ref(), opts))
        .collect()
}

fn print_table(batches: &[RecordBatch], limit: usize) {
    let opts = FormatOptions::default().with_null("NULL");
    let schema = batches[0].schema();

    let mut table = Table::new();
    table.load_preset(UTF8_FULL_CONDENSED);
    table.set_header(schema.fields().iter().map(|f| f.name().as_str()));

    // Right-align numeric columns
    for (i, field) in schema.fields().iter().enumerate() {
        if is_numeric(field.data_type()) {
            if let Some(c) = table.column_mut(i) {
                c.set_cell_alignment(CellAlignment::Right);
            }
        }
    }

    let mut printed = 0;
    let mut total = 0usize;

    for batch in batches {
        let Ok(formatters) = make_formatters(batch, &opts) else {
            continue;
        };
        for row in 0..batch.num_rows() {
            total += 1;
            if printed < limit {
                let cells: Vec<Cell> = formatters
                    .iter()
                    .map(|f| Cell::new(f.value(row).to_string()))
                    .collect();
                table.add_row(cells);
                printed += 1;
            }
        }
    }

    println!("{table}");
    if total > printed {
        println!("({printed} of {total} rows — use --limit to see more)");
    } else {
        println!("({total} rows)");
    }
}

fn print_json(batches: &[RecordBatch], limit: usize) {
    let opts = FormatOptions::default();
    let schema = batches[0].schema();
    let fields: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();

    let mut printed = 0;
    'outer: for batch in batches {
        let Ok(formatters) = make_formatters(batch, &opts) else {
            continue;
        };
        for row in 0..batch.num_rows() {
            if printed >= limit {
                break 'outer;
            }
            let mut map = serde_json::Map::new();
            for (name, f) in fields.iter().zip(formatters.iter()) {
                map.insert(
                    name.to_string(),
                    serde_json::Value::String(f.value(row).to_string()),
                );
            }
            println!("{}", serde_json::Value::Object(map));
            printed += 1;
        }
    }
}

fn print_csv(batches: &[RecordBatch], limit: usize) {
    let opts = FormatOptions::default();
    let schema = batches[0].schema();
    let fields: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
    println!("{}", fields.join(","));

    let mut printed = 0;
    'outer: for batch in batches {
        let Ok(formatters) = make_formatters(batch, &opts) else {
            continue;
        };
        for row in 0..batch.num_rows() {
            if printed >= limit {
                break 'outer;
            }
            let vals: Vec<String> = formatters
                .iter()
                .map(|f| {
                    let v = f.value(row).to_string();
                    if v.contains(',') || v.contains('"') || v.contains('\n') {
                        format!("\"{}\"", v.replace('"', "\"\""))
                    } else {
                        v
                    }
                })
                .collect();
            println!("{}", vals.join(","));
            printed += 1;
        }
    }
}
