#!/usr/bin/env python3
"""
Bouncing ball animation using curses.
This will test cursor positioning, colors, and screen clearing.
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
    curses.init_pair(1, curses.COLOR_RED, curses.COLOR_BLACK)
    curses.init_pair(2, curses.COLOR_GREEN, curses.COLOR_BLACK)
    curses.init_pair(3, curses.COLOR_BLUE, curses.COLOR_BLACK)
    curses.init_pair(4, curses.COLOR_YELLOW, curses.COLOR_BLACK)
    curses.init_pair(5, curses.COLOR_MAGENTA, curses.COLOR_BLACK)
    curses.init_pair(6, curses.COLOR_CYAN, curses.COLOR_BLACK)
    
    # Get screen dimensions
    height, width = stdscr.getmaxyx()
    
    # Ball properties
    ball_x, ball_y = width // 2, height // 2
    dx, dy = 1, 1
    ball_char = "●"
    color_idx = 1
    
    frame = 0
    
    while True:
        # Check for quit
        key = stdscr.getch()
        if key == ord('q') or key == 27:  # 'q' or ESC
            break
            
        # Clear screen
        stdscr.clear()
        
        # Update ball position
        ball_x += dx
        ball_y += dy
        
        # Bounce off walls
        if ball_x <= 0 or ball_x >= width - 1:
            dx = -dx
            color_idx = (color_idx % 6) + 1  # Cycle through colors
            
        if ball_y <= 0 or ball_y >= height - 1:
            dy = -dy
            color_idx = (color_idx % 6) + 1  # Cycle through colors
        
        # Keep ball in bounds
        ball_x = max(0, min(width - 1, ball_x))
        ball_y = max(0, min(height - 1, ball_y))
        
        # Draw ball
        try:
            stdscr.addstr(ball_y, ball_x, ball_char, curses.color_pair(color_idx))
        except curses.error:
            pass  # Ignore if position is invalid
        
        # Draw frame counter and instructions
        try:
            stdscr.addstr(0, 0, f"Frame: {frame}", curses.color_pair(7) if curses.COLORS > 7 else 0)
            stdscr.addstr(1, 0, "Press 'q' to quit", curses.color_pair(7) if curses.COLORS > 7 else 0)
            stdscr.addstr(height - 1, 0, f"Ball at ({ball_x}, {ball_y})", curses.color_pair(7) if curses.COLORS > 7 else 0)
        except curses.error:
            pass
        
        # Refresh screen
        stdscr.refresh()
        
        frame += 1
        time.sleep(0.05)  # 20 FPS

if __name__ == "__main__":
    try:
        curses.wrapper(main)
    except KeyboardInterrupt:
        sys.exit(0) 