#!/usr/bin/env python3
"""
Inspired by https://github.com/Eric-Lennartson/terminal-art/blob/main/cube.py

This is a cube that spins in the terminal. It uses the curses library to draw 
the cube. It uses the math library to calculate the cube.

"""


import curses
from curses import wrapper
from time import sleep
import random
import math

class vec3:
    def __init__(self, x=0, y=0, z=0):
        self.x = x
        self.y = y
        self.z = z

def rotate(vec, ax, ay, az):
    a = math.cos(ax)
    b = math.sin(ax)
    c = math.cos(ay)
    d = math.sin(ay)
    e = math.cos(az)
    f = math.sin(az)

    nx = c * e * vec.x - c * f * vec.y + d * vec.z
    ny = (a * f + b * d * e) * vec.x + (a * e - b * d * f) * vec.y - b * c * vec.z
    nz = (b * f - a * d * e) * vec.x + (a * d * f + b * e) * vec.y + a * c * vec.z

    return vec3(nx, ny, nz)

def scale(vec, x=1, y=1, z=1):
    return vec3(vec.x * x, vec.y * y, vec.z * z)

def translate(vec, x, y, z):
    nx = vec.x + x
    ny = vec.y + y
    nz = vec.z + z
    return vec3(nx, ny, nz)

def line(screen, vec1, vec2, rows, cols):
    """Draw a line between two points with bounds checking"""
    xdist = math.ceil(abs(vec1.x - vec2.x))
    ydist = math.ceil(abs(vec1.y - vec2.y))

    dist = max(xdist, ydist)

    for i in range(dist):
        xpos = int(map_value(i, 0, dist, vec1.x, vec2.x))
        ypos = int(map_value(i, 0, dist, vec1.y, vec2.y))
        
        # Bounds checking - ensure we don't write outside the screen
        if 0 <= ypos < rows and 0 <= xpos < cols:
            screen[ypos][xpos] = MAX_BRIGHT

def map_value(value, leftMin, leftMax, rightMin, rightMax):
    # Figure out how 'wide' each range is
    leftSpan = leftMax - leftMin
    rightSpan = rightMax - rightMin

    # Convert the left range into a 0-1 range (float)
    valueScaled = float(value - leftMin) / float(leftSpan)

    # Convert the 0-1 range into a value in the right range.
    return rightMin + (valueScaled * rightSpan)

def clamp(value, minval, maxval):
    return max(min(value, maxval), minval)

def draw_cube(screen, cube, rows, cols):
    """Draw cube with bounds checking"""
    line(screen, cube[0], cube[1], rows, cols)
    line(screen, cube[1], cube[2], rows, cols)
    line(screen, cube[2], cube[3], rows, cols)
    line(screen, cube[3], cube[0], rows, cols)
    line(screen, cube[4], cube[5], rows, cols)
    line(screen, cube[5], cube[6], rows, cols)
    line(screen, cube[6], cube[7], rows, cols)
    line(screen, cube[7], cube[4], rows, cols)
    line(screen, cube[0], cube[4], rows, cols)
    line(screen, cube[1], cube[5], rows, cols)
    line(screen, cube[2], cube[6], rows, cols)
    line(screen, cube[3], cube[7], rows, cols)

def create_cube(x, y, size=10):
    a = vec3(x - size, y + size * 0.45, x + size)
    b = vec3(x + size, y + size * 0.45, x + size)
    c = vec3(x + size, y - size * 0.45, x + size)
    d = vec3(x - size, y - size * 0.45, x + size)
    e = vec3(x - size, y + size * 0.45, x - size)
    f = vec3(x + size, y + size * 0.45, x - size)
    g = vec3(x + size, y - size * 0.45, x - size)
    h = vec3(x - size, y - size * 0.45, x - size)
    return [a, b, c, d, e, f, g, h]

# globals
bright = [' ','.',':','-','=','+','*','#','%','@']
MAX_BRIGHT = len(bright)-1
refresh_rate = 0.06

def main(stdscr):
    stdscr.clear()
    curses.curs_set(0) # remove the cursor

    # Use standard colors that work with rintty
    if curses.has_colors():
        curses.start_color()
        curses.use_default_colors()

        # Initialize standard color pairs instead of custom colors
        curses.init_pair(1, curses.COLOR_RED, -1)
        curses.init_pair(2, curses.COLOR_GREEN, -1)
        curses.init_pair(3, curses.COLOR_YELLOW, -1)
        curses.init_pair(4, curses.COLOR_BLUE, -1)
        curses.init_pair(5, curses.COLOR_MAGENTA, -1)
        curses.init_pair(6, curses.COLOR_CYAN, -1)
        curses.init_pair(7, curses.COLOR_WHITE, -1)

    rows = curses.LINES
    cols = curses.COLS - 1  # Leave space to avoid line wrapping

    # Ensure the cube fits within the screen
    max_size = min(rows // 2, cols // 4)
    cube_size = min(40, max_size)  # Increased from 20 to 40

    screen = [[0 for x in range(cols)] for y in range(rows)]
    colors = [[0 for x in range(cols)] for y in range(rows)]

    centerx = cols / 2
    centery = rows / 2

    cube = create_cube(centerx, centery, cube_size)

    decay_rate = 0.8
    idx = 0.0
    frame_count = 0

    try:
        while True:
            stdscr.clear()

            # Draw the cube with bounds checking
            draw_cube(screen, cube, rows, cols)

            # Update colors array
            for y in range(0, rows):
                for x in range(0, cols):
                    colors[y][x] = int(idx) % 7 + 1  # Use color pairs 1-7
                    idx += 0.0045
                    idx %= 70  # Prevent overflow

            # Rotate the cube
            for v in range(len(cube)):
                cube[v] = translate(cube[v], -centerx, -centery, -centerx)
                cube[v] = scale(cube[v], 1, 1 / 0.45, 1)
                cube[v] = rotate(cube[v], 0.015, 0.015, 0.001)
                cube[v] = scale(cube[v], 1, 0.45, 1) # adjusting back
                cube[v] = translate(cube[v], centerx, centery, centerx) # adjusting back

            # Display the screen with bounds checking
            for y in range(0, rows):
                for x in range(0, cols):
                    brightness = screen[y][x]
                    if brightness > 0:
                        b = bright[min(int(brightness), len(bright) - 1)]
                        color_pair = colors[y][x] if curses.has_colors() else 0
                        try:
                            stdscr.addstr(y, x, b, curses.color_pair(color_pair))
                        except curses.error:
                            # Ignore errors when writing to last position
                            pass
                        screen[y][x] = max(0, screen[y][x] - decay_rate)

            # Add frame counter and instructions
            try:
                stdscr.addstr(0, 0, f"Frame: {frame_count}", curses.color_pair(7) if curses.has_colors() else 0)
                stdscr.addstr(1, 0, "Press 'q' to quit", curses.color_pair(7) if curses.has_colors() else 0)
            except curses.error:
                pass

            stdscr.refresh()
            
            # Check for input to quit
            stdscr.nodelay(True)
            key = stdscr.getch()
            if key == ord('q') or key == 27:  # 'q' or ESC
                break
            
            sleep(refresh_rate)
            frame_count += 1

    except KeyboardInterrupt:
        pass

if __name__ == "__main__":
    wrapper(main) 