#!/bin/sh
echo "# hn_stories" > README.md
$1 --help | sed 's/$/<br>/' >> README.md
