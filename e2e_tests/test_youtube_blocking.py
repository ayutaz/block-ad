#!/usr/bin/env python3
"""
E2E test for YouTube ad blocking functionality
Tests actual blocking rate against YouTube ads
"""

import time
import requests
from selenium import webdriver
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC
from selenium.common.exceptions import TimeoutException

class YouTubeAdBlockTest:
    def __init__(self):
        self.ads_blocked = 0
        self.ads_shown = 0
        self.videos_tested = 0
        
    def setup_proxy(self):
        """Setup HTTP proxy to route through our VPN/ad blocker"""
        # This would be configured to use the actual VPN proxy
        proxy = {
            'http': 'http://localhost:8080',
            'https': 'http://localhost:8080'
        }
        return proxy
        
    def test_youtube_video(self, video_url):
        """Test if ads are blocked on a specific YouTube video"""
        options = webdriver.ChromeOptions()
        options.add_argument('--headless')
        options.add_argument('--no-sandbox')
        options.add_argument('--disable-dev-shm-usage')
        options.add_argument('--disable-gpu')
        
        # Skip proxy in CI environment
        import os
        if not os.getenv('CI'):
            options.add_argument('--proxy-server=http://localhost:8080')
        
        driver = webdriver.Chrome(options=options)
        try:
            driver.get(video_url)
            
            # Wait for video player to load
            wait = WebDriverWait(driver, 10)
            video_player = wait.until(
                EC.presence_of_element_located((By.ID, "movie_player"))
            )
            
            # Check for ad indicators
            ad_shown = False
            
            # Method 1: Check for skip ad button
            try:
                skip_button = driver.find_element(By.CLASS_NAME, "ytp-ad-skip-button")
                if skip_button:
                    ad_shown = True
                    self.ads_shown += 1
            except:
                pass
                
            # Method 2: Check for ad badge
            try:
                ad_badge = driver.find_element(By.CLASS_NAME, "ytp-ad-badge")
                if ad_badge:
                    ad_shown = True
                    self.ads_shown += 1
            except:
                pass
                
            # Method 3: Check for ad duration display
            try:
                ad_duration = driver.find_element(By.CLASS_NAME, "ytp-ad-duration-remaining")
                if ad_duration:
                    ad_shown = True
                    self.ads_shown += 1
            except:
                pass
                
            if not ad_shown:
                self.ads_blocked += 1
                
            self.videos_tested += 1
            
            # Watch for a few seconds to detect mid-roll ads
            time.sleep(10)
            
        finally:
            driver.quit()
            
    def test_ad_server_blocking(self):
        """Test if known ad servers are blocked"""
        ad_servers = [
            "https://doubleclick.net",
            "https://googleadservices.com",
            "https://googlesyndication.com",
            "https://youtube.com/api/stats/ads",
            "https://youtube.com/pagead",
            "https://googlevideo.com/ptracking",
            "https://static.doubleclick.net/instream/ad_status.js"
        ]
        
        blocked_count = 0
        proxy = self.setup_proxy()
        
        for server in ad_servers:
            try:
                response = requests.get(server, proxies=proxy, timeout=5)
                if response.status_code >= 400:
                    blocked_count += 1
            except (requests.exceptions.ConnectionError, requests.exceptions.Timeout):
                # Connection refused or timeout means blocked
                blocked_count += 1
                
        return blocked_count, len(ad_servers)
        
    def calculate_block_rate(self):
        """Calculate the overall ad block rate"""
        total_ads = self.ads_blocked + self.ads_shown
        if total_ads == 0:
            return 0
        return (self.ads_blocked / total_ads) * 100
        
    def run_full_test(self):
        """Run complete E2E test suite"""
        print("Starting YouTube Ad Block E2E Test...")
        
        # Test popular YouTube videos that typically have ads
        test_videos = [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",  # Popular video
            "https://www.youtube.com/watch?v=9bZkp7q19f0",  # Music video
            "https://www.youtube.com/watch?v=kJQP7kiw5Fk",  # Another popular video
        ]
        
        # Test video ad blocking
        print("\n1. Testing YouTube video ads...")
        for video in test_videos:
            print(f"   Testing: {video}")
            self.test_youtube_video(video)
            
        # Test ad server blocking
        print("\n2. Testing ad server blocking...")
        blocked, total = self.test_ad_server_blocking()
        print(f"   Blocked {blocked}/{total} ad servers")
        
        # Calculate results
        block_rate = self.calculate_block_rate()
        
        print("\n=== TEST RESULTS ===")
        print(f"Videos tested: {self.videos_tested}")
        print(f"Ads blocked: {self.ads_blocked}")
        print(f"Ads shown: {self.ads_shown}")
        print(f"Block rate: {block_rate:.1f}%")
        print(f"Ad servers blocked: {blocked}/{total}")
        
        # Check if we meet the 80% target
        if block_rate >= 80:
            print("\n✅ SUCCESS: Block rate exceeds 80% target!")
            return True
        else:
            print(f"\n❌ FAILED: Block rate {block_rate:.1f}% is below 80% target")
            return False

if __name__ == "__main__":
    test = YouTubeAdBlockTest()
    test.run_full_test()