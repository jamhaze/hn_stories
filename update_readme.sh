#!/bin/sh
echo "# hn_stories" > README.md
echo "\`\`\`" >> README.md
$1 --help  >> README.md
echo "\`\`\`" >> README.md
