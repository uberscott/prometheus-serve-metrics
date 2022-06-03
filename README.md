# PROMETHEUS PULL METRICS
This crate serves metrics for `opentelemetry-prometheus` 

You can setup prometheus to poll the metrics on a timer.

## START IT UP

All you have to do is add the crate and invoke:

```rust
prometheus_serve_metrics::init();
```

That's it!