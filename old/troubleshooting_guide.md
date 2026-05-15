# CVKG Troubleshooting Guide

## Common Issues and Solutions

### Issue: Example won't compile - missing feature flag

**Symptom:** Error about unresolved imports like `cvkg::render` or `SurtrRenderer`

**Solution:** Enable the appropriate feature flag:
```bash
# For GPU renderer examples:
cargo run --example shatter_demo -p cvkg --features gpu

# For native renderer examples:
cargo run --example interactive_demo -p cvkg-components --features native

# For web renderer examples:
cargo run --example web_demo -p cvkg-components --features web
```

### Issue: 