# Python Package

The `imessage-analysis` Python package provides query and analysis functions over an existing iMessage dataset. All functions return `pyarrow.Table` for seamless pandas integration.

## Installation

```sh
pip install imessage-analysis
```

## Requirements

- Python 3.11+
- `pyarrow` (installed automatically)
- macOS to build the dataset; query functions work on any platform if the Parquet dataset is already present

## Important: sync from the CLI first

The Python package is **query-only**. It reads a dataset built by the CLI tool. Before using it, run:

```sh
imessage-analysis sync
```

Calling `imessage_analysis.sync()` from Python will raise a `RuntimeError` — the Python interpreter does not have the required macOS Contacts permission, so any sync from Python would index messages but leave all names as phone numbers. Use the CLI to sync; use the Python package to query.

## Quick start

```python
import imessage_analysis

# Returns pyarrow.Table — call .to_pandas() to convert
df = imessage_analysis.top_contacts(limit=10).to_pandas()
print(df)
```

## Analysis functions

All functions return `pyarrow.Table`.

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
df = imessage_analysis.reactions(contact="Alice", year=2024).to_pandas()
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

### `contact_stats(contact=None, limit=50, data_dir=None)`

```python
df = imessage_analysis.contact_stats(limit=20).to_pandas()
df = imessage_analysis.contact_stats(contact="Alice").to_pandas()
```

### `search_contacts(query, limit=20, data_dir=None)`

```python
df = imessage_analysis.search_contacts("alice").to_pandas()
```

### `query(sql, data_dir=None) → pyarrow.Table`

Execute arbitrary SQL against the `messages` table.

```python
df = imessage_analysis.query(
    "SELECT year, COUNT(*) AS n FROM messages GROUP BY year ORDER BY year"
).to_pandas()
```

## Using in Jupyter notebooks

```python
import imessage_analysis
import pandas as pd
import matplotlib.pyplot as plt

df = imessage_analysis.query("SELECT * FROM messages").to_pandas()
df['timestamp'] = pd.to_datetime(df['timestamp'])

yearly = df.groupby('year').size()
yearly.plot(kind='bar', title='Messages per year')
plt.tight_layout()
plt.show()
```

## Building from source

The Python package is built with [maturin](https://github.com/PyO3/maturin).

```sh
pip install maturin
cd crates/imessage-python
maturin develop --release
```
