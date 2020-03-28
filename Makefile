VERSION=0.0.1
SERVER_IMAGE=sebge2/sonar-as-code:$VERSION

all:
	@docker build -t $(SERVER_IMAGE) --build-arg VERSION=$(VERSION) .

push:
	@docker push $(SERVER_IMAGE)
