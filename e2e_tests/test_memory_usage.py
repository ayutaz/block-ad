#!/usr/bin/env python3
"""
E2E test for memory usage
Ensures the app stays under 30MB memory limit
"""

import psutil
import subprocess
import time
import os

class MemoryUsageTest:
    def __init__(self):
        self.max_memory_mb = 30
        self.measurements = []
        
    def get_process_memory(self, process_name):
        """Get memory usage of a process by name"""
        for proc in psutil.process_iter(['pid', 'name', 'memory_info']):
            if process_name in proc.info['name']:
                memory_mb = proc.info['memory_info'].rss / 1024 / 1024
                return memory_mb
        return None
        
    def test_android_memory(self):
        """Test Android app memory usage"""
        print("Testing Android memory usage...")
        
        # Start the app (assuming ADB is configured)
        subprocess.run(["adb", "shell", "am", "start", "-n", "com.adblock/.MainActivity"])
        time.sleep(5)  # Wait for app to start
        
        # Monitor memory for 60 seconds
        for i in range(60):
            # Get memory usage via ADB
            result = subprocess.run(
                ["adb", "shell", "dumpsys", "meminfo", "com.adblock"],
                capture_output=True,
                text=True
            )
            
            # Parse total PSS from output
            for line in result.stdout.split('\n'):
                if 'TOTAL' in line and 'PSS' in line:
                    parts = line.split()
                    for j, part in enumerate(parts):
                        if part == 'TOTAL':
                            try:
                                memory_kb = int(parts[j+1])
                                memory_mb = memory_kb / 1024
                                self.measurements.append(memory_mb)
                                print(f"   Memory at {i}s: {memory_mb:.2f} MB")
                                break
                            except:
                                pass
                                
            time.sleep(1)
            
    def test_ios_memory(self):
        """Test iOS app memory usage"""
        print("Testing iOS memory usage...")
        
        # This would use xcrun instruments or similar
        # For simulation purposes:
        process_name = "AdBlock"
        
        for i in range(60):
            memory = self.get_process_memory(process_name)
            if memory:
                self.measurements.append(memory)
                print(f"   Memory at {i}s: {memory:.2f} MB")
            time.sleep(1)
            
    def analyze_results(self):
        """Analyze memory usage results"""
        if not self.measurements:
            print("❌ No memory measurements collected")
            return False
            
        avg_memory = sum(self.measurements) / len(self.measurements)
        max_memory = max(self.measurements)
        min_memory = min(self.measurements)
        
        print("\n=== MEMORY USAGE RESULTS ===")
        print(f"Average memory: {avg_memory:.2f} MB")
        print(f"Maximum memory: {max_memory:.2f} MB")
        print(f"Minimum memory: {min_memory:.2f} MB")
        print(f"Target: < {self.max_memory_mb} MB")
        
        if max_memory <= self.max_memory_mb:
            print(f"\n✅ SUCCESS: Maximum memory {max_memory:.2f} MB is under {self.max_memory_mb} MB limit!")
            return True
        else:
            print(f"\n❌ FAILED: Maximum memory {max_memory:.2f} MB exceeds {self.max_memory_mb} MB limit!")
            return False
            
    def run_test(self, platform="android"):
        """Run memory usage test"""
        print(f"Starting memory usage test for {platform}...")
        
        if platform == "android":
            self.test_android_memory()
        elif platform == "ios":
            self.test_ios_memory()
            
        return self.analyze_results()

if __name__ == "__main__":
    test = MemoryUsageTest()
    
    # Test Android
    test.run_test("android")
    
    # Reset for iOS test
    test.measurements = []
    
    # Test iOS (if on macOS)
    if os.uname().sysname == "Darwin":
        test.run_test("ios")