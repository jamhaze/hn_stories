#!/bin/sh
echo "# hn_stories" > README.md
echo '<a href="https://www.hannahilea.com/blog/houseplant-programming"><img alt="Static Badge" src="https://img.shields.io/badge/%F0%9F%AA%B4%20Houseplant%20-x?style=flat&amp;label=Project%20type&amp;color=1E1E1D"></a>' >> README.md 
echo "\`\`\`" >> README.md
$1 --help  >> README.md
echo "\`\`\`" >> README.md
