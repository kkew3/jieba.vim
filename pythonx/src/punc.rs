//! Quoted from Github repository: https://github.com/tsroten/zhon.
//! File: https://github.com/tsroten/zhon/blob/main/src/zhon/hanzi.py.

use const_format::concatcp;

/// A string containing Chinese punctuation marks (non-stops).
pub const NON_STOPS: &str = concatcp!(
    // Fullwidth ASCII variants.
    "\u{FF02}\u{FF03}\u{FF04}\u{FF05}\u{FF06}\u{FF07}\u{FF08}\u{FF09}\u{FF0A}",
    "\u{FF0B}\u{FF0C}\u{FF0D}\u{FF0F}\u{FF1A}\u{FF1B}\u{FF1C}\u{FF1D}\u{FF1E}",
    "\u{FF20}\u{FF3B}\u{FF3C}\u{FF3D}\u{FF3E}\u{FF3F}\u{FF40}\u{FF5B}\u{FF5C}",
    "\u{FF5D}\u{FF5E}\u{FF5F}\u{FF60}",
    // Halfwidth CJK punctuation.
    "\u{FF62}\u{FF63}\u{FF64}",
    // CJK symbols and punctuation.
    "\u{3000}\u{3001}\u{3003}",
    // CJK angle and corner brackets.
    "\u{3008}\u{3009}\u{300A}\u{300B}\u{300C}\u{300D}\u{300E}\u{300F}\u{3010}",
    "\u{3011}",
    // CJK brackets and symbols/punctuations.
    "\u{3014}\u{3015}\u{3016}\u{3017}\u{3018}\u{3019}\u{301A}\u{301B}\u{301C}",
    "\u{301D}\u{301E}\u{301F}",
    // Other CJK symbols.
    "\u{3030}",
    // Special CJK indicators.
    "\u{303E}\u{303F}",
    // Dashes.
    "\u{2013}\u{2014}",
    // Quotation marks and apostrophe.
    "\u{2018}\u{2019}\u{201B}\u{201C}\u{201D}\u{201E}\u{201F}",
    // General punctuation.
    "\u{2026}\u{2027}",
    // Overscores and underscores.
    "\u{FE4F}",
    // Small form variants.
    "\u{FE51}\u{FE54}",
    // Latin punctuation.
    "\u{00B7}",
);

/// A string of Chinese stops.
pub const STOPS: &str = concatcp!(
    "\u{FF0E}", // Fullwidth full stop.
    "\u{FF01}", // Fullwidth exclamation mark.
    "\u{FF1F}", // Fullwidth question mark.
    "\u{FF61}", // Halfwidth ideographic full stop.
    "\u{3002}", // Ideographic full stop.
);

/// A string containing all Chinese punctuation.
pub const PUNCTUATION: &str = concatcp!(NON_STOPS, STOPS);
