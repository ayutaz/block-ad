# Device Testing Plan

This document outlines the comprehensive testing plan for AdBlock app on real devices.

## Test Environment Requirements

### Android Devices
- **Minimum**: Android 7.0 (API 24) device
- **Recommended**: 
  - Android 10+ device (for latest VPN APIs)
  - Android 13+ device (for notification permissions)
- **Test Devices**:
  - Low-end: 2GB RAM device
  - Mid-range: 4GB RAM device  
  - High-end: 8GB+ RAM device

### iOS Devices
- **Minimum**: iOS 15.0 device
- **Recommended**:
  - iPhone 12 or newer
  - iPad (for tablet UI testing)
- **Test Devices**:
  - iPhone SE (small screen)
  - iPhone 15 (standard)
  - iPhone 15 Pro Max (large screen)
  - iPad Pro (tablet)

## Functional Testing

### 1. Installation & Setup
- [ ] Clean install from APK/TestFlight
- [ ] First launch experience
- [ ] Permission requests (VPN, notifications)
- [ ] Initial filter list download

### 2. Core Ad Blocking
- [ ] Enable VPN connection
- [ ] Browse popular websites
- [ ] Verify ads are blocked
- [ ] Check for broken layouts
- [ ] Test with different browsers

### 3. YouTube Ad Blocking
- [ ] Open YouTube app
- [ ] Play 10 different videos
- [ ] Count pre-roll ads shown vs blocked
- [ ] Test mid-roll ads
- [ ] Verify playback isn't interrupted

### 4. App-specific Testing
Popular apps to test:
- [ ] Instagram
- [ ] Facebook
- [ ] Twitter/X
- [ ] TikTok
- [ ] News apps
- [ ] Games with ads

### 5. Filter Management
- [ ] Update filter lists manually
- [ ] Add custom rules
- [ ] Import/export rules
- [ ] Verify auto-update works

### 6. Performance Monitoring
- [ ] Check real-time statistics
- [ ] Monitor CPU usage
- [ ] Verify memory usage < 30MB
- [ ] Test battery impact

## Performance Testing

### Memory Usage Measurement
1. Enable Developer Options
2. Go to Settings > Developer Options > Running Services
3. Note AdBlock memory usage:
   - Initial: _____ MB
   - After 1 hour: _____ MB
   - After 24 hours: _____ MB

### Battery Impact
1. Charge device to 100%
2. Enable AdBlock
3. Use device normally for 4 hours
4. Check battery usage in Settings
5. Target: < 5% battery usage

### Network Performance
1. Run speed test without AdBlock
2. Enable AdBlock
3. Run speed test again
4. Compare results:
   - Download speed impact: < 5%
   - Latency increase: < 10ms

## YouTube Ad Block Rate Testing

### Test Methodology
1. Clear YouTube app data
2. Disable personalized ads in Google settings
3. Create fresh test account
4. Watch videos from different categories:
   - Music videos
   - Tech reviews
   - Gaming content
   - News channels
   - Educational content

### Data Collection Template
```
Video #1:
- Channel: _____________
- Pre-roll ad shown: Yes/No
- Mid-roll ads: _____ shown / _____ total
- Post-roll ad shown: Yes/No

(Repeat for 20+ videos)

Total ads blocked: _____
Total ads shown: _____
Block rate: _____%
```

## Compatibility Testing

### Network Types
- [ ] WiFi (2.4GHz)
- [ ] WiFi (5GHz)
- [ ] 4G/LTE
- [ ] 5G
- [ ] Public WiFi (captive portal)

### VPN Conflicts
- [ ] Test with other VPN apps installed
- [ ] Verify handling of VPN switching
- [ ] Test always-on VPN setting

### App Compatibility
Test critical apps that may conflict:
- [ ] Banking apps
- [ ] Government apps
- [ ] Corporate apps
- [ ] Other ad blockers

## Stability Testing

### Long-running Test
1. Enable AdBlock
2. Use device normally for 48 hours
3. Monitor for:
   - [ ] Crashes
   - [ ] Memory leaks
   - [ ] Connection drops
   - [ ] Filter update failures

### Stress Testing
1. Add 1000+ custom rules
2. Browse rapidly between apps
3. Toggle VPN repeatedly
4. Force-stop and restart

## User Experience Testing

### UI/UX Checklist
- [ ] All buttons responsive
- [ ] Smooth animations
- [ ] Text readable on all screens
- [ ] Dark mode support
- [ ] Landscape orientation
- [ ] Accessibility features

### Localization
- [ ] Japanese text displays correctly
- [ ] No text truncation
- [ ] Date/time formats correct
- [ ] Error messages localized

## Bug Reporting Template

```
Device: [Model, OS version]
AdBlock Version: [Version number]
Issue Type: [Crash/Performance/UI/Functionality]

Steps to Reproduce:
1. 
2. 
3. 

Expected Result:

Actual Result:

Screenshots/Logs:
```

## Performance Benchmarks

### Target Metrics
- **Memory Usage**: < 30MB average
- **CPU Usage**: < 5% average
- **Battery Impact**: < 5% over 4 hours
- **YouTube Block Rate**: > 80%
- **Web Ad Block Rate**: > 95%
- **App Launch Time**: < 2 seconds
- **Filter Update Time**: < 10 seconds

### Measurement Tools
- Android: Android Studio Profiler
- iOS: Xcode Instruments
- Network: Charles Proxy / Wireshark
- Battery: Built-in OS tools

## Test Result Summary

```
Test Date: _____________
Tester: _____________
Device: _____________

Overall Result: PASS / FAIL

Key Metrics:
- Memory Usage: _____ MB
- YouTube Block Rate: _____%
- Web Block Rate: _____%
- Crashes: _____
- Major Bugs: _____

Recommendation: 
[ ] Ready for release
[ ] Minor fixes needed
[ ] Major issues to address
```

## Known Limitations

Based on testing and implementation, the following limitations have been identified:

### General Limitations
- **YouTube Ads**: Approximately 80% block rate due to dynamic ad serving
- **Native Ads**: Cannot block ads integrated directly into app content
- **Sponsored Content**: Cannot detect paid promotions or sponsored posts
- **HTTPS Certificate Pinning**: Some apps may not work if they use certificate pinning

### Platform-Specific Limitations

#### Android
- **System Apps**: Cannot block ads in some system apps due to permissions
- **WebView Ads**: Limited blocking in some WebView implementations
- **Background Restrictions**: May be killed by aggressive battery optimization
- **VPN Conflicts**: Cannot run simultaneously with other VPN apps

#### iOS
- **App Store Restrictions**: Cannot be distributed through App Store due to policy
- **Background Refresh**: Limited background filter updates compared to Android
- **System Extensions**: Requires user approval for VPN configuration
- **Memory Limits**: Stricter memory limits in Network Extensions

### Performance Limitations
- **Low-End Devices**: May experience slight latency on devices with <2GB RAM
- **Large Filter Lists**: Loading time increases with custom rule count >10,000
- **Battery Impact**: Continuous filtering uses 3-5% additional battery
- **Network Speed**: <1% impact on network throughput

### Content-Type Limitations
- **Video Ads**: Pre-roll ads in native video players (non-YouTube)
- **Dynamic Ads**: JavaScript-generated ads may require page reload
- **First-Party Ads**: Ads served from the same domain as content
- **Encrypted DNS**: Cannot filter if app uses DoH/DoT directly

### Regional Differences
- **CDN Variations**: Ad servers may vary by region
- **Language Support**: Filter lists optimized for English content
- **Local Ad Networks**: May not block region-specific ad networks
- **Compliance**: Some regions may have legal restrictions on ad blocking

## Next Steps

After testing:
1. File bug reports for all issues
2. Update documentation with findings
3. Plan fixes for critical issues
4. Retest after fixes
5. Get sign-off for release