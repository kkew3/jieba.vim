import logging

from .libjieba import setLogLevel as _setLogLevel
from .libjieba import initialize as _initialize
from .libjieba import cut as jieba_cut


def jieba_initialize(dictionary=None):
    _setLogLevel(logging.WARNING)
    _initialize(dictionary)
