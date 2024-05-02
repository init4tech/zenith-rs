FROM arm64v8/ubuntu:latest

RUN apt-get update && apt-get install -y curl unzip
RUN curl "https://awscli.amazonaws.com/awscli-exe-linux-aarch64.zip" -o "awscliv2.zip"
RUN unzip awscliv2.zip
RUN ./aws/install

COPY target/aarch64-unknown-linux-gnu/release/zenith-builder-example /usr/local/bin/zenith-builder-example

# Run the server
CMD ["/usr/local/bin/zenith-builder-example"]
