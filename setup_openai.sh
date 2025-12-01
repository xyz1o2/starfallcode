#!/bin/bash

echo "=== OpenAI Configuration Setup ==="
echo ""
echo "To use OpenAI API, you need an API key."
echo "Get your key from: https://platform.openai.com/api-keys"
echo ""

# 检查 .env 文件是否存在
if [[ ! -f .env ]]; then
    echo "Creating .env file..."
    cp .env.example .env
fi

echo ""
echo "Please do one of the following:"
echo ""
echo "Option 1: Edit .env file directly"
echo "  nano /mnt/g/ai-workspace/starfellcode/.env"
echo "  Then replace 'your_openai_api_key_here' with your actual API key"
echo ""
echo "Option 2: Export environment variable"
echo "  export OPENAI_API_KEY='sk-...'"
echo "  export LLM_PROVIDER=openai"
echo ""
echo "Option 3: Use this script (uncomment and edit)"
echo ""
echo "# Edit the line below with your actual API key, then uncomment:"
echo "# export OPENAI_API_KEY='sk-your-api-key-here'"
echo "# export LLM_PROVIDER=openai"
echo ""
