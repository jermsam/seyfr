#!/usr/bin/env python3
import sys
from app import SeyfrApplication

def main():
    app = SeyfrApplication()
    return app.run(sys.argv)

if __name__ == '__main__':
    sys.exit(main())
