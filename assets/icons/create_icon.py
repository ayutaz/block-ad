#!/usr/bin/env python3
"""
Create app icon for AdBlock in multiple sizes
Uses a simple shield design with "Ad" text
"""

import os
from PIL import Image, ImageDraw, ImageFont

def create_shield_icon(size):
    """Create a shield-shaped icon with Ad text"""
    # Create a new image with transparent background
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Calculate dimensions
    margin = size * 0.1
    shield_width = size - 2 * margin
    shield_height = size - 2 * margin
    
    # Shield coordinates
    x1 = margin
    y1 = margin
    x2 = size - margin
    y2 = size - margin
    
    # Create shield shape
    shield_points = [
        (x1 + shield_width * 0.5, y1),  # Top center
        (x2, y1 + shield_height * 0.2),  # Top right
        (x2, y1 + shield_height * 0.6),  # Right side
        (x1 + shield_width * 0.5, y2),  # Bottom point
        (x1, y1 + shield_height * 0.6),  # Left side
        (x1, y1 + shield_height * 0.2),  # Top left
    ]
    
    # Draw gradient background (red to dark red)
    for i in range(int(shield_height)):
        color_value = int(220 - (i / shield_height) * 40)
        temp_color = (color_value, 30, 30, 255)
        y_offset = y1 + i
        draw.line([(x1, y_offset), (x2, y_offset)], fill=temp_color, width=1)
    
    # Draw shield outline
    draw.polygon(shield_points, fill=(200, 30, 30, 255))
    draw.line(shield_points + [shield_points[0]], fill=(150, 20, 20, 255), width=int(size * 0.02))
    
    # Add "Ad" text
    text = "Ad"
    # Try to use a bold font, fallback to default if not available
    try:
        font_size = int(size * 0.35)
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", font_size)
    except:
        font = ImageFont.load_default()
    
    # Get text dimensions
    bbox = draw.textbbox((0, 0), text, font=font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    
    # Center text
    text_x = (size - text_width) / 2
    text_y = (size - text_height) / 2 - size * 0.05
    
    # Draw text with shadow
    shadow_offset = size * 0.02
    draw.text((text_x + shadow_offset, text_y + shadow_offset), text, 
              font=font, fill=(100, 10, 10, 255))
    draw.text((text_x, text_y), text, font=font, fill=(255, 255, 255, 255))
    
    # Add crossed line
    line_width = int(size * 0.06)
    draw.line([(x1 + shield_width * 0.2, y1 + shield_height * 0.7),
               (x2 - shield_width * 0.2, y1 + shield_height * 0.3)],
              fill=(255, 255, 255, 255), width=line_width)
    
    return img

def main():
    """Generate icons in multiple sizes"""
    # Icon sizes needed for iOS and Android
    sizes = {
        # iOS App Icon sizes
        'ios': [
            (20, 2), (20, 3),  # 20pt
            (29, 2), (29, 3),  # 29pt
            (40, 2), (40, 3),  # 40pt
            (60, 2), (60, 3),  # 60pt
            (1024, 1),  # App Store
        ],
        # Android launcher icon sizes
        'android': [
            (48, 1),    # mdpi
            (72, 1),    # hdpi
            (96, 1),    # xhdpi
            (144, 1),   # xxhdpi
            (192, 1),   # xxxhdpi
            (512, 1),   # Play Store
        ]
    }
    
    # Create directories
    os.makedirs('ios', exist_ok=True)
    os.makedirs('android', exist_ok=True)
    
    # Generate iOS icons
    for base_size, scale in sizes['ios']:
        size = base_size * scale
        icon = create_shield_icon(size)
        filename = f'ios/icon-{base_size}@{scale}x.png'
        icon.save(filename, 'PNG')
        print(f'Created {filename} ({size}x{size})')
    
    # Generate Android icons
    density_names = ['mdpi', 'hdpi', 'xhdpi', 'xxhdpi', 'xxxhdpi', 'web']
    for i, (size, _) in enumerate(sizes['android']):
        icon = create_shield_icon(size)
        density = density_names[i]
        filename = f'android/ic_launcher_{density}.png'
        icon.save(filename, 'PNG')
        print(f'Created {filename} ({size}x{size})')
    
    # Create a master icon
    master = create_shield_icon(1024)
    master.save('icon_master.png', 'PNG')
    print('Created icon_master.png (1024x1024)')

if __name__ == '__main__':
    main()