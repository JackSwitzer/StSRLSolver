#!/usr/bin/env python3
"""Quick test of Run 7 extraction with verified seed."""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from vod.run_extraction import run_extraction

if __name__ == "__main__":
    run_extraction(
        video_path="vod_data/merl/run7_first_hour.mkv",  # First hour clip
        seed="227QYN385T72G",  # Run 7 verified seed
        output_path="vod_data/merl/run7_extraction.json",
        chunk_minutes=5,
        video_duration_minutes=60,  # Full first hour
    )
