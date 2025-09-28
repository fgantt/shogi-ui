# Troubleshooting Guide

Comprehensive guide for diagnosing and resolving common issues with the automated tuning system.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Data Loading Problems](#data-loading-problems)
3. [Optimization Issues](#optimization-issues)
4. [Performance Problems](#performance-problems)
5. [Memory Issues](#memory-issues)
6. [Validation Errors](#validation-errors)
7. [File Format Issues](#file-format-issues)
8. [Error Messages Reference](#error-messages-reference)

## Installation Issues

### Rust Version Too Old

**Error**: `error: failed to select a version for the requirement 'rust-version = "1.70"'`

**Solution**:
```bash
# Update Rust to latest version
rustup update stable
rustup default stable

# Verify version
rustc --version  # Should be 1.70.0 or higher
```

### Build Failures

**Error**: `error: failed to run custom build command for '...'`

**Solutions**:
```bash
# Clean and rebuild
cargo clean
cargo build --release --bin tuner

# If still failing, try with verbose output
cargo build --release --bin tuner --verbose

# Check for missing system dependencies
# On Ubuntu/Debian:
sudo apt-get update
sudo apt-get install build-essential

# On macOS:
xcode-select --install
```

### Permission Denied

**Error**: `Permission denied (os error 13)`

**Solution**:
```bash
# Make executable
chmod +x target/release/tuner

# Or run with cargo
cargo run --release --bin tuner -- [arguments]
```

## Data Loading Problems

### File Not Found

**Error**: `No such file or directory (os error 2)`

**Solutions**:
```bash
# Check file exists
ls -la games.json

# Use absolute path
./target/release/tuner tune --dataset /full/path/to/games.json

# Check working directory
pwd
```

### Invalid JSON Format

**Error**: `invalid type: expected string, found null at line 1 column 1`

**Solutions**:
```bash
# Validate JSON format
python -m json.tool games.json

# Or use jq
jq . games.json

# Check for BOM or encoding issues
file games.json
hexdump -C games.json | head
```

### Empty Dataset

**Error**: `No valid games found in dataset`

**Solutions**:
```bash
# Check dataset size
wc -l games.json

# Validate game format
./target/release/tuner prepare-data --input games.json --output validated.json

# Check for correct game structure
head -20 games.json
```

### Unsupported Format

**Error**: `Unsupported file format: .xyz`

**Solutions**:
```bash
# Convert to supported format
# KIF/CSA to JSON:
./target/release/tuner prepare-data --input games.kif --output games.json

# PGN to JSON:
./target/release/tuner prepare-data --input games.pgn --output games.json

# Check supported formats
./target/release/tuner --help
```

## Optimization Issues

### Convergence Problems

**Symptoms**: Error not decreasing, oscillating values, NaN results

**Solutions**:
```bash
# Reduce learning rate
./target/release/tuner tune --learning-rate 0.001  # instead of 0.01

# Try different optimizer
./target/release/tuner tune --method lbfgs

# Add regularization
./target/release/tuner tune --regularization 0.01

# Check for data quality
./target/release/tuner prepare-data --input games.json --output clean.json --deduplicate
```

### Overfitting

**Symptoms**: Training error decreases but validation error increases

**Solutions**:
```bash
# Increase regularization
./target/release/tuner tune --regularization 0.01

# Reduce model complexity
./target/release/tuner tune --feature-selection

# Use early stopping
./target/release/tuner tune --early-stopping --patience 100

# Increase validation data
./target/release/tuner tune --validation-split 0.3
```

### Slow Convergence

**Symptoms**: Error decreases very slowly

**Solutions**:
```bash
# Increase learning rate
./target/release/tuner tune --learning-rate 0.02

# Try Adam optimizer
./target/release/tuner tune --method adam

# Use momentum
./target/release/tuner tune --method gradient --momentum 0.9

# Check data quality
./target/release/tuner tune --min-rating 2000
```

### Divergence

**Symptoms**: Error increases or becomes NaN/Inf

**Solutions**:
```bash
# Reduce learning rate dramatically
./target/release/tuner tune --learning-rate 0.0001

# Use gradient clipping
./target/release/tuner tune --gradient-clipping 1.0

# Check for data outliers
./target/release/tuner prepare-data --input games.json --output clean.json --remove-outliers

# Use robust optimizer
./target/release/tuner tune --method adam --beta1 0.95
```

## Performance Problems

### Slow Execution

**Symptoms**: Tuning takes much longer than expected

**Solutions**:
```bash
# Use more threads
./target/release/tuner tune --threads 16

# Reduce dataset size for testing
./target/release/tuner tune --max-positions 50000

# Use faster optimizer
./target/release/tuner tune --method lbfgs

# Enable progress monitoring
./target/release/tuner tune --progress --verbose
```

### High CPU Usage

**Symptoms**: System becomes unresponsive

**Solutions**:
```bash
# Limit threads
./target/release/tuner tune --threads 4

# Use CPU affinity
taskset -c 0-3 ./target/release/tuner tune [options]

# Reduce batch size
./target/release/tuner tune --batch-size 100

# Use lower priority
nice -n 10 ./target/release/tuner tune [options]
```

### Disk I/O Issues

**Symptoms**: High disk usage, slow checkpoints

**Solutions**:
```bash
# Reduce checkpoint frequency
./target/release/tuner tune --checkpoint-frequency 1000

# Use SSD storage
./target/release/tuner tune --checkpoint-dir /ssd/checkpoints

# Compress checkpoints
./target/release/tuner tune --compress-checkpoints

# Disable checkpoints for testing
./target/release/tuner tune --checkpoint-frequency 0
```

## Memory Issues

### Out of Memory

**Error**: `thread 'main' panicked at 'out of memory'`

**Solutions**:
```bash
# Reduce memory limit
./target/release/tuner tune --memory-limit 2048

# Use smaller batches
./target/release/tuner tune --batch-size 100

# Process data in chunks
./target/release/tuner prepare-data --input games.json --output chunk1.json --max-positions 50000
./target/release/tuner tune --dataset chunk1.json --output weights1.json

# Use memory-efficient optimizer
./target/release/tuner tune --method lbfgs
```

### Memory Leaks

**Symptoms**: Memory usage keeps increasing

**Solutions**:
```bash
# Enable memory monitoring
./target/release/tuner tune --memory-monitoring --verbose

# Use periodic garbage collection
./target/release/tuner tune --gc-frequency 100

# Restart periodically
./target/release/tuner tune --max-iterations 1000 --restart-on-checkpoint
```

### Swap Usage

**Symptoms**: System becomes very slow, high swap usage

**Solutions**:
```bash
# Monitor memory usage
htop
free -h

# Reduce memory limit
./target/release/tuner tune --memory-limit 4096

# Use swap-friendly settings
./target/release/tuner tune --memory-limit 6144 --batch-size 50
```

## Validation Errors

### Cross-Validation Failures

**Error**: `Cross-validation failed: insufficient data`

**Solutions**:
```bash
# Reduce k-fold
./target/release/tuner validate --k-fold 3

# Use holdout validation instead
./target/release/tuner validate --validation-method holdout

# Increase dataset size
./target/release/tuner prepare-data --input games.json --output larger.json --max-positions 100000
```

### Invalid Weights

**Error**: `Invalid weight file format`

**Solutions**:
```bash
# Validate weight file
./target/release/tuner validate --weights weights.json --check-format

# Recreate weights
./target/release/tuner tune --dataset games.json --output new_weights.json

# Check file corruption
md5sum weights.json
```

### Test Set Issues

**Error**: `Test set too small for reliable validation`

**Solutions**:
```bash
# Increase test split
./target/release/tuner validate --test-split 0.3

# Use larger dataset
./target/release/tuner prepare-data --input games.json --output large.json --min-games 10000

# Use different validation method
./target/release/tuner validate --validation-method bootstrap
```

## File Format Issues

### Corrupted Files

**Error**: `Failed to parse file: unexpected end of file`

**Solutions**:
```bash
# Check file integrity
file games.json
wc -l games.json

# Try to repair
./target/release/tuner prepare-data --input games.json --output repaired.json --repair

# Use backup if available
cp games.json.backup games.json
```

### Encoding Issues

**Error**: `invalid utf-8 sequence`

**Solutions**:
```bash
# Check file encoding
file -i games.json

# Convert encoding
iconv -f ISO-8859-1 -t UTF-8 games.json > games_utf8.json

# Remove BOM
sed -i '1s/^\xEF\xBB\xBF//' games.json
```

### Format Version Mismatch

**Error**: `Unsupported file format version`

**Solutions**:
```bash
# Check format version
head -10 games.json

# Convert to current format
./target/release/tuner prepare-data --input games.json --output converted.json --upgrade-format

# Use format converter
./target/release/tuner convert-format --input games.json --output new_format.json
```

## Error Messages Reference

### Common Error Codes

| Error Code | Description | Solution |
|------------|-------------|----------|
| `E001` | File not found | Check file path and permissions |
| `E002` | Invalid JSON | Validate JSON format |
| `E003` | Insufficient memory | Reduce memory limit or dataset size |
| `E004` | Convergence failure | Adjust learning rate or optimizer |
| `E005` | Invalid weights | Recreate weight file |
| `E006` | Data format error | Convert to supported format |
| `E007` | Threading error | Reduce thread count |
| `E008` | Disk space full | Free up disk space |
| `E009` | Network timeout | Check network connection |
| `E010` | Permission denied | Fix file permissions |

### Warning Messages

| Warning | Description | Action |
|---------|-------------|--------|
| `W001` | Low memory warning | Consider reducing batch size |
| `W002` | Slow convergence | Check learning rate |
| `W003` | High CPU usage | Reduce thread count |
| `W004` | Disk I/O high | Reduce checkpoint frequency |
| `W005` | Data quality low | Filter data better |
| `W006` | Overfitting detected | Increase regularization |
| `W007` | Underfitting detected | Reduce regularization |
| `W008` | Validation set small | Increase dataset size |

## Diagnostic Commands

### System Information
```bash
# Check system resources
free -h
df -h
nproc

# Check Rust version
rustc --version
cargo --version

# Check file permissions
ls -la target/release/tuner
```

### Performance Monitoring
```bash
# Monitor during tuning
htop
iotop
nethogs

# Memory usage
ps aux | grep tuner
pmap -x $(pgrep tuner)

# Disk usage
du -sh checkpoints/
```

### Data Validation
```bash
# Check dataset
./target/release/tuner validate-data --dataset games.json

# Test weight file
./target/release/tuner validate-weights --weights weights.json

# Benchmark system
./target/release/tuner benchmark-system
```

## Getting Help

### Debug Mode
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/tuner tune [options]

# Verbose output
./target/release/tuner tune --verbose --log-level debug [options]

# Save debug log
./target/release/tuner tune [options] 2>&1 | tee debug.log
```

### System Information
```bash
# Generate system report
./target/release/tuner system-info

# Check dependencies
./target/release/tuner check-dependencies

# Validate installation
./target/release/tuner validate-installation
```

### Contact Information

- **Documentation**: Check other guides in `docs/user/guides/` and `docs/user/api/`
- **Examples**: See `docs/user/api/CODE_EXAMPLES.md` and `docs/design/algorithms/OPTIMIZATION_EXAMPLES.md`
- **Issues**: Report bugs via GitHub issues
- **FAQ**: Common questions in `FAQ.md`

## Prevention Tips

### Before Starting
1. **Validate Data**: Always check data format and quality
2. **Test System**: Run small test first
3. **Monitor Resources**: Check available memory and disk space
4. **Backup Data**: Keep copies of important datasets
5. **Use Checkpoints**: Enable checkpointing for long runs

### During Execution
1. **Monitor Progress**: Watch for convergence issues
2. **Check Resources**: Monitor memory and CPU usage
3. **Save Results**: Regular checkpoints and result saving
4. **Log Everything**: Enable verbose logging for debugging
5. **Validate Incrementally**: Test results during tuning

### After Completion
1. **Validate Results**: Always test on holdout data
2. **Compare Baselines**: Measure improvement over default
3. **Document Settings**: Record successful configurations
4. **Archive Data**: Keep datasets and results for future reference
5. **Share Knowledge**: Document any issues and solutions found

## Next Steps

- [User Guide](USER_GUIDE.md) for detailed usage instructions
- [Performance Tuning Guide](PERFORMANCE_TUNING_GUIDE.md) for optimization tips
- [FAQ](FAQ.md) for common questions and answers
