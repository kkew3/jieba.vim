"""
Quoted from Github repository: https://github.com/tsroten/zhon.
File: https://github.com/tsroten/zhon/blob/main/src/zhon/hanzi.py.
"""

#: A string containing Chinese punctuation marks (non-stops).
non_stops = (
    # Fullwidth ASCII variants
    "\uFF02\uFF03\uFF04\uFF05\uFF06\uFF07\uFF08\uFF09\uFF0A\uFF0B\uFF0C\uFF0D"
    "\uFF0F\uFF1A\uFF1B\uFF1C\uFF1D\uFF1E\uFF20\uFF3B\uFF3C\uFF3D\uFF3E\uFF3F"
    "\uFF40\uFF5B\uFF5C\uFF5D\uFF5E\uFF5F\uFF60"
    # Halfwidth CJK punctuation
    "\uFF62\uFF63\uFF64"
    # CJK symbols and punctuation
    "\u3000\u3001\u3003"
    # CJK angle and corner brackets
    "\u3008\u3009\u300A\u300B\u300C\u300D\u300E\u300F\u3010\u3011"
    # CJK brackets and symbols/punctuation
    "\u3014\u3015\u3016\u3017\u3018\u3019\u301A\u301B\u301C\u301D\u301E\u301F"
    # Other CJK symbols
    "\u3030"
    # Special CJK indicators
    "\u303E\u303F"
    # Dashes
    "\u2013\u2014"
    # Quotation marks and apostrophe
    "\u2018\u2019\u201B\u201C\u201D\u201E\u201F"
    # General punctuation
    "\u2026\u2027"
    # Overscores and underscores
    "\uFE4F"
    # Small form variants
    "\uFE51\uFE54"
    # Latin punctuation
    "\u00B7"
)

#: A string of Chinese stops.
stops = (
    "\uFF0E"  # Fullwidth full stop
    "\uFF01"  # Fullwidth exclamation mark
    "\uFF1F"  # Fullwidth question mark
    "\uFF61"  # Halfwidth ideographic full stop
    "\u3002"  # Ideographic full stop
)

#: A string containing all Chinese punctuation.
punctuation = non_stops + stops