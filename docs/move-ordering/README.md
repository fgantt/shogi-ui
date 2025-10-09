# Move Ordering Documentation

This directory contains comprehensive documentation for the move ordering system.

## Documentation Files

### Core Documentation

- **[MOVE_ORDERING_API.md](../MOVE_ORDERING_API.md)** - Complete API reference with all methods and types
- **[MOVE_ORDERING_BEST_PRACTICES.md](../MOVE_ORDERING_BEST_PRACTICES.md)** - Best practices and recommended patterns
- **[MOVE_ORDERING_PERFORMANCE_GUIDE.md](../MOVE_ORDERING_PERFORMANCE_GUIDE.md)** - Performance tuning and optimization
- **[MOVE_ORDERING_TROUBLESHOOTING.md](../MOVE_ORDERING_TROUBLESHOOTING.md)** - Common issues and solutions

### Implementation Documentation

- **[DESIGN_MOVE_ORDERING_IMPROVEMENTS.md](../design/implementation/search-algorithm-optimizations/move-ordering-improvements/DESIGN_MOVE_ORDERING_IMPROVEMENTS.md)** - Design document
- **[IMPLEMENT_MOVE_ORDERING_IMPROVEMENTS.md](../design/implementation/search-algorithm-optimizations/move-ordering-improvements/IMPLEMENT_MOVE_ORDERING_IMPROVEMENTS.md)** - Implementation details
- **[TASKS_MOVE_ORDERING_IMPROVEMENTS.md](../design/implementation/search-algorithm-optimizations/move-ordering-improvements/TASKS_MOVE_ORDERING_IMPROVEMENTS.md)** - Task tracking

## Quick Links

### For Users

- [Quick Start](../MOVE_ORDERING_API.md#quick-start) - Get started quickly
- [Configuration Examples](../MOVE_ORDERING_API.md#configuration) - Configure for your needs
- [Best Practices](../MOVE_ORDERING_BEST_PRACTICES.md) - Do's and don'ts
- [Troubleshooting](../MOVE_ORDERING_TROUBLESHOOTING.md) - Fix common issues

### For Developers

- [API Reference](../MOVE_ORDERING_API.md#api-reference) - All methods and types
- [Integration Guide](../MOVE_ORDERING_API.md#integration) - Integrate with search engine
- [Performance Guide](../MOVE_ORDERING_PERFORMANCE_GUIDE.md) - Optimize performance
- [Implementation Details](../design/implementation/search-algorithm-optimizations/move-ordering-improvements/IMPLEMENT_MOVE_ORDERING_IMPROVEMENTS.md) - Technical details

## Examples

Working examples are available in the `/examples` directory:

- `move_ordering_usage.rs` - Basic usage examples with the actual API

## Getting Started

1. Read the [Quick Start](../MOVE_ORDERING_API.md#quick-start) section
2. Try the [basic usage example](../../examples/move_ordering_usage.rs)
3. Review [Best Practices](../MOVE_ORDERING_BEST_PRACTICES.md)
4. Tune using the [Performance Guide](../MOVE_ORDERING_PERFORMANCE_GUIDE.md)

## Feature Overview

The move ordering system provides:

### Core Features
- ✅ Basic move ordering structure
- ✅ PV (Principal Variation) move prioritization
- ✅ Killer move heuristic  
- ✅ History heuristic
- ✅ Static Exchange Evaluation (SEE)
- ✅ Move scoring integration

### Advanced Features
- ✅ Transposition table integration
- ✅ Configuration system with presets
- ✅ Performance optimization
- ✅ Advanced statistics tracking
- ✅ Error handling and recovery
- ✅ Memory management
- ✅ Position-specific strategies
- ✅ Dynamic weight adjustment

### Integration
- ✅ Search algorithm integration
- ✅ Negamax integration
- ✅ Alpha-beta pruning integration
- ✅ Quiescence search integration
- ✅ Iterative deepening integration

### Testing
- ✅ Comprehensive unit test suite (150+ tests)
- ✅ Integration tests
- ✅ Performance benchmarks
- ✅ Stress tests
- ✅ Memory leak tests
- ✅ Regression tests

## Support

If you encounter issues:

1. Check the [Troubleshooting Guide](../MOVE_ORDERING_TROUBLESHOOTING.md)
2. Review [Best Practices](../MOVE_ORDERING_BEST_PRACTICES.md)
3. Examine the working [examples](../../examples/)
4. Check the comprehensive test suite in `src/search/move_ordering.rs`

## Version Information

- Current Version: 0.4.0
- Last Updated: October 2025
- Status: Production Ready

## License

This documentation is part of the Shogi Engine project.



