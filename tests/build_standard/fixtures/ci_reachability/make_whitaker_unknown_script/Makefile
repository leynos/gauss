.PHONY: whitaker-unknown-script
whitaker-unknown-script:
	UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools $(UV) run --no-project --python 3.13 --with pytest==9.0.2 python -m pytest scripts/tests/unknown.py -c /dev/null --rootdir=. -p no:cacheprovider
