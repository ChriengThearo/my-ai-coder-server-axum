#!/bin/bash

# OneClick AI Backend - Railway Deployment Script
# This script helps deploy to Railway with proper configuration

set -e  # Exit on error

echo "🚂 OneClick AI Backend - Railway Deployment"
echo "==========================================="
echo ""

# Check if Railway CLI is installed
if ! command -v railway &> /dev/null; then
    echo "❌ Railway CLI not found!"
    echo ""
    echo "Install it with:"
    echo "  brew install railway"
    echo "  OR"
    echo "  npm install -g @railway/cli"
    echo ""
    exit 1
fi

echo "✅ Railway CLI found"
echo ""

# Check if logged in
if ! railway whoami &> /dev/null; then
    echo "🔐 Not logged in to Railway. Logging in..."
    railway login
else
    echo "✅ Logged in to Railway"
fi

echo ""

# Check if project is linked
if ! railway status &> /dev/null; then
    echo "📦 Project not linked. Initializing..."
    railway init
else
    echo "✅ Project linked"
fi

echo ""
echo "🔧 Setting environment variables..."
echo ""

# Prompt for LLM API key if not set
read -p "Enter your LLM API key (OpenAI/Anthropic): " LLM_API_KEY
if [ -z "$LLM_API_KEY" ]; then
    echo "❌ LLM API key is required!"
    exit 1
fi

# Set environment variables
railway variables set LLM_API_KEY="$LLM_API_KEY"
railway variables set LLM_BASE_URL="https://api.openai.com/v1"
railway variables set LLM_MODEL="gpt-4o"
railway variables set RUST_LOG="info"

echo ""
echo "✅ Environment variables set"
echo ""

# Build locally to verify
echo "🔨 Testing build locally..."
if cargo build --release; then
    echo "✅ Local build successful"
else
    echo "❌ Local build failed. Fix errors before deploying."
    exit 1
fi

echo ""
echo "🚀 Deploying to Railway..."
railway up

echo ""
echo "✅ Deployment complete!"
echo ""

# Get the URL
echo "🌍 Getting your deployment URL..."
railway domain

echo ""
echo "📝 Next steps:"
echo "1. Test your deployment with: curl \$(railway domain)/health"
echo "2. Update your VSCode extension with the Railway URL"
echo "3. Test the auth endpoints"
echo ""
echo "📊 Monitor logs with: railway logs -f"
echo "📈 Check status with: railway status"
echo ""
echo "🎉 Happy deploying!"
