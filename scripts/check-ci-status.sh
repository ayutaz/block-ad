#!/bin/bash

# Script to check CI status using GitHub CLI

echo "🔍 Checking CI/CD Status for block-ad..."
echo "========================================"

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed."
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check workflow runs
echo -e "\n📊 Recent Workflow Runs:"
gh run list --repo ayutaz/block-ad --limit 10

echo -e "\n🏗️ Workflow Status:"
gh workflow list --repo ayutaz/block-ad

echo -e "\n📦 Latest Releases:"
gh release list --repo ayutaz/block-ad --limit 5

echo -e "\n💡 To trigger a workflow manually:"
echo "gh workflow run <workflow-name> --repo ayutaz/block-ad"

echo -e "\n📱 To download latest Android APK:"
echo "gh release download android-latest --repo ayutaz/block-ad"