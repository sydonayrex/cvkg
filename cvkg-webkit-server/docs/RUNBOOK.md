# CVKG Server Runbook (Incident Response)

This document provides procedures for responding to common incidents with the CVKG server.

## Common Scenarios

### 1. Server Unresponsive (Health Check Fail)
**Symptoms**: `/health/liveness` returns 5xx or times out.
**Action**: 
1. Check process status: `systemctl status cvkg`.
2. Check logs: `journalctl -u cvkg -n 100`.
3. Restart process: `systemctl restart cvkg`.
4. Check if port is bound: `ss -tulpn | grep 3000`.

### 2. High Error Rate
**Symptoms**: Increase in 5xx responses in Prometheus metrics.
**Action**:
1. Check logs for "ERROR" level messages: `journalctl -u cvkg | grep ERROR`.
2. Verify if the `BuildOrchestrator` is failing (if applicable).
3. Check for resource exhaustion (OOM, Disk full).

### 3. Rate Limit Triggered Frequently
**Symptoms**: Users receiving 429 Too Many Requests.
**Action**:
1. Check if it's a legitimate traffic surge or a DDoS.
2. If legitimate, increase `CVKG_RATE_LIMIT_RPS` in the environment.
3. Reload/Restart the server.

### 4. Build Failures
**Symptoms**: Manual build via `/build` returns 500.
**Action**:
1. Check logs for build-specific errors.
2. Verify that `cargo` and other build dependencies are present on the host if the server is performing builds.
3. Check disk space in the `CVKG_PKG_DIR`.

## Escalation Path

1. **DevOps/SRE**: For infrastructure issues.
2. **Lead Engineer**: For application-level bugs or complex build failures.
3. **Security Team**: For suspected attacks or unauthorized access.
