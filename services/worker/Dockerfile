FROM python:3.11-slim
WORKDIR /app

COPY app app
COPY pyproject.toml pyproject.toml

ENV PYTHONUNBUFFERED=1

RUN useradd --system --uid 10001 workeruser

USER workeruser
EXPOSE 8090

ENTRYPOINT ["python", "-m", "app.server"]
