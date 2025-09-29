#!/usr/bin/env python3
"""
Very simple curses test - just basic text output.
The goal of this file was to debug the lines being overwritten. 
"""

import curses
import time
import sys

def main(stdscr):
    # Don't hide cursor or set nodelay - keep it simple
    
    # Basic test - just write some text
    stdscr.addstr(0, 0, "Hello World")
    stdscr.addstr(1, 0, "Line 2")
    stdscr.addstr(2, 0, "Line 3")
    
    # Refresh to display
    stdscr.refresh()
    
    # Wait a bit
    time.sleep(3)
    
    # Try colors if available
    if curses.has_colors():
        curses.start_color()
        curses.init_pair(1, curses.COLOR_RED, curses.COLOR_BLACK)
        
        stdscr.addstr(4, 0, "RED TEXT", curses.color_pair(1))
        stdscr.refresh()
        time.sleep(2)
    
    # Try background
    try:
        curses.init_pair(2, curses.COLOR_WHITE, curses.COLOR_BLUE)
        stdscr.addstr(5, 0, "BLUE BACKGROUND", curses.color_pair(2))
        stdscr.refresh()
        time.sleep(2)
    except:
        pass

if __name__ == "__main__":
    try:
        curses.wrapper(main)
    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1) 