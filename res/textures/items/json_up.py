def print_rectangles(x_step, max_x):
    x = 0.0
    while x <= max_x:
        print('    {')
        print('      "x": {:.1f},'.format(x))
        print('      "y": 0.0,')
        print('      "w": 16.0,')
        print('      "h": 16.0')
        print('    },')
        x += x_step

print_rectangles(16.0, 272.0)
