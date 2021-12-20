# Ubuntu:20.04(amd64)。
FROM --platform=linux/amd64 ubuntu:20.04

ENV TZ Asia/Tokyo
ENV DEBIAN_FRONTEND=noninteractive

RUN sed -i -e 's/archive.ubuntu.com/jp.archive.ubuntu.com/g' /etc/apt/sources.list

RUN \
    apt update && apt install -y --no-install-recommends \
        software-properties-common \
        apt-transport-https \
        dirmngr \
        curl \
        wget \
        time \
        iproute2 \
        build-essential \
        sudo \
        zip \
        unzip \
        git \
    && rm -rf /var/lib/apt/lists/*

# Raku install
RUN \
    apt update && apt install -y --no-install-recommends \
        rakudo \
    && rm -rf /var/lib/apt/lists/*
   
# C#(mono) install
RUN \
    apt update && apt install -y --no-install-recommends \
        gnupg \
        ca-certificates \
    && yes | apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys 3FA7E0328081BFF6A14DA29AA6A19B38D3D831EF \
    && echo "deb https://download.mono-project.com/repo/ubuntu stable-focal main" | tee /etc/apt/sources.list.d/mono-official-stable.list \
    && apt update && apt install -y --no-install-recommends \
        mono-devel \
    && rm -rf /var/lib/apt/lists/*

# C#(.NET) install
RUN \
    wget https://download.visualstudio.microsoft.com/download/pr/820db713-c9a5-466e-b72a-16f2f5ed00e2/628aa2a75f6aa270e77f4a83b3742fb8/dotnet-sdk-5.0.100-linux-x64.tar.gz \
    && mkdir -p $HOME/dotnet \
    && tar zxf dotnet-sdk-5.0.100-linux-x64.tar.gz -C $HOME/dotnet \
    && rm dotnet-sdk-5.0.100-linux-x64.tar.gz
ENV DOTNET_ROOT="/root/dotnet"
ENV PATH="${PATH}:/root/dotnet"

# C/C++ install
RUN \
    apt update && apt install -y --no-install-recommends \
        g++-10 \
        gcc-10 \
    && rm -rf /var/lib/apt/lists/*

# Java11 install
RUN \
    apt update && apt install -y --no-install-recommends \
        default-jdk \
    && rm -rf /var/lib/apt/lists/*

# Python3 install
RUN \
    apt update && apt install -y --no-install-recommends \
        python3.9 \
    && rm -rf /var/lib/apt/lists/*

# Pypy3 install
RUN \
    cd /opt \
    && wget https://downloads.python.org/pypy/pypy3.7-v7.3.3-linux64.tar.bz2 \
    && tar xf pypy3.7-v7.3.3-linux64.tar.bz2 \
    && cd /bin \
    && ln -s /opt/pypy3.7-v7.3.3-linux64/bin/pypy3 pypy3 \
    && cd /opt \
    && rm pypy3.7-v7.3.3-linux64.tar.bz2

# go install
RUN \
    wget https://golang.org/dl/go1.15.5.linux-amd64.tar.gz \
    && tar -C /usr/local -xzf go1.15.5.linux-amd64.tar.gz \
    && rm go1.15.5.linux-amd64.tar.gz
ENV PATH="${PATH}:/usr/local/go/bin"
    
# Rust install
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH="${PATH}:/root/.cargo/bin"

RUN \
    cargo new --bin judge && cd judge \
    && wget https://raw.githubusercontent.com/cafecoder-dev/language-update/20.10/Rust/Cargo.toml -O Cargo.toml \
    && cargo build --release

# Nim install
RUN curl https://nim-lang.org/choosenim/init.sh -sSf | sh -s -- -y \
ENV PATH="/root/.nimble/bin:${PATH}"
    
# Ruby install
RUN \
    apt update && apt install -y --no-install-recommends \
        make \
        libffi-dev \
        openssl \
        libssl-dev \
        zlib1g-dev \
    && rm -rf /var/lib/apt/lists/* \
    && git clone https://github.com/sstephenson/rbenv.git ~/.rbenv
ENV PATH="/root/.rbenv/bin:${PATH}"
    
RUN \
    echo 'eval "$(rbenv init -)"' >> ~/.profile \
    && bash -c exec $SHELL -l \
    && git clone https://github.com/sstephenson/ruby-build.git ~/.rbenv/plugins/ruby-build \
    && rbenv install 2.7.2 \
    && rbenv global 2.7.2

# Kotlin install
RUN \
    curl -s https://get.sdkman.io | bash \
    && bash && echo 'source "/root/.sdkman/bin/sdkman-init.sh"' >> ~/.profile && \
    && source ~/.profile && \
    && sdk install kotlin

# Fortran install
RUN \
    apt update && apt install -y --no-install-recommends \
        gfortran-10 \
    && rm -rf /var/lib/apt/lists/*

# crystal
RUN \
    curl -sSL https://dist.crystal-lang.org/apt/setup.sh | bash \
    && apt update && apt install -y --no-install-recommends \
        crystal \
    && rm -rf /var/lib/apt/lists/*
    
# Perl install
RUN \
    wget https://www.cpan.org/src/5.0/perl-5.32.0.tar.gz \
    && tar -xzf perl-5.32.0.tar.gz && cd perl-5.32.0 \
    && ./Configure -Dprefix=$HOME/perl -Dscriptdir=$HOME/perl/bin -des -Dman1dir=none -Dman3dir=none -DDEBUGGING=-g \
    && make --jobs=8 install \
    && rm perl-5.32.0.tar.gz

# install external libraries
RUN \
    wget https://raw.githubusercontent.com/MikeMirzayanov/testlib/master/testlib.h \
    && wget https://github.com/atcoder/ac-library/releases/download/v1.0/ac-library.zip \
    && unzip ac-library.zip \
    && rm ac-library.zip

RUN \
    apt update && apt install -y --no-install-recommends \
        libcap-dev \
    && rm -rf /var/lib/apt/lists/* \
    && git clone https://github.com/ioi/isolate.git /isolate
COPY ./default.cf /isolate/default.cf
RUN cd /isolate && make install

ENV DOWNLOAD_ROOT=/download

RUN mkdir /judge
RUN mkdir /download
RUN mkdir /box
RUN mkdir -p /judge/Main && chmod -R 777 /judge
RUN chmod 777 /root
RUN cp /testlib.h /judge/testlib.h

COPY . /cafecoder-docker-rust
RUN \
    cd cafecoder-docker-rust \
    && cargo build --release \
    && cp target/release/cafecoder-docker-rs /cafecoder-docker-rs \
    && cp .env /.env \
    && cp service-account-cafecoder.json /service-account-cafecoder.json \
    && cp default.cf /default.cf \
    && rm -rf target/

WORKDIR / 

RUN source $HOME/.profile && dotnet -v ; exit 0

ENTRYPOINT ["./cafecoder-docker-rs"]
