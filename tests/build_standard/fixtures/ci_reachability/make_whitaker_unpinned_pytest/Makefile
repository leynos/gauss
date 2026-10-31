.PHONY: whitaker-unpinned-pytest
whitaker-unpinned-pytest:
	UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools $(UV) run --no-project --python 3.13 python -m pytest scripts/tests/test_whitaker_contracts.py -c /dev/null --rootdir=. -p no:cacheprovider
