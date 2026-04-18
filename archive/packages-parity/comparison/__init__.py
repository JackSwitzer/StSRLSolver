"""
Game State Comparison Framework

Tools for comparing Python emulator state against actual game state.
"""

from .save_reader import (
    read_save_file,
    SaveState,
    RNGState,
    CardSave,
    decode_save_file,
    encode_save_file,
    get_default_save_path,
)
