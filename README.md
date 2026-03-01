# hn_stories
<a href="https://www.hannahilea.com/blog/houseplant-programming"><img alt="Static Badge" src="https://img.shields.io/badge/%F0%9F%AA%B4%20Houseplant%20-x?style=flat&amp;label=Project%20type&amp;color=1E1E1D"></a>
```
This is a command line tool for retrieving the URLs of stories posted on https://news.ycombinator.com/

Usage: hn_stories [OPTIONS] <--category <CATEGORY>|--query <QUERY>>

Options:

  -c, --category <CATEGORY>  The category of the stories to fetch [possible values: new, top, best]
  -q, --query <QUERY>        The query to search for stories with
  -l, --limit <LIMIT>        Set the limit for the number of stories you wish to retrieve with above options [default: 30]
  -t, --time                 Display the time at which the story was posted
  -h, --help                 Print help
  -V, --version              Print version

