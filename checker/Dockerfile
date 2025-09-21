FROM python:3.12@sha256:a5bbd10dc5f2d6e965e101eef58f2f15cc3be5802cad7a12897aa443839c4943

ADD src/requirements.txt .
RUN pip install -r requirements.txt

ADD src .

EXPOSE 8400

CMD ["gunicorn", "-c", "gunicorn.conf.py", "checker:app()"]
