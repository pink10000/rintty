#!/usr/bin/env python3
"""
Simple text animation - basic test case.
Just prints lines of text with basic formatting.
"""

import time
import sys

def main():
    lines = [
        "╔═══════════════════════════════════════╗",
        "║            RINTTY TEST                ║", 
        "║         Animation System              ║",
        "╠═══════════════════════════════════════╣",
        "║                                       ║",
        "║  This is a simple text animation      ║",
        "║  that tests basic VTE parsing.        ║",
        "║                                       ║",
        "║  Features being tested:               ║",
        "║  ✓ Line-by-line text output           ║",
        "║  ✓ Unicode characters                 ║", 
        "║  ✓ Cursor positioning                 ║",
        "║  ✓ Screen clearing                    ║",
        "║                                       ║",
        "╚═══════════════════════════════════════╝",
    ]
    
    try:
        frame = 0
        while True:
            # Clear screen using ANSI codes
            print("\x1b[2J\x1b[H", end="")
            
            # Print the box
            for i, line in enumerate(lines):
                print(f"\x1b[{i+5};10H{line}")
            
            # Print dynamic content
            print(f"\x1b[25;10HFrame: {frame}")
            print(f"\x1b[26;10HTime: {time.strftime('%H:%M:%S')}")
            print(f"\x1b[27;10HPress Ctrl+C to quit")
            
            # Add some simple animation
            dots = "." * ((frame % 4) + 1)
            print(f"\x1b[23;10HAnimating{dots:<4}")
            
            sys.stdout.flush()
            frame += 1
            time.sleep(0.5)
            
    except KeyboardInterrupt:
        print("\x1b[2J\x1b[H")  # Clear screen
        print("Animation stopped.")
        sys.exit(0)

if __name__ == "__main__":
    main() 