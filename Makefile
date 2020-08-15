.PHONY: all clean prepare swagger build docker_build docker_run

all: clean prepare swagger build

clean:
	-@cargo clean 2> /dev/null ||:
	-@rm -rf ./enigmaservice 2> /dev/null ||:

prepare:
	-@docker rm openapi
	-@docker rmi `docker images -f "dangling=true" -q` 2> /dev/null ||:
	docker build -t openapi --build-arg USER=${USER} -f ./Dockerfile_openapi .

swagger:
	docker run --name openapi openapi
	docker cp openapi:/home/${USER} ./enigmaservice

build:
	RUST_LOG=info cargo run -p enigma

docker_build:
	docker build -t enigma .

docker_run:
	docker run --rm -p 3000:3000 -v "${PWD}/config:/config" -d enigma
