#!/usr/bin/env python3
"""
Unbound PTR Record Generator
Automatically generates PTR records from A records in Unbound configuration files.
Designed for Jenkins/K8s deployment pipeline integration.
"""

import re
import os
import sys
import glob
import ipaddress
from pathlib import Path
from collections import defaultdict

def get_reverse_zone(ip):
    """Get reverse zone for IP (Class C)."""
    parts = ip.split('.')
    return f"{parts[2]}.{parts[1]}.{parts[0]}.in-addr.arpa"

def get_ip_segment(ip):
    """Get IP segment (first 3 octets) for grouping."""
    parts = ip.split('.')
    return f"{parts[0]}-{parts[1]}-{parts[2]}"

def is_private_ip(ip):
    """Check if IP address is private (RFC 1918)."""
    try:
        return ipaddress.IPv4Address(ip).is_private
    except ipaddress.AddressValueError:
        return False

def extract_a_records(zone_file):
    """Extract A records from zone file."""
    records = []
    zone_name = Path(zone_file).stem.replace('-zone', '').replace('-', '.')

    with open(zone_file, 'r') as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith('#') or line.startswith(';'):
                continue

            pattern = r'local-data:\s*"([^\s"]+)\s+.*?([0-9]+\.[0-9]+\.[0-9]+\.[0-9]+)"'

            match = re.search(pattern, line)
            if match:
                hostname, ip = match.groups()
                hostname = hostname.rstrip('.')  # Remove trailing dot if present

                # Only include private IPs for PTR generation
                if is_private_ip(ip):
                    records.append((hostname, ip))
                else:
                    print(f"  Skipping public IP: {ip} for {hostname}")

    return records

def generate_ptr_files(records_by_segment, config_dir):
    """Generate PTR configuration files grouped by IP segment."""

    for segment, records in records_by_segment.items():
        # Create output filename based on IP segment
        output_file = Path(config_dir) / f"{segment}-ptrs.conf"

        # Get reverse zone for this segment
        sample_ip = records[0][1]  # Get IP from first record
        reverse_zone = get_reverse_zone(sample_ip)

        # Write PTR file
        with open(output_file, 'w') as f:
            f.write(f"# PTR records for IP segment {segment.replace('-', '.')}.x\n")
            f.write(f"# Total records: {len(records)}\n")
            f.write(f"# Generated from multiple zone files\n\n")

            f.write("server:\n")
            # Write reverse zone definition
            f.write(f'    local-zone: "{reverse_zone}." static\n')

            # Write PTR records sorted by IP
            for hostname, ip in sorted(records, key=lambda x: tuple(map(int, x[1].split('.')))):
                f.write(f'    local-data-ptr: "{ip} {hostname}"\n')

        print(f"Generated {output_file} with {len(records)} PTR records")

def main():
    script_directory = os.path.dirname(__file__)
    config_dir = os.path.join(script_directory, '..', 'conf')

    if len(sys.argv) > 1:
        # Process specific file
        zone_file = sys.argv[1]

        if not Path(zone_file).exists():
            print(f"Error: File {zone_file} not found")
            sys.exit(1)

        zone_files = [zone_file]
    else:
        # Process all zone files in current directory
        zone_files = glob.glob(f"{config_dir}/*-zone.conf")

        if not zone_files:
            print(f"No *-zone.conf files found in {config_dir}")
            sys.exit(1)

    # Collect all A records from all zone files
    all_records = []

    for zone_file in zone_files:
        print(f"Processing {zone_file}...")
        try:
            a_records = extract_a_records(zone_file)
            if a_records:
                all_records.extend(a_records)
                print(f"  Found {len(a_records)} A records")
            else:
                print(f"  No A records found in {zone_file}")
        except Exception as e:
            print(f"  Error processing {zone_file}: {e}")

    if not all_records:
        print("No A records found in any zone files")
        sys.exit(1)

    # Group records by IP segment
    records_by_segment = defaultdict(list)

    for hostname, ip in all_records:
        segment = get_ip_segment(ip)
        records_by_segment[segment].append((hostname, ip))

    print(f"\nFound {len(all_records)} total A records across {len(records_by_segment)} IP segments:")
    for segment, records in records_by_segment.items():
        print(f"  {segment.replace('-', '.')}.x: {len(records)} records")

    # Generate PTR files grouped by IP segment
    print(f"\nGenerating PTR files in {config_dir}...")
    generate_ptr_files(records_by_segment, config_dir)

    print("\nPTR generation completed!")

if __name__ == "__main__":
    main()
