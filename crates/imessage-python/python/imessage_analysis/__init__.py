"""
imessage-analysis — extract, query, and analyse your Mac iMessage history.

All query/analysis functions return ``pyarrow.Table``.
Call ``.to_pandas()`` on any result to convert to a pandas DataFrame.

Quick start::

    import imessage_analysis

    imessage_analysis.sync()                         # build or update the dataset
    df = imessage_analysis.top_contacts().to_pandas()
"""

from imessage_analysis._lib import (
    contact_stats,
    effects,
    links,
    query,
    reactions,
    refresh,
    run_etl,
    search_contacts,
    seasonality,
    sync,
    time_series,
    top_contacts,
)

__all__ = [
    "sync",
    "run_etl",
    "refresh",
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
