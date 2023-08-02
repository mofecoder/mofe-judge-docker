FROM rust:slim-bookworm as builder

WORKDIR /work
RUN apt-get update && apt-get install -y libssl-dev pkg-config
COPY .env .
COPY Cargo.lock .
COPY Cargo.toml .
COPY src ./src
RUN cargo fetch

RUN cargo build --release

FROM debian:bookworm

ENV TZ Asia/Tokyo
ENV DEBIAN_FRONTEND=noninteractive

SHELL ["/bin/bash", "-c"]

# install compilers
RUN \
    apt-get update && apt-get install -y --no-install-recommends \
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
        git \
        libbz2-dev libdb-dev libreadline-dev libffi-dev  \
        libgdbm-dev liblzma-dev libncursesw5-dev libsqlite3-dev \
        libssl-dev zlib1g-dev uuid-dev pkg-config openssl
RUN mkdir -p /judge

# Raku install
RUN apt-get install -y rakudo --no-install-recommends

# C#(.NET) install
RUN wget -O dotnet-sdk.tar.gz https://download.visualstudio.microsoft.com/download/pr/87a55ae3-917d-449e-a4e8-776f82976e91/03380e598c326c2f9465d262c6a88c45/dotnet-sdk-7.0.305-linux-x64.tar.gz && \
    mkdir -p $HOME/dotnet && tar zxf dotnet-sdk.tar.gz -C $HOME/dotnet
ENV PATH $PATH:/root/dotnet
ENV DOTNET_ROOT /root/dotnet
ENV DOTNET_EnableWriteXorExecute 0
ENV COMPlus_EnableDiagnostics 0
RUN cd /judge && \
    curl -L https://raw.githubusercontent.com/cafecoder-dev/language-update/23.07/CSharp/Main.csproj -o Main.csproj && \
    echo 'Console.WriteLine();' > Main.cs && \
    dotnet publish -o /tmp -c Release -v q --nologo 1>&2 && \
    rm Main.cs

# C/C++ install
RUN apt-get install build-essential gcc-12 g++-12 -y --no-install-recommends
RUN mkdir /opt/ac-library && \
    cd /tmp && \
    wget https://github.com/atcoder/ac-library/releases/download/v1.5.1/ac-library.zip && \
    unzip /tmp/ac-library.zip -d /opt/ac-library

# Fortran install
RUN apt-get install gfortran -y --no-install-recommends

# Java install
RUN apt-get install openjdk-17-jdk -y --no-install-recommends

# Python3 install
RUN apt install libopenblas-dev liblapack-dev -y --no-install-recommends &&\
    wget https://www.python.org/ftp/python/3.11.4/Python-3.11.4.tgz && \
    tar xzf Python-3.11.4.tgz && \
    cd Python-3.11.4 && \
    ./configure --enable-optimizations && \
    make && \
    make install
RUN python3.11 -m pip install git+https://github.com/not522/ac-library-python \
    numpy==1.25.0 \
    scipy==1.11.1 \
    networkx==3.1 \
    sympy==1.12 \
    sortedcontainers==2.4.0 \
    bitarray==2.8.0

# PyPy3 install
RUN cd /opt && \
    wget https://downloads.python.org/pypy/pypy3.10-v7.3.12-linux64.tar.bz2 && \
    tar xf pypy3.10-v7.3.12-linux64.tar.bz2 && \
    ln -s /opt/pypy3.10-v7.3.12-linux64/bin/pypy3 /bin/pypy3
RUN pypy3 -m ensurepip && \
    pypy3 -m pip install --break-system-packages \
    git+https://github.com/not522/ac-library-python \
    numpy==1.25.0 \
    networkx==3.1 \
    sympy==1.12 \
    sortedcontainers==2.4.0 \
    bitarray==2.8.0

# go install
RUN cd /tmp && \
    wget https://go.dev/dl/go1.20.6.linux-amd64.tar.gz && \
    tar -C /usr/local -xzf go1.20.6.linux-amd64.tar.gz
ENV PATH $PATH:/usr/local/go/bin
ENV USER=$USER

# Rust install
RUN curl https://sh.rustup.rs -sSf | sh -s -- -y
ENV PATH $PATH:/root/.cargo/bin
RUN \
    cd judge && \
    cargo init --bin . && \
    curl -OL https://raw.githubusercontent.com/cafecoder-dev/language-update/23.07/Rust/Cargo.toml && \
    cargo build --release

# Nim install
RUN curl https://nim-lang.org/choosenim/init.sh -sSf | sh -s -- -y
ENV PATH $PATH:/root/.nimble/bin
RUN nimble install https://github.com/zer0-star/Nim-ACL

# Ruby install
RUN apt-get install make libffi-dev openssl libssl-dev zlib1g-dev libyaml-dev -y --no-install-recommends && \
    git clone https://github.com/sstephenson/rbenv.git ~/.rbenv && \
    echo 'export PATH="$HOME/.rbenv/bin:$PATH"' >> ~/.profile && \
    echo 'eval "$(rbenv init -)"' >> ~/.profile && \
    bash -c exec $SHELL -l && \
    git clone https://github.com/sstephenson/ruby-build.git ~/.rbenv/plugins/ruby-build && \
    export PATH="$HOME/.rbenv/bin:$PATH" && rbenv install 3.2.2 && rbenv global 3.2.2
ENV PATH $PATH:/root/.rbenv/bin:/root/.rbenv/shims
RUN gem install rbtree ac-library-rb sorted_set

# Kotlin install
RUN apt-get install zip unzip -y --no-install-recommends && \
    curl -s https://get.sdkman.io | bash && \
    bash && \
    echo 'source "/root/.sdkman/bin/sdkman-init.sh"' >> ~/.profile && \
    source ~/.profile && \
    sdk install kotlin
ENV PATH $PATH:/root/.sdkman/candidates/kotlin/current/bin

# Crystal install
RUN cd /opt &&  \
    wget https://github.com/crystal-lang/crystal/releases/download/1.8.2/crystal-1.8.2-1-linux-x86_64.tar.gz && \
    tar -xzf crystal-1.8.2-1-linux-x86_64.tar.gz && \
    ln -s /opt/crystal-1.8.2-1/bin/crystal /bin/crystal && \
    cd

# Perl install
RUN wget https://www.cpan.org/src/5.0/perl-5.38.0.tar.gz && \
    tar -xzf perl-5.38.0.tar.gz && \
    cd perl-5.38.0  && \
    ./Configure -Dprefix=$HOME/perl -Dscriptdir=$HOME/perl/bin -des -Dman1dir=none -Dman3dir=none -DDEBUGGING=-g && \
    make --jobs=8 install

# dc install
RUN apt-get install dc -y --no-install-recommends

# install testlib
RUN mkdir /opt/testlib && \
    wget -P /opt/testlib https://raw.githubusercontent.com/MikeMirzayanov/testlib/master/testlib.h

# install isolate
RUN \
    apt-get install libcap-dev --no-install-recommends && \
    git clone https://github.com/ioi/isolate.git /isolate
COPY ./default.cf /isolate/default.cf
RUN cd /isolate && make install

RUN apt-get clean && rm -rf /var/lib/apt/lists/* && rm -rf /tmp && mkdir /tmp

ENV DOWNLOAD_ROOT=/download
ENV DOTNET_ROOT=$HOME/dotnet

ENV DOWNLOAD_ROOT=/download
RUN mkdir /download
RUN mkdir /box
RUN mkdir -p /judge/Main && chmod -R 777 /judge
RUN chmod 777 /root

WORKDIR /

COPY default.cf .
COPY Rocket.toml .
COPY --from=builder /work/target/release/cafecoder-docker-rs app

RUN source $HOME/.profile && dotnet -v ; exit 0

ENTRYPOINT ["./app"]
