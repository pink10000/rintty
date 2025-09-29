#!/usr/bin/env python3
"""
Debug what goes through the PTY by writing very specific sequences.
"""

import sys
import time

def main():
    # Test 1: Basic text with explicit positioning
    print("TEST 1: Basic text")
    sys.stdout.flush()
    time.sleep(1)
    
    # Test 2: Clear and position
    print("\x1b[2J\x1b[H", end="")  # Clear screen, home
    print("Line 1")
    print("Line 2") 
    print("Line 3")
    sys.stdout.flush()
    time.sleep(2)
    
    # Test 3: Explicit positioning
    print("\x1b[5;10HExplicit Position", end="")
    sys.stdout.flush()
    time.sleep(2)
    
    # Test 4: Background color
    print("\x1b[6;1H", end="")  # Row 6, Col 1
    print("\x1b[41m", end="")   # Red background
    print("RED BACKGROUND")     # This includes a newline
    print("\x1b[0m", end="")    # Reset
    sys.stdout.flush()
    time.sleep(3)

if __name__ == "__main__":
    main() 