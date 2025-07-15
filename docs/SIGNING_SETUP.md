# Code Signing Setup Guide

This guide explains how to set up code signing for releasing the AdBlock app to the App Store and Google Play Store.

## iOS Code Signing

### Prerequisites
- Apple Developer account ($99/year)
- Xcode with valid developer certificates
- App IDs created in Apple Developer Portal

### Required Identifiers
1. **Main App Bundle ID**: `com.adblock.app`
2. **Network Extension Bundle ID**: `com.adblock.app.networkextension`

### Steps

1. **Create App IDs in Apple Developer Portal**
   ```
   - Sign in to developer.apple.com
   - Navigate to Certificates, Identifiers & Profiles
   - Create two App IDs:
     * com.adblock.app (Main app)
     * com.adblock.app.networkextension (Network Extension)
   - Enable required capabilities:
     * Main app: Network Extensions
     * Extension: Packet Tunnel Provider
   ```

2. **Create Provisioning Profiles**
   ```
   - Create App Store distribution profiles for both App IDs
   - Download and install the profiles in Xcode
   ```

3. **Configure GitHub Secrets**
   Add the following secrets to your GitHub repository:
   ```
   - IOS_TEAM_ID: Your 10-character Team ID
   - IOS_SIGNING_CERTIFICATE_P12_DATA: Base64 encoded .p12 certificate
   - IOS_SIGNING_CERTIFICATE_PASSWORD: Password for the .p12 file
   - IOS_PROVISIONING_PROFILE_DATA: Base64 encoded provisioning profile
   - IOS_PROVISIONING_PROFILE_DATA_EXT: Base64 encoded extension profile
   ```

4. **Export Certificate**
   ```bash
   # Export your certificate from Keychain
   security export -t identities -f pkcs12 -k ~/Library/Keychains/login.keychain -P "your-password" -o certificate.p12
   
   # Convert to base64
   base64 -i certificate.p12 -o certificate.p12.base64
   ```

5. **Export Provisioning Profiles**
   ```bash
   # Find your profiles
   cd ~/Library/MobileDevice/Provisioning\ Profiles/
   
   # Convert to base64
   base64 -i "YourProfile.mobileprovision" -o profile.base64
   ```

## Android Code Signing

### Prerequisites
- Java keytool or Android Studio
- Google Play Console account ($25 one-time fee)

### Steps

1. **Generate Signing Key**
   ```bash
   keytool -genkey -v -keystore adblock-release.keystore \
     -alias adblock -keyalg RSA -keysize 2048 -validity 10000
   ```

2. **Configure GitHub Secrets**
   Add the following secrets:
   ```
   - ANDROID_SIGNING_KEY: Base64 encoded keystore file
   - ANDROID_KEY_ALIAS: Key alias (e.g., "adblock")
   - ANDROID_KEY_PASSWORD: Key password
   - ANDROID_STORE_PASSWORD: Keystore password
   ```

3. **Convert Keystore to Base64**
   ```bash
   base64 -i adblock-release.keystore -o keystore.base64
   ```

## Local Testing

### iOS
1. Open the project in Xcode
2. Select your development team in Signing & Capabilities
3. Build and run on a real device

### Android
1. Create `local.properties` file:
   ```properties
   storeFile=/path/to/your/keystore
   storePassword=your-store-password
   keyAlias=your-key-alias
   keyPassword=your-key-password
   ```
2. Build signed APK:
   ```bash
   cd android
   ./gradlew assembleRelease
   ```

## Security Notes
- Never commit signing certificates or passwords to the repository
- Use GitHub Secrets for all sensitive information
- Rotate certificates periodically
- Keep backup of all certificates and passwords in a secure location

## Troubleshooting

### iOS Common Issues
- **"No Team Found in Archive"**: Ensure DEVELOPMENT_TEAM is set in GitHub Secrets
- **"Provisioning profile doesn't match"**: Update profiles in Apple Developer Portal
- **"Certificate not found"**: Re-export and update certificate in GitHub Secrets

### Android Common Issues
- **"Keystore file not found"**: Check base64 encoding and GitHub Secret name
- **"Wrong password"**: Ensure passwords match those used when creating keystore
- **"Key alias not found"**: Verify the alias name matches exactly