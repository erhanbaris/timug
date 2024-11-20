## Filters
### formatdatetime
Expecting **%Y-%m-%d %H:%M:%S** format and accept format argument as string.
Example usage:
{{ post.date | formatdatetime("%B %d, %Y") }}