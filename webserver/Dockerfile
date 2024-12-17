FROM python:3.13-bookworm as builder

RUN pip install poetry==1.4.2

ENV POETRY_NO_INTERACTION=1 \
    POETRY_VIRTUALENVS_IN_PROJECT=1 \
    POETRY_VIRTUALENVS_CREATE=1 \
    POETRY_CACHE_DIR=/tmp/poetry_cache

WORKDIR /app

COPY pyproject.toml poetry.lock ./
COPY ./oapicode ./oapicode
RUN touch README.md

RUN --mount=type=cache,target=$POETRY_CACHE_DIR poetry install --without dev --no-root

FROM python:3.13-slim-bookworm as runtime

RUN apt update && apt install -y python3 wget
RUN wget "https://github.com/barnumbirr/zola-debian/releases/download/v0.19.2-1/zola_0.19.2-1_amd64_bookworm.deb"
RUN dpkg -i "zola_0.19.2-1_amd64_bookworm.deb" 

ENV VIRTUAL_ENV=/app/.venv \
    PATH="/app/.venv/bin:$PATH"

COPY --from=builder ${VIRTUAL_ENV} ${VIRTUAL_ENV}

COPY webserver ./webserver
COPY zolasite zolasite
COPY ./oapicode ./oapicode

ENTRYPOINT ["python", "-m", "webserver.main"]
