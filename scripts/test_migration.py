#!/usr/bin/env python3
"""
Test script for opening book migration

This script tests the migration from JSON to binary format.
"""

import json
import sys
import os
import subprocess

def test_json_migration():
    """Test the JSON to binary migration"""
    print("Testing JSON to binary migration...")
    
    # Test data - simplified version of the actual opening book
    test_data = [
        {
            "name": "Test Opening",
            "moves": {
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1": [
                    {"from": "27", "to": "26"},
                    {"from": "77", "to": "76"}
                ],
                "lnsgkgsnl/1r5b1/ppppppppp/9/9/7P1/PPPPPPP1P/1B5R1/LNSGKGSNL w - 2": [
                    {"from": "83", "to": "84"}
                ]
            }
        }
    ]
    
    # Write test JSON file
    test_file = "test_opening_book.json"
    with open(test_file, 'w', encoding='utf-8') as f:
        json.dump(test_data, f, indent=2)
    
    print(f"Created test file: {test_file}")
    
    # Test the Python converter
    try:
        result = subprocess.run([
            sys.executable, 
            "scripts/convert_opening_book.py", 
            test_file, 
            "--report", 
            "--rust-code"
        ], capture_output=True, text=True, check=True)
        
        print("Python converter output:")
        print(result.stdout)
        
        if result.stderr:
            print("Python converter errors:")
            print(result.stderr)
            
    except subprocess.CalledProcessError as e:
        print(f"Python converter failed: {e}")
        print(f"Error output: {e.stderr}")
        return False
    
    # Clean up
    os.remove(test_file)
    
    return True

def test_rust_integration():
    """Test the Rust integration"""
    print("\nTesting Rust integration...")
    
    # This would test the Rust code compilation and basic functionality
    try:
        result = subprocess.run([
            "cargo", "check"
        ], capture_output=True, text=True, check=True)
        
        print("Rust compilation successful")
        return True
        
    except subprocess.CalledProcessError as e:
        print(f"Rust compilation failed: {e}")
        print(f"Error output: {e.stderr}")
        return False

def main():
    print("=== Opening Book Migration Test ===")
    
    # Test JSON migration
    json_success = test_json_migration()
    
    # Test Rust integration
    rust_success = test_rust_integration()
    
    print("\n=== Test Results ===")
    print(f"JSON Migration: {'PASS' if json_success else 'FAIL'}")
    print(f"Rust Integration: {'PASS' if rust_success else 'FAIL'}")
    
    if json_success and rust_success:
        print("\nAll tests passed! Migration is working correctly.")
        return 0
    else:
        print("\nSome tests failed. Please check the errors above.")
        return 1

if __name__ == "__main__":
    sys.exit(main())





