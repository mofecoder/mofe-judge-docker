FROM rust:1.57.0 AS build

WORKDIR /work
COPY . /work
RUN cargo build --release

FROM ubuntu:20.04

ENV TZ Asia/Tokyo
ENV DEBIAN_FRONTEND=noninteractive

RUN sed -i -e 's/archive.ubuntu.com/jp.archive.ubuntu.com/g' /etc/apt/sources.list

RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        software-properties-common \
        apt-transport-https \
        dirmngr \
        curl \
        time \
        iproute2 \
        build-essential \
        apt-utils \
        sudo \
        unzip \
        git \
        gnupg \
        libssl-dev \
        ca-certificates \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Raku install
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        rakudo \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
   
# C#(mono) install
RUN \
    yes | apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF \
    && echo "deb https://download.mono-project.com/repo/ubuntu stable-focal main" | tee /etc/apt/sources.list.d/mono-official-stable.list \
    && apt-get update && apt-get install -y --no-install-recommends \
        mono-devel \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# C#(.NET) install
RUN \
    curl -OL https://download.visualstudio.microsoft.com/download/pr/820db713-c9a5-466e-b72a-16f2f5ed00e2/628aa2a75f6aa270e77f4a83b3742fb8/dotnet-sdk-5.0.100-linux-x64.tar.gz \
    && mkdir -p "$HOME/dotnet" \
    && tar zxf dotnet-sdk-5.0.100-linux-x64.tar.gz -C "$HOME/dotnet" \
    && rm dotnet-sdk-5.0.100-linux-x64.tar.gz
ENV DOTNET_ROOT="/root/dotnet"
ENV PATH="${PATH}:/root/dotnet"

# C/C++ install
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        g++-10 \
        gcc-10 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Java11 install
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        default-jdk \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Python3 install
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        python3.9 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Pypy3 install
RUN \
    cd /opt \
    && curl -OL https://downloads.python.org/pypy/pypy3.7-v7.3.3-linux64.tar.bz2 \
    && tar xf pypy3.7-v7.3.3-linux64.tar.bz2 \
    && cd /bin \
    && ln -s /opt/pypy3.7-v7.3.3-linux64/bin/pypy3 pypy3 \
    && cd /opt \
    && rm pypy3.7-v7.3.3-linux64.tar.bz2

# go install
RUN \
    curl -OL https://golang.org/dl/go1.15.5.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.15.5.linux-amd64.tar.gz \
    && rm go1.15.5.linux-amd64.tar.gz
ENV PATH="${PATH}:/usr/local/go/bin"
    
# Rust install
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="${PATH}:/root/.cargo/bin"

RUN \
    mkdir -p /judge \
    && cd judge \
    && cargo new --bin rust_workspace && cd rust_workspace \
    && curl -OL https://raw.githubusercontent.com/cafecoder-dev/language-update/20.10/Rust/Cargo.toml \
    && cargo build --release

# Nim install
RUN curl https://nim-lang.org/choosenim/init.sh -sSf | sh -s -- -y \
ENV PATH="/root/.nimble/bin:${PATH}"
    
RUN \
    curl -OL https://cache.ruby-lang.org/pub/ruby/2.7/ruby-2.7.2.tar.gz \
    && tar xvf ruby-2.7.2.tar.gz -C /root \
    && rm ruby-2.7.2.tar.gz
ENV PATH="/root/ruby-2.7.2/bin:${PATH}"

# Kotlin install
RUN \
    curl -OL https://github.com/JetBrains/kotlin/releases/download/v1.4.10/kotlin-compiler-1.4.10.zip \
    && unzip kotlin-compiler-1.4.10.zip -d /root \
    && rm kotlin-compiler-1.4.10.zip
ENV PATH="/root/kotlinc/bin:${PATH}"

# Fortran install
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        gfortran-10 \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# crystal
RUN \
    curl -sSL https://dist.crystal-lang.org/apt/setup.sh | bash \
    && apt-get update && apt-get install -y --no-install-recommends \
        crystal \
    && apt-get clean && rm -rf /var/lib/apt/lists/*
    
# Perl install
RUN \
    curl -OL https://www.cpan.org/src/5.0/perl-5.32.0.tar.gz \
    && tar -xzf perl-5.32.0.tar.gz && cd perl-5.32.0 \
    && ./Configure -Dprefix="/root/perl" -Dscriptdir="/root/perl/bin" -des -Dman1dir=none -Dman3dir=none -DDEBUGGING=-g \
    && make --jobs=8 install \
    && rm ../perl-5.32.0.tar.gz

# install external libraries
RUN \
    curl -OL https://raw.githubusercontent.com/MikeMirzayanov/testlib/master/testlib.h \
    && curl -OL https://github.com/atcoder/ac-library/releases/download/v1.0/ac-library.zip \
    && unzip ac-library.zip \
    && rm ac-library.zip

RUN \
    apt-get update && apt-get install -y --no-install-recommends \
        libcap-dev \
    && apt-get clean && rm -rf /var/lib/apt/lists/* \
    && git clone https://github.com/ioi/isolate.git /isolate
COPY ./default.cf /isolate/default.cf
RUN cd /isolate && make install

ENV DOWNLOAD_ROOT=/download

RUN \
    mkdir /download \
    && mkdir /box \
    && mkdir -p /judge/Main && chmod -R 777 /judge \
    && chmod 777 /root \
    && cp /testlib.h /judge/testlib.h

WORKDIR / 
COPY --from=build /work/target/release/cafecoder-docker-rs app
COPY --from=build /work/.env .env
COPY --from=build /work/default.cf default.cf
COPY --from=build /work/service-account-cafecoder.json service-account-cafecoder.json

RUN \
    dotnet -v ; exit 0

ENTRYPOINT ["./cafecoder-docker-rs"]
