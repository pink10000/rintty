#!/usr/bin/env python3
"""
Background color test - isolates the background rendering issue.
"""

import curses
import time
import sys

def main(stdscr):
    # Initialize curses
    curses.curs_set(0)  # Hide cursor
    stdscr.nodelay(True)  # Non-blocking input
    
    # Initialize colors
    curses.start_color()
    curses.init_pair(1, curses.COLOR_WHITE, curses.COLOR_RED)    # White on Red
    curses.init_pair(2, curses.COLOR_BLACK, curses.COLOR_GREEN)  # Black on Green  
    curses.init_pair(3, curses.COLOR_YELLOW, curses.COLOR_BLUE)  # Yellow on Blue
    curses.init_pair(4, curses.COLOR_WHITE, curses.COLOR_BLACK)  # White on Black (normal)
    
    # Get screen dimensions
    height, width = stdscr.getmaxyx()
    
    frame = 0
    
    while True:
        # Check for quit
        key = stdscr.getch()
        if key == ord('q') or key == 27:  # 'q' or ESC
            break
        
        # Test 1: Clear screen and set background
        stdscr.clear()
        stdscr.bkgd(' ', curses.color_pair(2))  # Green background
        
        # Test 2: Draw lines with different backgrounds
        try:
            # Line 1: Red background with text
            for x in range(min(40, width)):
                stdscr.addch(2, x, ' ', curses.color_pair(1))
            stdscr.addstr(2, 5, "RED BACKGROUND", curses.color_pair(1))
            
            # Line 2: Blue background with text  
            for x in range(min(40, width)):
                stdscr.addch(4, x, ' ', curses.color_pair(3))
            stdscr.addstr(4, 5, "BLUE BACKGROUND", curses.color_pair(3))
            
            # Line 3: Normal text
            stdscr.addstr(6, 5, "Normal text (should have green background)", curses.color_pair(4))
            
            # Test 3: Single character with background
            stdscr.addstr(8, 5, "Single char test:", curses.color_pair(4))
            stdscr.addch(8, 25, 'X', curses.color_pair(1))  # Should have red background
            
            # Test 4: Frame info
            stdscr.addstr(10, 5, f"Frame: {frame}", curses.color_pair(4))
            stdscr.addstr(11, 5, "Screen should have green background", curses.color_pair(4))
            stdscr.addstr(12, 5, "Press 'q' to quit", curses.color_pair(4))
            
            # Test 5: Fill a rectangle with background color
            for y in range(15, 20):
                for x in range(10, 30):
                    if y < height and x < width:
                        stdscr.addch(y, x, ' ', curses.color_pair(1))
            stdscr.addstr(17, 12, "FILLED RECT", curses.color_pair(1))
            
        except curses.error:
            pass
        
        # Refresh screen
        stdscr.refresh()
        
        frame += 1
        time.sleep(0.2)  # 5 FPS

if __name__ == "__main__":
    try:
        curses.wrapper(main)
    except KeyboardInterrupt:
        sys.exit(0) 