# Python Package

The `imessage-analysis` Python package wraps the same Rust core as the CLI, returning results as `pyarrow.Table` objects for seamless pandas integration.

## Installation

```sh
pip install imessage-analysis
```

## Requirements

- Python 3.11+
- `pyarrow` (installed automatically as a dependency)
- macOS for ETL; query/analysis works on any platform if a Parquet dataset is present

## Quick start

```python
import imessage_analysis

# Build the dataset (requires macOS + Full Disk Access)
imessage_analysis.run_etl()

# Query — returns pyarrow.Table
table = imessage_analysis.query("SELECT name, COUNT(*) AS n FROM messages GROUP BY name ORDER BY n DESC LIMIT 10")

# Convert to pandas
df = table.to_pandas()
print(df)
```

## ETL functions

### `run_etl(db_path=None, data_dir=None, contacts_config=None, auto_contacts=True)`

Run the full ETL pipeline.

```python
imessage_analysis.run_etl()

# Custom paths
imessage_analysis.run_etl(
    db_path="/Volumes/Backup/chat.db",
    data_dir="/tmp/my-messages/",
    contacts_config="/Users/me/contacts.toml",
)
```

### `refresh(data_dir=None)`

Incremental update — only processes messages since the last ETL run.

```python
imessage_analysis.refresh()
```

## Query function

### `query(sql, data_dir=None, limit=None) → pyarrow.Table`

Execute arbitrary SQL. The table is named `messages`.

```python
# All messages from a specific year
table = imessage_analysis.query("SELECT * FROM messages WHERE year = 2024")
df = table.to_pandas()

# Message count by year
imessage_analysis.query(
    "SELECT year, COUNT(*) AS n FROM messages GROUP BY year ORDER BY year"
).to_pandas()
```

## Analysis functions

All analysis functions return `pyarrow.Table`.

### `top_contacts(limit=10, year=None, direct_only=True, data_dir=None)`

```python
df = imessage_analysis.top_contacts(limit=20, year=2024).to_pandas()
```

### `time_series(contact=None, window=28, start=None, end=None, data_dir=None)`

```python
df = imessage_analysis.time_series(contact="Alice", window=7).to_pandas()
df = imessage_analysis.time_series(start="2023-01-01", end="2023-12-31").to_pandas()
```

### `reactions(contact=None, year=None, data_dir=None)`

```python
df = imessage_analysis.reactions().to_pandas()
```

### `effects(year=None, data_dir=None)`

```python
df = imessage_analysis.effects(year=2024).to_pandas()
```

### `links(limit=20, data_dir=None)`

```python
df = imessage_analysis.links(limit=30).to_pandas()
```

### `seasonality(kind="dow", data_dir=None)`

```python
dow = imessage_analysis.seasonality(kind="dow").to_pandas()
monthly = imessage_analysis.seasonality(kind="month").to_pandas()
```

### `contact_stats(contact=None, data_dir=None)`

```python
df = imessage_analysis.contact_stats().to_pandas()
```

## Using in Jupyter notebooks

The Python package is the recommended interface for notebook-based analysis.

```python
import imessage_analysis
import pandas as pd
import matplotlib.pyplot as plt

# Load data
df = imessage_analysis.query("SELECT * FROM messages").to_pandas()
df['timestamp'] = pd.to_datetime(df['timestamp'])

# Messages per year
yearly = df.groupby('year').size()
yearly.plot(kind='bar', title='Messages per year')
plt.tight_layout()
plt.show()
```

## Building from source

The Python package is built with [maturin](https://github.com/PyO3/maturin). To build a development version:

```sh
pip install maturin
cd crates/imessage-python
maturin develop --release
```

To build a distributable wheel:

```sh
maturin build --release
# wheel is written to target/wheels/
```
