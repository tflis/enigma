.PHONY: all
all: clean prepare swagger build

.PHONY: clean
clean:
	-@cargo clean 2> /dev/null ||:
	-@rm -rf ./enigmaservice 2> /dev/null ||:

.PHONY: prepare
prepare:
	-@docker rmi `docker images -f "dangling=true" -q` 2> /dev/null ||:
	docker build -t openapi --build-arg USER=${USER} -f ./Dockerfile_openapi .

.PHONY: swagger
swagger:
	docker run --rm -v `pwd`:/local openapi

.PHONY: build
build:
	cargo run -p enigma

.PHONY: docker_build
docker_build:
	docker build -t enigma .

.PHONY: docker_run
docker_run:
	docker run --rm -p 3000:3000 -v "${PWD}/config:/config" enigma