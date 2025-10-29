# Memory Leak Testing Guide

This guide explains how to test Localhost HTTP server for memory leaks using various tools and methods.

## Quick Start: The Easy Way (Recommended)

### Setup

**Terminal 1 - Start the server:**
```bash
cd localhost
cargo run --release
```

Wait until you see:
```
[+] Bound to 127.0.0.1:8080
[*] Server initialized
```

**Terminal 2 - Find server PID and monitor:**
```bash
# Find the process ID
pgrep localhost
# Output example: 91242

# Monitor memory (macOS syntax)
top -pid 91242
```

**Terminal 3 - Run load test:**
```bash
# Light test (good for initial check)
siege -c50 -t60S http://127.0.0.1:8080/

# Or aggressive benchmark
siege -b http://127.0.0.1:8080/
```

### What to Watch

In Terminal 2, look at the `MEM` column:

**✅ Healthy (no leak):**
```
Initial:       993K
During load:   10M (spike during requests)
After test:    2-3M (returns close to baseline)
```

**❌ Memory leak detected:**
```
Initial:       993K
After 1min:    5M
After 2min:    15M
After 3min:    30M  ← Keeps growing = LEAK!
```

---

## Method 1: `top` with Real-Time Sorting

Most practical for quick testing.

### Commands

```bash
# Find server PID
pgrep localhost

# Monitor by memory usage (auto-sorts by memory)
top -o %MEM

# Or monitor specific PID
top -pid 91242
```

### Keyboard Shortcuts in `top`

| Key | Action |
|-----|--------|
| `o` | Change sort order |
| `Shift+M` | Sort by memory |
| `Shift+P` | Sort by CPU |
| `q` | Quit |

### Interpretation

- **RES** = actual memory used (what matters)
- **VIRT** = virtual memory allocated
- **%MEM** = percentage of system RAM

---

## Method 2: `watch` Command (Simple Alternative)

Continuous monitoring without the `top` interface.

```bash
# Terminal 1: Start server
cargo run --release

# Terminal 2: Monitor every 2 seconds
watch -n 2 'ps aux | grep localhost | grep -v grep'

# Terminal 3: Run load test
siege -c100 -t30S http://127.0.0.1:8080/
```

Output example:
```
Every 2.0s: ps aux | grep localhost

user  91242  0.5   0.2   50M  15M  ??  S  15:04  0:01 target/release/localhost
```

Watch the **5th column (RES)** — should stay stable or return to baseline after load.

---

## Method 3: Siege Load Testing Profiles

Start gentle, then increase intensity.

### Light Test (Safest)
```bash
siege -c10 -t15S http://127.0.0.1:8080/
```
- 10 concurrent connections
- 15 seconds duration
- Good for initial smoke test

### Medium Test
```bash
siege -c50 -t30S http://127.0.0.1:8080/
```
- 50 concurrent connections
- 30 seconds duration
- Realistic load

### Heavy Test
```bash
siege -c100 -t60S http://127.0.0.1:8080/
```
- 100 concurrent connections
- 60 seconds duration
- Stress test

### Aggressive Benchmark (Max Load)
```bash
siege -b http://127.0.0.1:8080/
```
- Opens as many connections as possible
- Runs until interrupted (Ctrl+C)
- **Watch server closely!**

---

# Watch Terminal 2 during and after


**Expected behavior:**
- Memory spikes during connections
- Returns to near-baseline after all complete
- No hanging connections


## Quick Reference

| Goal | Command |
|------|---------|
| Find server | `pgrep localhost` |
| Monitor memory | `top -pid <PID>` |
| Light load | `siege -c10 -t15S http://127.0.0.1:8080/` |
| Medium load | `siege -c50 -t30S http://127.0.0.1:8080/` |
| Upload test | `curl -F "file=@test.jpg" http://127.0.0.1:8080/upload` |
| Check FDs | `lsof -p $(pgrep localhost)` |

---

## Summary

**Best practice workflow:**

1. Start server: `cargo run --release`
2. Monitor in separate terminal: `top -o %MEM`
3. Run load test: `siege -c50 -t30S http://127.0.0.1:8080/`
4. Watch memory column during and after test
5. If memory returns to baseline → ✅ Healthy
6. If memory keeps growing → ❌ Leak detected
