# Code Signing Guide

This guide explains how to set up code signing for Android and iOS releases.

## Android Signing

### Generate a Keystore

1. Create a new keystore:
```bash
keytool -genkey -v -keystore adblock-release.jks \
  -keyalg RSA -keysize 2048 -validity 10000 \
  -alias adblock
```

2. Keep the keystore file secure and never commit it to the repository.

### Configure GitHub Secrets

Add the following secrets to your GitHub repository:

1. **ANDROID_SIGNING_KEY**: Base64 encoded keystore
   ```bash
   base64 -i adblock-release.jks | pbcopy  # macOS
   base64 adblock-release.jks | xclip -selection clipboard  # Linux
   ```

2. **ANDROID_KEY_ALIAS**: The alias used when creating the keystore (e.g., `adblock`)

3. **ANDROID_KEYSTORE_PASSWORD**: The keystore password

4. **ANDROID_KEY_PASSWORD**: The key password (usually same as keystore password)

### Local Signing

For local builds, create a `keystore.properties` file in the `android` directory:

```properties
storePassword=your_keystore_password
keyPassword=your_key_password
keyAlias=adblock
storeFile=/path/to/adblock-release.jks
```

Add this file to `.gitignore` to keep it secure.

## iOS Signing

### Prerequisites

1. Apple Developer account
2. Valid distribution certificate
3. App Store provisioning profile

### Generate Certificates

1. In Xcode, go to Preferences > Accounts
2. Select your team and click "Manage Certificates"
3. Create a new distribution certificate if needed

### Export Certificate

1. Open Keychain Access
2. Find your distribution certificate
3. Export as .p12 file with a password

### Configure GitHub Secrets

1. **IOS_CERTIFICATE_BASE64**: Base64 encoded .p12 certificate
   ```bash
   base64 -i Certificates.p12 | pbcopy
   ```

2. **IOS_CERTIFICATE_PASSWORD**: Password for the .p12 file

3. **IOS_TEAM_ID**: Your Apple Developer Team ID

4. **IOS_SIGN_IDENTITY**: Certificate name (e.g., "iPhone Distribution: Your Name")

5. **IOS_PROVISION_PROFILE_BASE64**: Base64 encoded provisioning profile
   ```bash
   base64 -i AdBlock.mobileprovision | pbcopy
   ```

6. **IOS_PROVISION_PROFILE_NAME**: Name of the provisioning profile

### App Store Connect API (Optional)

For automatic TestFlight uploads:

1. **APPSTORE_CONNECT_API_KEY**: API key content
2. **APPSTORE_CONNECT_API_KEY_ID**: Key ID
3. **APPSTORE_CONNECT_ISSUER_ID**: Issuer ID

## Google Play Store Upload

For automatic Play Store uploads, add:

1. **GOOGLE_PLAY_SERVICE_ACCOUNT_JSON**: Service account JSON key

Create a service account:
1. Go to Google Play Console > Setup > API access
2. Create a new service account
3. Grant "Release manager" permissions
4. Download the JSON key

## Security Best Practices

1. **Never commit signing files** to the repository
2. **Use strong passwords** for all certificates and keystores
3. **Rotate certificates** before they expire
4. **Limit access** to signing secrets
5. **Use separate keys** for debug and release builds
6. **Back up** your certificates and keystores securely

## Troubleshooting

### Android Issues

**Error: "No signing configuration found"**
- Ensure all Android secrets are properly set in GitHub
- Check that the base64 encoding is correct

**Error: "Keystore was tampered with"**
- The keystore password is incorrect
- The keystore file is corrupted

### iOS Issues

**Error: "No identity found"**
- Certificate is not properly imported
- Certificate has expired
- Wrong certificate type (need distribution certificate)

**Error: "Provisioning profile doesn't match"**
- Profile doesn't include the certificate
- Profile has expired
- Wrong app identifier

## Release Process

1. **Create a tag**: `git tag v1.0.0 && git push origin v1.0.0`
2. **GitHub Actions** will automatically:
   - Build signed APK and AAB for Android
   - Build IPA for iOS
   - Create a draft release
3. **Review and publish** the release on GitHub

For manual releases, use the workflow dispatch feature in GitHub Actions.