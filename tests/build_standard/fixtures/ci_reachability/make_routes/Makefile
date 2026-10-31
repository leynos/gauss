.PHONY: verify-stable mode-suite host-suite
verify-stable: test
# An explicit empty recipe retains this pattern; its prerequisite owns the suite.
suite-%: test
	
ifeq ($(MODE),coverage)
mode-suite: test
else
mode-suite: test-ci
endif
ifeq ($(BUILD_HOST_OS),Linux)
host-suite: test
else
host-suite: test-ci
endif
