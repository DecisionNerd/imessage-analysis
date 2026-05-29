"""
imessage-analysis — extract, query, and analyse your Mac iMessage history.

All query/analysis functions return ``pyarrow.Table``.
Call ``.to_pandas()`` on any result to convert to a pandas DataFrame.

Quick start::

    import imessage_analysis

    imessage_analysis.run_etl()                      # build the dataset once
    imessage_analysis.refresh()                      # incremental update
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
    seasonality,
    time_series,
    top_contacts,
)

__all__ = [
    "run_etl",
    "refresh",
    "query",
    "top_contacts",
    "time_series",
    "reactions",
    "effects",
    "links",
    "seasonality",
    "contact_stats",
]
