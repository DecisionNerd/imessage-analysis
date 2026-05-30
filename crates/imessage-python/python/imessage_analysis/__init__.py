"""
imessage-analysis — query and analyse your Mac iMessage history from Python.

All query and analysis functions return ``pyarrow.Table``.
Call ``.to_pandas()`` on any result to convert to a pandas DataFrame.

Quick start::

    import imessage_analysis

    # Sync must be run from the CLI first — see note below
    df = imessage_analysis.top_contacts().to_pandas()

Note on syncing
---------------
The Python package is query-only. It reads an existing dataset built by the
CLI tool. To build or update the dataset, run this from your terminal::

    imessage-analysis sync

The CLI binary has the required macOS Contacts permission; the Python
interpreter does not, so calling sync() from Python will index messages
but leave all contact names as phone numbers.
"""

from imessage_analysis._lib import (
    contact_stats,
    effects,
    links,
    query,
    reactions,
    search_contacts,
    seasonality,
    time_series,
    top_contacts,
)

__all__ = [
    "query",
    "search_contacts",
    "top_contacts",
    "time_series",
    "reactions",
    "effects",
    "links",
    "seasonality",
    "contact_stats",
]


def sync(*args, **kwargs):
    raise RuntimeError(
        "imessage_analysis.sync() is not supported from Python.\n"
        "Run `imessage-analysis sync` from your terminal instead.\n"
        "The CLI binary has the required macOS Contacts permission to resolve names.\n"
        "Once synced, all query functions work normally here."
    )


def run_etl(*args, **kwargs):
    raise RuntimeError(
        "imessage_analysis.run_etl() is not supported from Python.\n"
        "Run `imessage-analysis sync` from your terminal instead."
    )


def refresh(*args, **kwargs):
    raise RuntimeError(
        "imessage_analysis.refresh() is not supported from Python.\n"
        "Run `imessage-analysis sync` from your terminal instead."
    )
