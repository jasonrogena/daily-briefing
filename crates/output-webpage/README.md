# output-webpage

Serves the briefing as a rendered HTML page via the shared web server. The briefing content is treated as Markdown and rendered to HTML on each pipeline run.

Requires a `[webserver]` section in the config.

## Configuration

```toml
[[outputs]]
name = "webpage"
type = "webpage"
endpoint = "/"
title = "Daily Briefing"
```

| Field | Description |
|---|---|
| `name` | Unique name for this output, used in logs. |
| `endpoint` | URL path served by the shared web server (e.g. `"/"`). |
| `title` | Page title shown in the browser tab and as the page heading. |
