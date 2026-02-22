set shell := ["bash", "-eu", "-o", "pipefail", "-c"]

translation_plan *ARGS:
	if [ ! -x .venv/bin/python ]; then echo "virtualenv missing (.venv/bin/python)" >&2; echo "run 'python3 -m venv .venv && .venv/bin/pip install polars' first" >&2; exit 1; fi
	.venv/bin/python tools/translation_plan.py {{ARGS}}
