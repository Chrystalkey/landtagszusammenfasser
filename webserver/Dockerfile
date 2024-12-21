FROM oapi-preimage AS oapifile
FROM python:3.13-slim-bookworm AS builder

LABEL maintainer="Benedikt Schäfer"
LABEL description="Webserver for the LTZF"
LABEL version="0.1"

RUN apt-get update && apt-get install -y --no-install-recommends \
    gcc maven jq \
    && rm -rf /var/lib/apt/lists/*

RUN pip install --no-cache-dir poetry==1.4.2

ENV POETRY_NO_INTERACTION=1 \
    POETRY_VIRTUALENVS_IN_PROJECT=1 \
    POETRY_VIRTUALENVS_CREATE=1 \
    POETRY_CACHE_DIR=/tmp/poetry_cache

WORKDIR /app

COPY --from=oapifile /app/oapicode-python ./oapicode

COPY pyproject.toml poetry.lock ./
RUN touch README.md

RUN --mount=type=cache,target=$POETRY_CACHE_DIR poetry install --without dev --no-root

FROM python:3.13-slim-bookworm AS runtime

RUN groupadd -r appuser && useradd -r -g appuser appuser

RUN apt-get update && apt-get install -y --no-install-recommends wget \
    && wget "https://github.com/barnumbirr/zola-debian/releases/download/v0.19.2-1/zola_0.19.2-1_amd64_bookworm.deb" \
    && dpkg -i "zola_0.19.2-1_amd64_bookworm.deb" \
    && apt-get clean \
    && rm -rf /var/lib/apt/lists/* \
    && rm "zola_0.19.2-1_amd64_bookworm.deb"

ENV VIRTUAL_ENV=/app/.venv \
    PATH="/app/.venv/bin:$PATH"

WORKDIR /app

COPY --from=builder ${VIRTUAL_ENV} ${VIRTUAL_ENV}

COPY webserver /app/webserver
COPY zolasite /app/zolasite
COPY --from=oapifile /app/oapicode-python ./oapicode

RUN chown -R appuser:appuser /app
USER appuser

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget --no-verbose --tries=1 --spider http://localhost:8000/health || exit 1

EXPOSE 8000
EXPOSE 80

ENTRYPOINT ["python", "-m", "webserver.main"]
