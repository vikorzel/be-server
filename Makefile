pyenv:
	python3 -m venv .pyenv
	. .pyenv/bin/activate
	pip3 install -r requirements.txt

integration-tests: pyenv
	. .pyenv/bin/activate
	pytest integration
	
