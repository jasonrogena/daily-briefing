# input-home-assistant-metrics

Reads sensor state history from Home Assistant and compares two consecutive time windows of equal length. Supports both cumulative sensors (e.g. energy counters) and gauge sensors (e.g. battery level, temperature).

## Configuration

```toml
[[inputs]]
name = "home_health"
type = "home_assistant_metrics"
url = "http://homeassistant.local:8123"
token_env = "HA_TOKEN"
hours = 24

[[inputs.entities]]
id = "sensor.grid_consumption_energy"
label = "Grid Consumption"

[[inputs.entities]]
id = "sensor.solar_production_energy"
label = "Solar Production"

[[inputs.entities]]
id = "sensor.hue_motion_sensor_1_battery"
label = "Motion Sensor 1 Battery"
kind = "gauge"
```

### `[[inputs]]` fields

| Field | Default | Description |
|---|---|---|
| `name` | — | Unique name for this input, used in logs. |
| `url` | — | Home Assistant base URL (e.g. `http://homeassistant.local:8123`). |
| `token_env` | — | Name of the env var holding a HA long-lived access token. |
| `hours` | `24` | Length of each comparison window in hours. History covering `2 × hours` is fetched to enable comparison. |
| `entities` | — | List of entity configs (see below). |

### `[[inputs.entities]]` fields

| Field | Default | Description |
|---|---|---|
| `id` | — | Home Assistant entity ID (e.g. `sensor.grid_consumption_energy`). |
| `label` | entity ID | Human-readable label used in the report sent to the processor. |
| `kind` | `"cumulative"` | Sensor type — see below. |

### Entity kinds

| Kind | Use for | Behaviour |
|---|---|---|
| `"cumulative"` | Energy counters, ever-increasing totals | Reports the delta (change) over each window and the percentage change between windows. |
| `"gauge"` | Battery level, temperature, any point-in-time reading | Reports the latest reading within the current window. Falls back to the previous window if the sensor has not reported recently. |
