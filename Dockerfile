FROM rust

USER root

RUN apt-get update && apt-get upgrade -y && apt-get install tmux sudo -y

RUN mkdir -p /app/package_check

RUN useradd -s /usr/bin/bash -m ubuntu

RUN echo "ubuntu ALL=(ALL) NOPASSWD:ALL" > /etc/sudoers.d/linux-user

WORKDIR /app/package_check

COPY . .

RUN chown -R ubuntu:ubuntu /app/package_check

USER ubuntu

RUN cargo install bacon --locked &&\
    rustup target add wasm32-unknown-unknown 

RUN curl https://raw.githubusercontent.com/pyass/neovim_setup/main/setup_neovim.sh | bash - 

RUN curl -sL https://deb.nodesource.com/setup_18.x | sudo -E bash - && sudo apt install nodejs -y

RUN cargo build

EXPOSE 9000

CMD ["bash"]
