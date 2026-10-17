.PHONY: whitaker-extra-script
whitaker-extra-script:
	UV_CACHE_DIR=.uv-cache UV_TOOL_DIR=.uv-tools $(UV) run --no-project --python 3.13 --with pytest==9.0.2 --with pyyaml==6.0.3 python -m pytest scripts/tests/test_whitaker_contracts.py scripts/tests/extra.py -c /dev/null --rootdir=. -p no:cacheprovider
