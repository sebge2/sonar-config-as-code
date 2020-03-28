FROM frolvlad/alpine-glibc:glibc-2.31

ARG VERSION=0.0.0
RUN wget https://github.com/sebge2/sonar-config-as-code/releases/download/v$VERSION/sonar-as-code-linux-amd64 -O /usr/bin/sonar-as-code
RUN chmod ugo+x /usr/bin/sonar-as-code
