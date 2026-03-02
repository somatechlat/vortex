#!/usr/bin/env python
"""
VORTEX Django Admin Management Script

This Django app provides read-only monitoring for the Rust execution engine.
It connects to the SAME PostgreSQL database that Rust uses via SeaORM.
"""
import os
import sys

if __name__ == "__main__":
    os.environ.setdefault("DJANGO_SETTINGS_MODULE", "vortex_admin.settings")
    
    # Add the django_admin directory to Python path
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    
    try:
        from django.core.management import execute_from_command_line
    except ImportError as exc:
        raise ImportError(
            "Couldn't import Django. Are you sure it's installed?"
        ) from exc
    
    execute_from_command_line(sys.argv)
