# CVKG Monitoring & Alerts

The CVKG server exports metrics in Prometheus format at the `/metrics` endpoint.

## Key Metrics

- `http_requests_total`: Total number of HTTP requests.
- `http_request_duration_seconds`: Request duration histogram.
- `governor_rate_limited_total`: Number of requests rate limited by the governor middleware.
- `axum_serve_error`: Errors during serving.

## Recommended Alerts (Prometheus/Alertmanager)

### 1. Server Down
```yaml
alert: CvkgServerDown
expr: up{job="cvkg"} == 0
for: 2m
labels:
  severity: critical
annotations:
  summary: "CVKG Server is down"
  description: "The CVKG server at {{ $labels.instance }} has been down for more than 2 minutes."
```

### 2. High Error Rate
```yaml
alert: CvkgHighErrorRate
expr: sum(rate(http_requests_total{status=~"5.."}[5m])) / sum(rate(http_requests_total[5m])) > 0.05
for: 5m
labels:
  severity: warning
annotations:
  summary: "High HTTP 5xx error rate"
  description: "CVKG server is returning > 5% errors for more than 5 minutes."
```

### 3. Slow Responses
```yaml
alert: CvkgSlowResponses
expr: histogram_quantile(0.95, sum(rate(http_request_duration_seconds_bucket[5m])) by (le)) > 2
for: 5m
labels:
  severity: warning
annotations:
  summary: "Slow HTTP responses"
  description: "95th percentile of response time is > 2s for more than 5 minutes."
```

### 4. Intense Rate Limiting
```yaml
alert: CvkgIntenseRateLimiting
expr: rate(governor_rate_limited_total[5m]) > 10
for: 5m
labels:
  severity: info
annotations:
  summary: "Significant rate limiting active"
  description: "Server is rate limiting more than 10 requests/sec on average."
```
