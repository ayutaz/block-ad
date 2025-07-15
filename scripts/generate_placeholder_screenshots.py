#!/usr/bin/env python3
"""
Generate placeholder screenshots for app store submissions.
Requires: Pillow (pip install Pillow)
"""

from PIL import Image, ImageDraw, ImageFont
import os
from pathlib import Path

# Brand colors
PRIMARY_COLOR = (0, 122, 255)  # iOS Blue
BACKGROUND_COLOR = (245, 245, 247)  # Light gray
TEXT_COLOR = (0, 0, 0)
ACCENT_COLOR = (255, 59, 48)  # Red for emphasis

# Screenshot dimensions
IOS_SIZES = {
    "6.7-inch": (1290, 2796),
    "6.5-inch": (1284, 2778),
    "6.1-inch": (1179, 2556),
    "5.5-inch": (1242, 2208),
    "12.9-inch": (2048, 2732)
}

ANDROID_SIZES = {
    "phone": (1080, 2400),  # Common 20:9 ratio
    "tablet": (1600, 2560)  # 10" tablet
}

# Screenshot content
SCREENSHOTS = [
    {
        "title": "Block Ads Everywhere",
        "subtitle": "System-wide ad blocking",
        "stats": ["12,345 Ads Blocked", "1.2 GB Data Saved", "99% Block Rate"]
    },
    {
        "title": "Real-time Protection",
        "subtitle": "VPN-based filtering",
        "stats": ["Active Protection", "0ms Latency", "No Data Logging"]
    },
    {
        "title": "YouTube Ad Blocking",
        "subtitle": "Skip video ads automatically",
        "stats": ["80%+ YouTube Ads", "Instant Skip", "Buffer-free Videos"]
    },
    {
        "title": "Privacy First",
        "subtitle": "Your data stays on device",
        "stats": ["On-device Processing", "No Cloud Servers", "Open Source"]
    },
    {
        "title": "Custom Rules",
        "subtitle": "Take full control",
        "stats": ["Custom Filters", "Whitelist Sites", "Import/Export"]
    }
]

def create_placeholder_screenshot(width, height, screenshot_data, index):
    """Create a placeholder screenshot with given dimensions and content."""
    # Create image
    img = Image.new('RGB', (width, height), BACKGROUND_COLOR)
    draw = ImageDraw.Draw(img)
    
    # Calculate positions
    center_x = width // 2
    padding = width // 10
    
    # Try to use a nice font, fallback to default
    try:
        title_font_size = width // 15
        subtitle_font_size = width // 20
        stat_font_size = width // 25
        
        # Try to load system fonts
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_font_size)
        subtitle_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", subtitle_font_size)
        stat_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", stat_font_size)
    except:
        # Fallback to default font
        title_font = ImageFont.load_default()
        subtitle_font = ImageFont.load_default()
        stat_font = ImageFont.load_default()
    
    # Draw app icon placeholder
    icon_size = width // 4
    icon_top = height // 10
    icon_left = (width - icon_size) // 2
    draw.rounded_rectangle(
        [(icon_left, icon_top), (icon_left + icon_size, icon_top + icon_size)],
        radius=icon_size // 5,
        fill=PRIMARY_COLOR
    )
    
    # Draw "AdBlock" text in icon
    icon_text = "AB"
    bbox = draw.textbbox((0, 0), icon_text, font=title_font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    draw.text(
        (icon_left + (icon_size - text_width) // 2, 
         icon_top + (icon_size - text_height) // 2),
        icon_text,
        fill=(255, 255, 255),
        font=title_font
    )
    
    # Draw title
    title_y = icon_top + icon_size + padding
    bbox = draw.textbbox((0, 0), screenshot_data["title"], font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, title_y),
        screenshot_data["title"],
        fill=TEXT_COLOR,
        font=title_font
    )
    
    # Draw subtitle
    subtitle_y = title_y + title_font_size + padding // 2
    bbox = draw.textbbox((0, 0), screenshot_data["subtitle"], font=subtitle_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, subtitle_y),
        screenshot_data["subtitle"],
        fill=(100, 100, 100),
        font=subtitle_font
    )
    
    # Draw stats boxes
    stats_y = subtitle_y + subtitle_font_size + padding
    stat_height = height // 8
    stat_spacing = padding // 2
    
    for i, stat in enumerate(screenshot_data["stats"]):
        stat_y = stats_y + i * (stat_height + stat_spacing)
        
        # Draw stat box
        draw.rounded_rectangle(
            [(padding, stat_y), (width - padding, stat_y + stat_height)],
            radius=20,
            fill=(255, 255, 255),
            outline=PRIMARY_COLOR,
            width=3
        )
        
        # Draw stat text
        bbox = draw.textbbox((0, 0), stat, font=stat_font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        draw.text(
            ((width - text_width) // 2, 
             stat_y + (stat_height - text_height) // 2),
            stat,
            fill=PRIMARY_COLOR,
            font=stat_font
        )
    
    # Draw "Screenshot Placeholder" watermark
    watermark = f"Screenshot {index + 1} - Placeholder"
    bbox = draw.textbbox((0, 0), watermark, font=subtitle_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, height - padding * 2),
        watermark,
        fill=(200, 200, 200),
        font=subtitle_font
    )
    
    return img

def generate_app_icons():
    """Generate placeholder app icons."""
    sizes = {
        "ios": 1024,
        "android": 512
    }
    
    for platform, size in sizes.items():
        img = Image.new('RGB', (size, size), PRIMARY_COLOR)
        draw = ImageDraw.Draw(img)
        
        # Draw "AB" text
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", size // 3)
        except:
            font = ImageFont.load_default()
        
        text = "AB"
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        draw.text(
            ((size - text_width) // 2, (size - text_height) // 2),
            text,
            fill=(255, 255, 255),
            font=font
        )
        
        # Save icon
        if platform == "ios":
            path = Path("assets/app-store/ios/graphics/app-icon-1024.png")
        else:
            path = Path("assets/app-store/android/graphics/icon-512.png")
        
        path.parent.mkdir(parents=True, exist_ok=True)
        img.save(path, "PNG")
        print(f"Generated {path}")

def generate_android_feature_graphic():
    """Generate Android feature graphic (1024x500)."""
    img = Image.new('RGB', (1024, 500), PRIMARY_COLOR)
    draw = ImageDraw.Draw(img)
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 80)
        subtitle_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 40)
    except:
        title_font = ImageFont.load_default()
        subtitle_font = ImageFont.load_default()
    
    # Draw title
    title = "AdBlock"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((1024 - text_width) // 2, 150),
        title,
        fill=(255, 255, 255),
        font=title_font
    )
    
    # Draw subtitle
    subtitle = "System-wide Ad Blocker"
    bbox = draw.textbbox((0, 0), subtitle, font=subtitle_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((1024 - text_width) // 2, 250),
        subtitle,
        fill=(255, 255, 255),
        font=subtitle_font
    )
    
    # Draw features
    features = "Free • Open Source • Privacy Focused"
    bbox = draw.textbbox((0, 0), features, font=subtitle_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((1024 - text_width) // 2, 350),
        features,
        fill=(200, 200, 200),
        font=subtitle_font
    )
    
    path = Path("assets/app-store/android/graphics/feature-graphic.png")
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, "PNG")
    print(f"Generated {path}")

def main():
    """Generate all placeholder screenshots."""
    print("Generating placeholder screenshots...")
    
    # Generate iOS screenshots
    for size_name, (width, height) in IOS_SIZES.items():
        print(f"\nGenerating iOS {size_name} screenshots...")
        for i, screenshot_data in enumerate(SCREENSHOTS):
            img = create_placeholder_screenshot(width, height, screenshot_data, i)
            path = Path(f"assets/app-store/ios/screenshots/{size_name}/screenshot_{i+1}.png")
            path.parent.mkdir(parents=True, exist_ok=True)
            img.save(path, "PNG")
            print(f"  Generated {path}")
    
    # Generate Android screenshots
    for device_type, (width, height) in ANDROID_SIZES.items():
        print(f"\nGenerating Android {device_type} screenshots...")
        for i, screenshot_data in enumerate(SCREENSHOTS):
            img = create_placeholder_screenshot(width, height, screenshot_data, i)
            path = Path(f"assets/app-store/android/screenshots/{device_type}/screenshot_{i+1}.png")
            path.parent.mkdir(parents=True, exist_ok=True)
            img.save(path, "PNG")
            print(f"  Generated {path}")
    
    # Generate app icons
    print("\nGenerating app icons...")
    generate_app_icons()
    
    # Generate Android feature graphic
    print("\nGenerating Android feature graphic...")
    generate_android_feature_graphic()
    
    print("\n✅ All placeholder screenshots generated successfully!")
    print("Replace these with actual screenshots before app store submission.")

if __name__ == "__main__":
    main()