#!/bin/sh
docker run -it -v "$(pwd)":/app ekidd/rust-musl-builder /bin/sh -c "cd /app && chmod +x ./scripts/make_deb.sh && ./scripts/make_deb.sh"