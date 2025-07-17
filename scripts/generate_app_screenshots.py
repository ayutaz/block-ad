#!/usr/bin/env python3
"""
Generate realistic app screenshots for app store submissions.
Requires: Pillow (pip install Pillow)
"""

from PIL import Image, ImageDraw, ImageFont
import os
from pathlib import Path
import datetime

# Brand colors
PRIMARY_COLOR = (0, 122, 255)  # iOS Blue
BACKGROUND_COLOR = (245, 245, 247)  # Light gray
DARK_BACKGROUND = (28, 28, 30)  # Dark mode
TEXT_COLOR = (0, 0, 0)
TEXT_COLOR_DARK = (255, 255, 255)
ACCENT_COLOR = (255, 59, 48)  # Red for emphasis
SUCCESS_COLOR = (52, 199, 89)  # Green
CARD_COLOR = (255, 255, 255)
CARD_COLOR_DARK = (44, 44, 46)

# Screenshot dimensions
IOS_SIZES = {
    "6.7-inch": (1290, 2796),  # iPhone 14 Pro Max
    "6.5-inch": (1284, 2778),  # iPhone 14 Plus
    "6.1-inch": (1179, 2556),  # iPhone 14 Pro
    "5.5-inch": (1242, 2208),  # iPhone 8 Plus
    "12.9-inch": (2048, 2732)  # iPad Pro
}

ANDROID_SIZES = {
    "phone": (1080, 2400),  # Common 20:9 ratio
    "tablet": (1600, 2560)  # 10" tablet
}

def create_main_screen(width, height, dark_mode=False):
    """Create the main protection screen."""
    bg_color = DARK_BACKGROUND if dark_mode else BACKGROUND_COLOR
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    card_color = CARD_COLOR_DARK if dark_mode else CARD_COLOR
    
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    # Font sizes
    title_size = width // 20
    large_text_size = width // 10
    medium_text_size = width // 25
    small_text_size = width // 30
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_size)
        large_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", large_text_size)
        medium_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", medium_text_size)
        small_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", small_text_size)
    except:
        title_font = ImageFont.load_default()
        large_font = ImageFont.load_default()
        medium_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
    
    padding = width // 20
    y_offset = height // 10
    
    # Draw status bar
    draw_status_bar(draw, width, 0, dark_mode)
    
    # Title
    title = "AdBlock"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        title,
        fill=text_color,
        font=title_font
    )
    
    y_offset += title_size + padding
    
    # Protection status circle
    circle_size = width // 3
    circle_x = (width - circle_size) // 2
    circle_y = y_offset
    
    # Outer circle
    draw.ellipse(
        [(circle_x - 10, circle_y - 10), 
         (circle_x + circle_size + 10, circle_y + circle_size + 10)],
        fill=SUCCESS_COLOR,
        outline=SUCCESS_COLOR
    )
    
    # Inner circle
    draw.ellipse(
        [(circle_x, circle_y), 
         (circle_x + circle_size, circle_y + circle_size)],
        fill=card_color
    )
    
    # Check mark
    check_size = circle_size // 3
    check_x = circle_x + circle_size // 2
    check_y = circle_y + circle_size // 2
    draw.line(
        [(check_x - check_size//2, check_y),
         (check_x - check_size//6, check_y + check_size//3),
         (check_x + check_size//2, check_y - check_size//3)],
        fill=SUCCESS_COLOR,
        width=width//50
    )
    
    # Status text
    y_offset = circle_y + circle_size + padding
    status_text = "保護中"
    bbox = draw.textbbox((0, 0), status_text, font=large_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        status_text,
        fill=SUCCESS_COLOR,
        font=large_font
    )
    
    y_offset += large_text_size + padding * 2
    
    # Stats cards
    stats = [
        ("ブロック済み広告", "12,345"),
        ("節約データ", "1.2 GB"),
        ("ブロック率", "99.2%")
    ]
    
    card_height = height // 10
    card_spacing = padding // 2
    
    for stat_name, stat_value in stats:
        # Draw card
        draw.rounded_rectangle(
            [(padding, y_offset), (width - padding, y_offset + card_height)],
            radius=20,
            fill=card_color
        )
        
        # Stat name
        draw.text(
            (padding * 2, y_offset + card_height // 4),
            stat_name,
            fill=(150, 150, 150) if not dark_mode else (180, 180, 180),
            font=small_font
        )
        
        # Stat value
        bbox = draw.textbbox((0, 0), stat_value, font=medium_font)
        text_width = bbox[2] - bbox[0]
        draw.text(
            (width - padding * 2 - text_width, y_offset + card_height // 4),
            stat_value,
            fill=PRIMARY_COLOR,
            font=medium_font
        )
        
        y_offset += card_height + card_spacing
    
    # Navigation bar
    draw_navigation_bar(draw, width, height, dark_mode)
    
    return img

def create_youtube_screen(width, height, dark_mode=False):
    """Create YouTube ad blocking screen."""
    bg_color = DARK_BACKGROUND if dark_mode else BACKGROUND_COLOR
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    card_color = CARD_COLOR_DARK if dark_mode else CARD_COLOR
    
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    # Font sizes
    title_size = width // 20
    header_size = width // 15
    medium_text_size = width // 25
    small_text_size = width // 30
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_size)
        header_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", header_size)
        medium_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", medium_text_size)
        small_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", small_text_size)
    except:
        title_font = ImageFont.load_default()
        header_font = ImageFont.load_default()
        medium_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
    
    padding = width // 20
    y_offset = height // 10
    
    # Draw status bar
    draw_status_bar(draw, width, 0, dark_mode)
    
    # Title
    title = "YouTube広告ブロック"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        title,
        fill=text_color,
        font=title_font
    )
    
    y_offset += title_size + padding * 2
    
    # YouTube icon placeholder
    icon_size = width // 4
    icon_x = (width - icon_size) // 2
    draw.rounded_rectangle(
        [(icon_x, y_offset), (icon_x + icon_size, y_offset + icon_size)],
        radius=20,
        fill=ACCENT_COLOR
    )
    
    # Play button in icon
    play_size = icon_size // 3
    play_x = icon_x + icon_size // 2 - play_size // 4
    play_y = y_offset + icon_size // 2
    points = [
        (play_x, play_y - play_size // 2),
        (play_x + play_size // 2, play_y),
        (play_x, play_y + play_size // 2)
    ]
    draw.polygon(points, fill=(255, 255, 255))
    
    y_offset += icon_size + padding * 2
    
    # Block rate
    rate_text = "80%+"
    bbox = draw.textbbox((0, 0), rate_text, font=header_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        rate_text,
        fill=SUCCESS_COLOR,
        font=header_font
    )
    
    y_offset += header_size + padding // 2
    
    subtitle = "のYouTube広告をブロック"
    bbox = draw.textbbox((0, 0), subtitle, font=medium_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        subtitle,
        fill=text_color,
        font=medium_font
    )
    
    y_offset += medium_text_size + padding * 2
    
    # Features
    features = [
        "✓ 動画再生前の広告をスキップ",
        "✓ 動画途中の広告を削除",
        "✓ バナー広告を非表示",
        "✓ 快適な視聴体験"
    ]
    
    for feature in features:
        draw.text(
            (padding * 2, y_offset),
            feature,
            fill=SUCCESS_COLOR,
            font=small_font
        )
        y_offset += small_text_size + padding // 2
    
    # Navigation bar
    draw_navigation_bar(draw, width, height, dark_mode)
    
    return img

def create_custom_rules_screen(width, height, dark_mode=False):
    """Create custom rules screen."""
    bg_color = DARK_BACKGROUND if dark_mode else BACKGROUND_COLOR
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    card_color = CARD_COLOR_DARK if dark_mode else CARD_COLOR
    
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    # Font sizes
    title_size = width // 20
    medium_text_size = width // 25
    small_text_size = width // 30
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_size)
        medium_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", medium_text_size)
        small_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", small_text_size)
        code_font = ImageFont.truetype("/System/Library/Fonts/Menlo.ttc", small_text_size)
    except:
        title_font = ImageFont.load_default()
        medium_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
        code_font = ImageFont.load_default()
    
    padding = width // 20
    y_offset = height // 10
    
    # Draw status bar
    draw_status_bar(draw, width, 0, dark_mode)
    
    # Title
    title = "カスタムフィルター"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        title,
        fill=text_color,
        font=title_font
    )
    
    y_offset += title_size + padding
    
    # Add button
    button_height = height // 15
    draw.rounded_rectangle(
        [(padding, y_offset), (width - padding, y_offset + button_height)],
        radius=15,
        fill=PRIMARY_COLOR
    )
    
    add_text = "+ 新しいルールを追加"
    bbox = draw.textbbox((0, 0), add_text, font=medium_font)
    text_width = bbox[2] - bbox[0]
    text_height = bbox[3] - bbox[1]
    draw.text(
        ((width - text_width) // 2, y_offset + (button_height - text_height) // 2),
        add_text,
        fill=(255, 255, 255),
        font=medium_font
    )
    
    y_offset += button_height + padding
    
    # Rule examples
    rules = [
        ("||ads.example.com^", "広告ドメインをブロック", True),
        ("@@||safe-site.com^", "特定サイトを許可", True),
        ("##.banner-ad", "要素を非表示", True),
        ("||tracking.*/analytics/*", "トラッキングをブロック", False)
    ]
    
    card_height = height // 12
    card_spacing = padding // 2
    
    for rule, description, is_active in rules:
        # Draw card
        draw.rounded_rectangle(
            [(padding, y_offset), (width - padding, y_offset + card_height)],
            radius=15,
            fill=card_color
        )
        
        # Toggle switch
        switch_width = width // 10
        switch_height = card_height // 3
        switch_x = width - padding * 2 - switch_width
        switch_y = y_offset + (card_height - switch_height) // 2
        
        switch_color = SUCCESS_COLOR if is_active else (200, 200, 200)
        draw.rounded_rectangle(
            [(switch_x, switch_y), (switch_x + switch_width, switch_y + switch_height)],
            radius=switch_height // 2,
            fill=switch_color
        )
        
        # Switch knob
        knob_size = switch_height - 4
        knob_x = switch_x + (switch_width - knob_size - 2) if is_active else switch_x + 2
        draw.ellipse(
            [(knob_x, switch_y + 2), (knob_x + knob_size, switch_y + 2 + knob_size)],
            fill=(255, 255, 255)
        )
        
        # Rule text
        draw.text(
            (padding * 2, y_offset + card_height // 4),
            rule,
            fill=text_color,
            font=code_font
        )
        
        # Description
        draw.text(
            (padding * 2, y_offset + card_height // 2),
            description,
            fill=(150, 150, 150) if not dark_mode else (180, 180, 180),
            font=small_font
        )
        
        y_offset += card_height + card_spacing
    
    # Navigation bar
    draw_navigation_bar(draw, width, height, dark_mode)
    
    return img

def create_privacy_screen(width, height, dark_mode=False):
    """Create privacy screen."""
    bg_color = DARK_BACKGROUND if dark_mode else BACKGROUND_COLOR
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    card_color = CARD_COLOR_DARK if dark_mode else CARD_COLOR
    
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    # Font sizes
    title_size = width // 20
    header_size = width // 15
    medium_text_size = width // 25
    small_text_size = width // 30
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_size)
        header_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", header_size)
        medium_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", medium_text_size)
        small_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", small_text_size)
    except:
        title_font = ImageFont.load_default()
        header_font = ImageFont.load_default()
        medium_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
    
    padding = width // 20
    y_offset = height // 10
    
    # Draw status bar
    draw_status_bar(draw, width, 0, dark_mode)
    
    # Title
    title = "プライバシー優先"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        title,
        fill=text_color,
        font=title_font
    )
    
    y_offset += title_size + padding * 2
    
    # Shield icon
    shield_size = width // 4
    shield_x = (width - shield_size) // 2
    
    # Draw shield
    points = [
        (shield_x + shield_size // 2, y_offset),
        (shield_x + shield_size, y_offset + shield_size // 3),
        (shield_x + shield_size, y_offset + shield_size * 2 // 3),
        (shield_x + shield_size // 2, y_offset + shield_size),
        (shield_x, y_offset + shield_size * 2 // 3),
        (shield_x, y_offset + shield_size // 3)
    ]
    draw.polygon(points, fill=SUCCESS_COLOR)
    
    # Check mark in shield
    check_size = shield_size // 3
    check_x = shield_x + shield_size // 2
    check_y = y_offset + shield_size // 2
    draw.line(
        [(check_x - check_size//3, check_y),
         (check_x - check_size//6, check_y + check_size//4),
         (check_x + check_size//3, check_y - check_size//4)],
        fill=(255, 255, 255),
        width=width//40
    )
    
    y_offset += shield_size + padding * 2
    
    # Privacy features
    features = [
        ("🔒", "すべての処理はデバイス上で実行"),
        ("🚫", "個人情報の収集なし"),
        ("📊", "使用統計は匿名化"),
        ("🌐", "外部サーバーへの接続なし"),
        ("📖", "オープンソースで透明性を確保")
    ]
    
    card_height = height // 10
    
    for emoji, text in features:
        # Draw card
        draw.rounded_rectangle(
            [(padding, y_offset), (width - padding, y_offset + card_height)],
            radius=15,
            fill=card_color
        )
        
        # Emoji
        draw.text(
            (padding * 2, y_offset + (card_height - medium_text_size) // 2),
            emoji,
            fill=text_color,
            font=medium_font
        )
        
        # Text
        draw.text(
            (padding * 4, y_offset + (card_height - small_text_size) // 2),
            text,
            fill=text_color,
            font=small_font
        )
        
        y_offset += card_height + padding // 2
    
    # Navigation bar
    draw_navigation_bar(draw, width, height, dark_mode)
    
    return img

def create_performance_screen(width, height, dark_mode=False):
    """Create performance monitoring screen."""
    bg_color = DARK_BACKGROUND if dark_mode else BACKGROUND_COLOR
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    card_color = CARD_COLOR_DARK if dark_mode else CARD_COLOR
    
    img = Image.new('RGB', (width, height), bg_color)
    draw = ImageDraw.Draw(img)
    
    # Font sizes
    title_size = width // 20
    large_text_size = width // 12
    medium_text_size = width // 25
    small_text_size = width // 30
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", title_size)
        large_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", large_text_size)
        medium_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", medium_text_size)
        small_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", small_text_size)
    except:
        title_font = ImageFont.load_default()
        large_font = ImageFont.load_default()
        medium_font = ImageFont.load_default()
        small_font = ImageFont.load_default()
    
    padding = width // 20
    y_offset = height // 10
    
    # Draw status bar
    draw_status_bar(draw, width, 0, dark_mode)
    
    # Title
    title = "パフォーマンス"
    bbox = draw.textbbox((0, 0), title, font=title_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset),
        title,
        fill=text_color,
        font=title_font
    )
    
    y_offset += title_size + padding
    
    # Memory usage card
    card_height = height // 6
    draw.rounded_rectangle(
        [(padding, y_offset), (width - padding, y_offset + card_height)],
        radius=20,
        fill=card_color
    )
    
    # Memory icon and value
    memory_text = "28 MB"
    bbox = draw.textbbox((0, 0), memory_text, font=large_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset + card_height // 4),
        memory_text,
        fill=SUCCESS_COLOR,
        font=large_font
    )
    
    memory_label = "メモリ使用量（目標: 30MB以下）"
    bbox = draw.textbbox((0, 0), memory_label, font=small_font)
    text_width = bbox[2] - bbox[0]
    draw.text(
        ((width - text_width) // 2, y_offset + card_height * 2 // 3),
        memory_label,
        fill=(150, 150, 150) if not dark_mode else (180, 180, 180),
        font=small_font
    )
    
    y_offset += card_height + padding
    
    # Performance metrics
    metrics = [
        ("CPU使用率", "1.2%", SUCCESS_COLOR),
        ("処理時間", "< 1ms", SUCCESS_COLOR),
        ("キャッシュヒット率", "98.5%", SUCCESS_COLOR),
        ("フィルター数", "125,432", PRIMARY_COLOR)
    ]
    
    card_height = height // 12
    
    for metric_name, metric_value, color in metrics:
        # Draw card
        draw.rounded_rectangle(
            [(padding, y_offset), (width - padding, y_offset + card_height)],
            radius=15,
            fill=card_color
        )
        
        # Metric name
        draw.text(
            (padding * 2, y_offset + (card_height - small_text_size) // 2),
            metric_name,
            fill=(150, 150, 150) if not dark_mode else (180, 180, 180),
            font=small_font
        )
        
        # Metric value
        bbox = draw.textbbox((0, 0), metric_value, font=medium_font)
        text_width = bbox[2] - bbox[0]
        draw.text(
            (width - padding * 2 - text_width, y_offset + (card_height - medium_text_size) // 2),
            metric_value,
            fill=color,
            font=medium_font
        )
        
        y_offset += card_height + padding // 2
    
    # Navigation bar
    draw_navigation_bar(draw, width, height, dark_mode)
    
    return img

def draw_status_bar(draw, width, y, dark_mode=False):
    """Draw iOS-style status bar."""
    text_color = TEXT_COLOR_DARK if dark_mode else TEXT_COLOR
    
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 14)
    except:
        font = ImageFont.load_default()
    
    # Time
    time = datetime.datetime.now().strftime("%H:%M")
    draw.text((20, y + 5), time, fill=text_color, font=font)
    
    # Battery and signal indicators (simplified)
    draw.text((width - 100, y + 5), "100% 🔋", fill=text_color, font=font)

def draw_navigation_bar(draw, width, height, dark_mode=False):
    """Draw bottom navigation bar."""
    nav_height = height // 12
    y = height - nav_height
    
    # Draw separator
    draw.line([(0, y), (width, y)], fill=(200, 200, 200) if not dark_mode else (60, 60, 60), width=1)
    
    # Navigation items
    items = ["ホーム", "統計", "設定"]
    item_width = width // len(items)
    
    try:
        font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", width // 40)
    except:
        font = ImageFont.load_default()
    
    for i, item in enumerate(items):
        x = i * item_width + item_width // 2
        bbox = draw.textbbox((0, 0), item, font=font)
        text_width = bbox[2] - bbox[0]
        draw.text(
            (x - text_width // 2, y + nav_height // 3),
            item,
            fill=PRIMARY_COLOR if i == 0 else (150, 150, 150),
            font=font
        )

def generate_app_icons():
    """Generate app icons with better design."""
    sizes = {
        "ios": 1024,
        "android": 512
    }
    
    for platform, size in sizes.items():
        img = Image.new('RGB', (size, size), PRIMARY_COLOR)
        draw = ImageDraw.Draw(img)
        
        # Draw shield shape
        shield_size = size * 0.6
        shield_x = (size - shield_size) // 2
        shield_y = size * 0.2
        
        points = [
            (size // 2, shield_y),
            (shield_x + shield_size, shield_y + shield_size * 0.3),
            (shield_x + shield_size, shield_y + shield_size * 0.6),
            (size // 2, shield_y + shield_size),
            (shield_x, shield_y + shield_size * 0.6),
            (shield_x, shield_y + shield_size * 0.3)
        ]
        draw.polygon(points, fill=(255, 255, 255))
        
        # Draw "AB" text
        try:
            font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", int(size * 0.25))
        except:
            font = ImageFont.load_default()
        
        text = "AB"
        bbox = draw.textbbox((0, 0), text, font=font)
        text_width = bbox[2] - bbox[0]
        text_height = bbox[3] - bbox[1]
        
        draw.text(
            ((size - text_width) // 2, (size - text_height) // 2),
            text,
            fill=PRIMARY_COLOR,
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
    """Generate Android feature graphic with better design."""
    img = Image.new('RGB', (1024, 500), PRIMARY_COLOR)
    draw = ImageDraw.Draw(img)
    
    try:
        title_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 80)
        subtitle_font = ImageFont.truetype("/System/Library/Fonts/Helvetica.ttc", 40)
    except:
        title_font = ImageFont.load_default()
        subtitle_font = ImageFont.load_default()
    
    # Draw shield
    shield_size = 200
    shield_x = 100
    shield_y = 150
    
    points = [
        (shield_x + shield_size // 2, shield_y),
        (shield_x + shield_size, shield_y + shield_size * 0.3),
        (shield_x + shield_size, shield_y + shield_size * 0.6),
        (shield_x + shield_size // 2, shield_y + shield_size),
        (shield_x, shield_y + shield_size * 0.6),
        (shield_x, shield_y + shield_size * 0.3)
    ]
    draw.polygon(points, fill=(255, 255, 255, 128))
    
    # Draw title
    title = "AdBlock"
    draw.text(
        (400, 180),
        title,
        fill=(255, 255, 255),
        font=title_font
    )
    
    # Draw subtitle
    subtitle = "システム全体の広告ブロッカー"
    draw.text(
        (400, 280),
        subtitle,
        fill=(255, 255, 255, 200),
        font=subtitle_font
    )
    
    path = Path("assets/app-store/android/graphics/feature-graphic.png")
    path.parent.mkdir(parents=True, exist_ok=True)
    img.save(path, "PNG")
    print(f"Generated {path}")

def main():
    """Generate all app screenshots."""
    print("Generating app screenshots...")
    
    # Screenshot generators
    screenshot_funcs = [
        ("main", create_main_screen),
        ("youtube", create_youtube_screen),
        ("custom_rules", create_custom_rules_screen),
        ("privacy", create_privacy_screen),
        ("performance", create_performance_screen)
    ]
    
    # Generate iOS screenshots
    for size_name, (width, height) in IOS_SIZES.items():
        print(f"\nGenerating iOS {size_name} screenshots...")
        for i, (name, func) in enumerate(screenshot_funcs):
            # Generate both light and dark mode for first screenshot
            if i == 0:
                img = func(width, height, dark_mode=False)
                path = Path(f"assets/app-store/ios/screenshots/{size_name}/screenshot_{i+1}_light.png")
                path.parent.mkdir(parents=True, exist_ok=True)
                img.save(path, "PNG")
                print(f"  Generated {path}")
                
                img = func(width, height, dark_mode=True)
                path = Path(f"assets/app-store/ios/screenshots/{size_name}/screenshot_{i+1}_dark.png")
                img.save(path, "PNG")
                print(f"  Generated {path}")
            else:
                img = func(width, height, dark_mode=False)
                path = Path(f"assets/app-store/ios/screenshots/{size_name}/screenshot_{i+1}.png")
                path.parent.mkdir(parents=True, exist_ok=True)
                img.save(path, "PNG")
                print(f"  Generated {path}")
    
    # Generate Android screenshots
    for device_type, (width, height) in ANDROID_SIZES.items():
        print(f"\nGenerating Android {device_type} screenshots...")
        for i, (name, func) in enumerate(screenshot_funcs):
            img = func(width, height, dark_mode=False)
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
    
    print("\n✅ All app screenshots generated successfully!")

if __name__ == "__main__":
    main()