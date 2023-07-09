# Ubuntu:20.04(amd64)
# m1 mac だとハッシュを指定しないと arm64 で build してしまうので・・。
FROM ubuntu@sha256:e3d7ff9efd8431d9ef39a144c45992df5502c995b9ba3c53ff70c5b52a848d9c

ENV TZ Asia/Tokyo
ENV DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-c"]

# install compilers
RUN \
    apt update && apt-get install -y \
        software-properties-common \
        apt-transport-https \
        dirmngr \
        curl \
        wget \
        time \
        iproute2 \
        build-essential \
        sudo \
        unzip \
        git

# Raku install
RUN apt-get install -y rakudo

# C#(.NET) install

RUN wget -O dotnet-sdk.tar.gz https://download.visualstudio.microsoft.com/download/pr/87a55ae3-917d-449e-a4e8-776f82976e91/03380e598c326c2f9465d262c6a88c45/dotnet-sdk-7.0.305-linux-x64.tar.gz && \
    mkdir -p $HOME/dotnet && tar zxf dotnet-sdk.tar.gz -C $HOME/dotnet && \
    echo 'export DOTNET_ROOT=$HOME/dotnet' >> ~/.profile && \
    echo 'export PATH=$PATH:$HOME/dotnet' >> ~/.profile

# C/C++ install
RUN apt-get install g++-13 gcc-13 -y

# Java11 install
RUN apt-get install default-jdk -y

# Python3 install
RUN apt-get install python3.11 -y

# PyPy3 install
RUN cd /opt && \
    wget -O pypy3.tar.bz2 https://downloads.python.org/pypy/pypy3.10-v7.3.12-aarch64.tar.bz2 && \
    tar xf pypy3.tar.bz2 && \
    cd /bin && \
    ln -s /opt/pypy3/bin/pypy3 pypy3

# go install
RUN wget -O go.tar.gz https://go.dev/dl/go1.20.5.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go.tar.gz && \
    echo 'export PATH=$PATH:/usr/local/go/bin' >> ~/.profile

ENV USER=$USER

# Rust install
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y && \
    source $HOME/.cargo/env && \
    cargo new rust_workspace && \
    cd rust_workspace &&\
    wget https://raw.githubusercontent.com/cafecoder-dev/language-update/23.07/Rust/Cargo.toml -O Cargo.toml && \
    cargo build --release && \
    cd /

# Nim install
RUN curl https://nim-lang.org/choosenim/init.sh -sSf | sh -s -- -y && \
    echo 'export PATH=/root/.nimble/bin:$PATH' >> ~/.profile

# Ruby install
RUN apt-get install make libffi-dev openssl libssl-dev zlib1g-dev -y && \
    git clone https://github.com/sstephenson/rbenv.git ~/.rbenv && \
    echo 'export PATH="$HOME/.rbenv/bin:$PATH"' >> ~/.profile && \
    echo 'eval "$(rbenv init -)"' >> ~/.profile && \
    bash -c exec $SHELL -l && \
    git clone https://github.com/sstephenson/ruby-build.git ~/.rbenv/plugins/ruby-build && \
    export PATH="$HOME/.rbenv/bin:$PATH" && rbenv install 3.2.2 && rbenv global 3.2.2

# Kotlin install
RUN apt-get install zip unzip -y && \
    curl -s https://get.sdkman.io | bash && \
    bash && \
    echo 'source "/root/.sdkman/bin/sdkman-init.sh"' >> ~/.profile && \
    source ~/.profile && \
    sdk install kotlin

# Fortran install
RUN apt-get install gfortran-10 -y

# Crystal install
RUN curl -sSL https://crystal-lang.org/install.sh | bash -s

# Perl install
RUN wget -O perl.tar.gz https://www.cpan.org/src/5.0/perl-5.38.0.tar.gz && \
    tar -xzf perl.tar.gz && \
    cd perl && \
    ./Configure -Dprefix=$HOME/perl -Dscriptdir=$HOME/perl/bin -des -Dman1dir=none -Dman3dir=none -DDEBUGGING=-g && \
    make --jobs=8 install

# dc install
RUN apt-get install dc -y

# install external libraries
RUN \
    wget https://raw.githubusercontent.com/MikeMirzayanov/testlib/master/testlib.h && \
    wget https://github.com/atcoder/ac-library/releases/download/v1.4/ac-library.zip && unzip ac-library.zip

RUN \
    apt-get install libcap-dev && \
    git clone https://github.com/ioi/isolate.git /isolate
COPY ./default.cf /isolate/default.cf
RUN cd /isolate && make install

RUN apt-get clean && rm -rf /var/lib/apt/lists/*

ENV DOWNLOAD_ROOT=/download
ENV DOTNET_ROOT=$HOME/dotnet
ENV PATH $PATH:$HOME/dotnet
ENV PATH $PATH:/usr/local/go/bin
ENV PATH $PATH:$HOME/.cargo/bin
ENV PATH $PATH:/root/.nimble/bin
ENV PATH $PATH:$HOME/.rbenv/bin
ENV PATH $PATH:/root/.sdkman/candidates/kotlin/current/bin

RUN mkdir /judge
RUN mkdir /download
RUN mkdir /box
RUN mv /rust_workspace /judge
RUN mkdir -p /judge/Main && chmod -R 777 /judge
RUN chmod 777 /root
RUN cp /testlib.h /judge/testlib.h

#RUN mkdir -p /cafecoder-docker-rust/src
#COPY dummy.rs /cafecoder-docker-rust/src/main.rs
#COPY Cargo.toml /cafecoder-docker-rust
#COPY Cargo.lock /cafecoder-docker-rust
#COPY .env /cafecoder-docker-rust
#COPY service-account-cafecoder.json /cafecoder-docker-rust
#COPY default.cf /cafecoder-docker-rust
#COPY service-account-cafecoder.json /cafecoder-docker-rust
#RUN cd cafecoder-docker-rust && source $HOME/.cargo/env && cargo build --release

#COPY src/ /cafecoder-docker-rust/src
COPY . /cafecoder-docker-rust
RUN cd cafecoder-docker-rust && \
    source $HOME/.cargo/env && \
    cargo build --release && \
    cp target/release/cafecoder-docker-rs / && \
    cp .env / && \
    cp service-account-cafecoder.json / && \
    cp default.cf / && \
    mkdir /temp

WORKDIR /

RUN source $HOME/.profile && dotnet -v ; exit 0

ENTRYPOINT ["./cafecoder-docker-rs"]
